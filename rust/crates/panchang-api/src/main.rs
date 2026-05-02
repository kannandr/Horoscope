use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use panchang_core::{
    civil_day, month, panchang_day, search_muhurta, snapshot, CivilDayRequest, ErrorResponse,
    MonthRequest, MuhurtaSearchRequest, PanchangDayRequest, PanchangError, SnapshotRequest,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(OpenApi)]
#[openapi(
    paths(snapshot_handler, civil_day_handler, panchang_day_handler, month_handler, muhurta_handler, healthz, readyz),
    components(schemas(
        SnapshotRequest,
        panchang_core::SnapshotResponse,
        CivilDayRequest,
        panchang_core::CivilDayResponse,
        PanchangDayRequest,
        panchang_core::PanchangDayResponse,
        MonthRequest,
        panchang_core::MonthResponse,
        MuhurtaSearchRequest,
        panchang_core::MuhurtaSearchResponse,
        ErrorResponse
    )),
    tags((name = "panchang", description = "Native Panchang calculation API"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/openapi.json", get(openapi))
        .route("/v1/panchang/snapshot", post(snapshot_handler))
        .route("/v1/panchang/civil-day", post(civil_day_handler))
        .route("/v1/panchang/day", post(panchang_day_handler))
        .route("/v1/panchang/month", post(month_handler))
        .route("/v1/muhurta/search", post(muhurta_handler))
        .layer(middleware::from_fn(request_id))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("bind API listener");
    tracing::info!(%bind, "panchang-api listening");
    axum::serve(listener, app).await.expect("serve API");
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
async fn readyz() -> &'static str {
    "ready"
}

async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[utoipa::path(post, path = "/v1/panchang/snapshot", request_body = SnapshotRequest, responses((status = 200, body = panchang_core::SnapshotResponse), (status = 400, body = ErrorResponse)))]
async fn snapshot_handler(Json(req): Json<SnapshotRequest>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(snapshot(req)?))
}

#[utoipa::path(post, path = "/v1/panchang/civil-day", request_body = CivilDayRequest, responses((status = 200, body = panchang_core::CivilDayResponse), (status = 400, body = ErrorResponse)))]
async fn civil_day_handler(
    Json(req): Json<CivilDayRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(civil_day(req)?))
}

#[utoipa::path(post, path = "/v1/panchang/day", request_body = PanchangDayRequest, responses((status = 200, body = panchang_core::PanchangDayResponse), (status = 400, body = ErrorResponse)))]
async fn panchang_day_handler(
    Json(req): Json<PanchangDayRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(panchang_day(req)?))
}

#[utoipa::path(post, path = "/v1/panchang/month", request_body = MonthRequest, responses((status = 200, body = panchang_core::MonthResponse), (status = 400, body = ErrorResponse)))]
async fn month_handler(Json(req): Json<MonthRequest>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(month(req)?))
}

#[utoipa::path(post, path = "/v1/muhurta/search", request_body = MuhurtaSearchRequest, responses((status = 200, body = panchang_core::MuhurtaSearchResponse), (status = 400, body = ErrorResponse)))]
async fn muhurta_handler(
    Json(req): Json<MuhurtaSearchRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(search_muhurta(req)?))
}

struct ApiError(PanchangError);

impl From<PanchangError> for ApiError {
    fn from(value: PanchangError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            PanchangError::InvalidTimezone(_)
            | PanchangError::InvalidDateTime(_)
            | PanchangError::InvalidDate(_)
            | PanchangError::InvalidCoordinates => StatusCode::BAD_REQUEST,
            PanchangError::Calculation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };
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
