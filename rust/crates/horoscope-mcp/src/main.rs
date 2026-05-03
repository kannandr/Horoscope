use axum::{
    extract::Json,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Json as JsonResponse, Response},
    routing::{get, post},
    Router,
};
use horoscope_core::{calculate_south_indian_natal_chart, NatalChartRequest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
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

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .route("/mcp", post(mcp))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("bind MCP listener");
    tracing::info!(%bind, "horoscope-mcp listening");
    axum::serve(listener, app).await.expect("serve MCP");
}

async fn mcp(headers: HeaderMap, Json(req): Json<JsonRpcRequest>) -> Response {
    let id = req.id.clone();
    if !is_authorized(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            JsonResponse(err(
                id,
                -32001,
                "MCP shared password is required".to_string(),
                Some(json!({
                    "hint": "Pass Authorization: Bearer <MCP_SHARED_SECRET> or x-mcp-password: <MCP_SHARED_SECRET>."
                })),
            )),
        )
            .into_response();
    }

    let id = req.id.clone();
    let out = match req.method.as_str() {
        "initialize" => ok(
            id,
            json!({
                "protocolVersion": "2025-06-18",
                "serverInfo": { "name": "horoscope-mcp", "version": env!("CARGO_PKG_VERSION") },
                "capabilities": { "tools": {} }
            }),
        ),
        "tools/list" => ok(
            id,
            json!({
                "tools": [
                    {
                        "name": "calculate_south_indian_natal_chart",
                        "description": "South Indian sidereal natal chart: lagna, Sun/Moon, Panchang at birth, Tamil hints, Vimshottari dasha–bhukti from birth through as-of + horizon years.",
                        "inputSchema": natal_chart_schema()
                    }
                ]
            }),
        ),
        "tools/call" => call_tool(id, req.params),
        _ => err(
            id,
            -32601,
            format!("unknown MCP method: {}", req.method),
            None,
        ),
    };
    (StatusCode::OK, JsonResponse(out)).into_response()
}

fn is_authorized(headers: &HeaderMap) -> bool {
    let Some(secret) = configured_shared_secret() else {
        return true;
    };

    let bearer_ok = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().strip_prefix("Bearer "))
        .is_some_and(|candidate| candidate.trim() == secret);

    let password_header_ok = headers
        .get("x-mcp-password")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|candidate| candidate.trim() == secret);

    bearer_ok || password_header_ok
}

fn configured_shared_secret() -> Option<String> {
    std::env::var("MCP_SHARED_SECRET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn call_tool(id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
    let Some(params) = params else {
        return err(id, -32602, "missing params".to_string(), None);
    };
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let result = match name {
        "calculate_south_indian_natal_chart" => serde_json::from_value::<NatalChartRequest>(args)
            .map_err(|e| e.to_string())
            .and_then(|r| calculate_south_indian_natal_chart(r).map_err(|e| e.to_string()))
            .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string())),
        _ => return err(id, -32602, format!("unknown tool: {name}"), None),
    };
    match result {
        Ok(value) => ok(
            id,
            json!({ "content": [{ "type": "text", "text": value.to_string() }], "structuredContent": value }),
        ),
        Err(message) => err(
            id,
            -32000,
            "tool call failed".to_string(),
            Some(json!({ "message": message })),
        ),
    }
}

fn ok(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

fn err(id: Option<Value>, code: i32, message: String, data: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data,
        }),
    }
}

fn natal_chart_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "birth_local": {
                "type": "string",
                "description": "Birth datetime: RFC3339 with offset, or YYYY-MM-DDTHH:MM:SS (wall time in `timezone`)."
            },
            "timezone": { "type": "string", "description": "IANA timezone, e.g. Asia/Kolkata" },
            "latitude": { "type": "number" },
            "longitude": { "type": "number" },
            "ayanamsha": { "type": "string", "enum": ["lahiri", "lahiri_alt_stub", "raman"] },
            "engine": { "type": "string", "enum": ["meeus", "surya_mean"] },
            "dasha_horizon_years": {
                "type": "integer",
                "description": "Emit dasha–bhukti intersecting birth through as-of plus this many calendar years (default 20)."
            },
            "as_of_local": {
                "type": "string",
                "description": "Optional anchor for dasha window end (same formats as birth_local). Defaults to now in `timezone`."
            }
        },
        "required": ["birth_local", "timezone", "latitude", "longitude"]
    })
}
