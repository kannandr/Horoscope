use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use muhurta_engine::{
    search_muhurta, EngineError, ErrorResponse, InProcessPanchangClient, McpPanchangClient,
    MuhurtaSearchRequest, MuhurtaSearchResponse, MuhurtaWindow, PanchangClient,
};
use panchang_core::{AyanamshaId, EngineId};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(OpenApi)]
#[openapi(
    paths(muhurta_handler, healthz, readyz),
    components(schemas(
        MuhurtaSearchRequest,
        MuhurtaSearchResponse,
        MuhurtaWindow,
        AyanamshaId,
        EngineId,
        ErrorResponse
    )),
    tags((
        name = "muhurta",
        description = "Auspicious-time scoring usage app on top of the Panchang core. \
                       Calls panchang-mcp over JSON-RPC (PANCHANG_MCP_BASE_URL); \
                       falls back to in-process panchang-core if that is unset."
    ))
)]
struct ApiDoc;

#[derive(Clone)]
struct AppState {
    client: Arc<dyn PanchangClient>,
    data_source: &'static str,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let state = build_state();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/openapi.json", get(openapi))
        .route("/v1/muhurta/search", post(muhurta_handler))
        .with_state(state)
        .layer(middleware::from_fn(request_id))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8090".to_string());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("bind muhurta-api listener");
    tracing::info!(%bind, "muhurta-api listening");
    axum::serve(listener, app).await.expect("serve muhurta-api");
}

/// Pick a Panchang client at startup based on env:
///
/// - `PANCHANG_MCP_BASE_URL` set → wire-mode `McpPanchangClient` (production).
/// - unset → in-process `panchang-core` fallback. Logs a warn so this is
///   never a silent regression.
///
/// `MCP_SHARED_SECRET` is forwarded as `Authorization: Bearer …` when set.
fn build_state() -> AppState {
    match std::env::var("PANCHANG_MCP_BASE_URL").ok().filter(|v| !v.trim().is_empty()) {
        Some(url) => {
            let secret = std::env::var("MCP_SHARED_SECRET").ok();
            let client = McpPanchangClient::new(url.clone(), secret);
            tracing::info!(
                endpoint = client.endpoint(),
                "muhurta-api: data source = panchang-mcp (wire mode)"
            );
            AppState {
                client: Arc::new(client),
                data_source: "panchang-mcp",
            }
        }
        None => {
            tracing::warn!(
                "muhurta-api: PANCHANG_MCP_BASE_URL is unset; falling back to in-process \
                 panchang-core. This is convenient for local dev but the production wire \
                 boundary requires the MCP client."
            );
            AppState {
                client: Arc::new(InProcessPanchangClient),
                data_source: "in-process",
            }
        }
    }
}

async fn request_id(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(request_id.clone());
    let mut res = next.run(req).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        res.headers_mut().insert("x-request-id", value);
    }
    res
}

#[utoipa::path(get, path = "/healthz", responses((status = 200, body = String)))]
async fn healthz() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/readyz", responses((status = 200, body = String)))]
async fn readyz(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ready",
        "data_source": state.data_source
    }))
}

async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[utoipa::path(
    post,
    path = "/v1/muhurta/search",
    request_body = MuhurtaSearchRequest,
    responses(
        (status = 200, body = MuhurtaSearchResponse),
        (status = 400, body = ErrorResponse),
        (status = 502, body = ErrorResponse)
    )
)]
async fn muhurta_handler(
    State(state): State<AppState>,
    Json(req): Json<MuhurtaSearchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let resp = search_muhurta(state.client.as_ref(), req).await?;
    Ok(Json(resp))
}

struct ApiError(EngineError);

impl From<EngineError> for ApiError {
    fn from(value: EngineError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.0.http_status()).unwrap_or(StatusCode::BAD_GATEWAY);
        match &self.0 {
            EngineError::Transport { tool, .. }
            | EngineError::McpRpc { tool, .. }
            | EngineError::Decode { tool, .. } => {
                tracing::warn!(tool = %tool, error = %self.0, "muhurta upstream MCP failed");
            }
            EngineError::Calculation(_) => {}
        }
        (
            status,
            Json(ErrorResponse {
                error: self.0.to_string(),
                request_id: None,
            }),
        )
            .into_response()
    }
}
