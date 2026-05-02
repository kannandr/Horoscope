use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse, Response},
    routing::{get, post},
    Router,
};
use panchang_core::{
    civil_day, search_muhurta, snapshot, CivilDayRequest, MuhurtaSearchRequest, SnapshotRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
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
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .route("/mcp", post(mcp))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await.expect("bind MCP listener");
    tracing::info!(%bind, "panchang-mcp listening");
    axum::serve(listener, app).await.expect("serve MCP");
}

async fn mcp(Json(req): Json<JsonRpcRequest>) -> Response {
    let id = req.id.clone();
    let out = match req.method.as_str() {
        "initialize" => ok(id, json!({
            "protocolVersion": "2025-06-18",
            "serverInfo": { "name": "panchang-mcp", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": { "tools": {} }
        })),
        "tools/list" => ok(id, json!({
            "tools": [
                {
                    "name": "calculate_panchang_snapshot",
                    "description": "Calculate Panchang angas and local transition times for an observer and local datetime.",
                    "inputSchema": snapshot_schema()
                },
                {
                    "name": "list_civil_day_segments",
                    "description": "List tithi and nakshatra intervals intersecting one local civil day.",
                    "inputSchema": civil_day_schema()
                },
                {
                    "name": "search_auspicious_windows",
                    "description": "Search South Indian/Tamil-focused auspicious time windows with transparent scoring reasons.",
                    "inputSchema": muhurta_schema()
                },
                {
                    "name": "explain_auspicious_window",
                    "description": "Explain the scoring reasons returned by search_auspicious_windows.",
                    "inputSchema": { "type": "object", "properties": { "window": { "type": "object" } }, "required": ["window"] }
                }
            ]
        })),
        "tools/call" => call_tool(id, req.params),
        _ => err(id, -32601, format!("unknown MCP method: {}", req.method), None),
    };
    (StatusCode::OK, JsonResponse(out)).into_response()
}

fn call_tool(id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
    let Some(params) = params else {
        return err(id, -32602, "missing params".to_string(), None);
    };
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let result = match name {
        "calculate_panchang_snapshot" => serde_json::from_value::<SnapshotRequest>(args)
            .map_err(|e| e.to_string())
            .and_then(|r| snapshot(r).map_err(|e| e.to_string()))
            .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string())),
        "list_civil_day_segments" => serde_json::from_value::<CivilDayRequest>(args)
            .map_err(|e| e.to_string())
            .and_then(|r| civil_day(r).map_err(|e| e.to_string()))
            .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string())),
        "search_auspicious_windows" => serde_json::from_value::<MuhurtaSearchRequest>(args)
            .map_err(|e| e.to_string())
            .and_then(|r| search_muhurta(r).map_err(|e| e.to_string()))
            .and_then(|r| serde_json::to_value(r).map_err(|e| e.to_string())),
        "explain_auspicious_window" => Ok(json!({
            "content": [{
                "type": "text",
                "text": "Use the window.score, window.reasons, and window.exclusions fields. Reasons are positive rule matches; exclusions are cautionary or disqualifying rule matches."
            }]
        })),
        _ => return err(id, -32602, format!("unknown tool: {name}"), None),
    };
    match result {
        Ok(value) if name == "explain_auspicious_window" => ok(id, value),
        Ok(value) => ok(id, json!({ "content": [{ "type": "text", "text": value.to_string() }], "structuredContent": value })),
        Err(message) => err(id, -32000, "tool call failed".to_string(), Some(json!({ "message": message }))),
    }
}

fn ok(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse { jsonrpc: "2.0", id, result: Some(result), error: None }
}

fn err(id: Option<Value>, code: i32, message: String, data: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse { jsonrpc: "2.0", id, result: None, error: Some(JsonRpcError { code, message, data }) }
}

fn snapshot_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "when_local": { "type": "string", "description": "Local datetime, e.g. 2026-04-30T12:00:00" },
            "timezone": { "type": "string", "description": "IANA timezone" },
            "latitude": { "type": "number" },
            "longitude": { "type": "number" },
            "ayanamsha": { "type": "string", "enum": ["lahiri", "lahiri_alt_stub", "raman"] },
            "engine": { "type": "string", "enum": ["meeus", "surya_mean"] }
        },
        "required": ["when_local", "timezone", "latitude", "longitude"]
    })
}

fn civil_day_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "date": { "type": "string", "description": "YYYY-MM-DD" },
            "timezone": { "type": "string" },
            "latitude": { "type": "number" },
            "longitude": { "type": "number" },
            "ayanamsha": { "type": "string", "enum": ["lahiri", "lahiri_alt_stub", "raman"] },
            "engine": { "type": "string", "enum": ["meeus", "surya_mean"] }
        },
        "required": ["date", "timezone", "latitude", "longitude"]
    })
}

fn muhurta_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "date_start": { "type": "string" },
            "date_end": { "type": "string" },
            "timezone": { "type": "string" },
            "latitude": { "type": "number" },
            "longitude": { "type": "number" },
            "purpose_preset": { "type": "string" },
            "min_duration_minutes": { "type": "integer" },
            "ayanamsha": { "type": "string", "enum": ["lahiri", "lahiri_alt_stub", "raman"] },
            "engine": { "type": "string", "enum": ["meeus", "surya_mean"] }
        },
        "required": ["date_start", "date_end", "timezone", "latitude", "longitude"]
    })
}
