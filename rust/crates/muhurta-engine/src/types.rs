use panchang_core::{AyanamshaId, EngineId, PanchangError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MuhurtaSearchRequest {
    pub date_start: String,
    pub date_end: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub purpose_preset: Option<String>,
    pub min_duration_minutes: Option<u32>,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MuhurtaWindow {
    pub start_local: String,
    pub end_local: String,
    pub duration_minutes: u32,
    pub score: i32,
    pub label: String,
    pub reasons: Vec<String>,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MuhurtaSearchResponse {
    pub preset: String,
    pub timezone: String,
    pub windows: Vec<MuhurtaWindow>,
}

/// Errors the muhurta engine can produce. Distinguishes calculation-level
/// problems (bad input → 400) from transport / decode problems hitting MCP
/// (wire failure → 502 / 504).
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("calculation: {0}")]
    Calculation(#[from] PanchangError),
    #[error("MCP transport for {tool}: {cause}")]
    Transport { tool: String, cause: String },
    #[error("MCP returned error from {tool} (HTTP {status}): {message}")]
    McpRpc {
        tool: String,
        status: u16,
        message: String,
        data: Option<serde_json::Value>,
    },
    #[error("MCP response decode for {tool}: {detail}")]
    Decode { tool: String, detail: String },
}

impl EngineError {
    /// HTTP status muhurta-api should map this to.
    pub fn http_status(&self) -> u16 {
        match self {
            EngineError::Calculation(PanchangError::InvalidTimezone(_))
            | EngineError::Calculation(PanchangError::InvalidDateTime(_))
            | EngineError::Calculation(PanchangError::InvalidDate(_))
            | EngineError::Calculation(PanchangError::InvalidCoordinates) => 400,
            EngineError::Calculation(PanchangError::Calculation(_)) => 422,
            EngineError::Transport { .. } => 502,
            // MCP returns JSON-RPC errors in-body with HTTP 200; treat those as 502.
            // Propagate real HTTP 401/403 so callers can prompt for MCP_SHARED_SECRET.
            EngineError::McpRpc { status: http_s, .. } => match *http_s {
                401 | 403 => *http_s,
                200 | 204 => 502,
                s if (400..500).contains(&s) => s,
                _ => 502,
            },
            EngineError::Decode { .. } => 502,
        }
    }
}

#[cfg(test)]
mod http_status_tests {
    use super::{EngineError, PanchangError};

    #[test]
    fn mcp_rpc_json_over_http_200_maps_to_502() {
        assert_eq!(
            EngineError::McpRpc {
                tool: "t".into(),
                status: 200,
                message: "x".into(),
                data: None,
            }
            .http_status(),
            502
        );
    }

    #[test]
    fn mcp_rpc_http_401_propagates() {
        assert_eq!(
            EngineError::McpRpc {
                tool: "t".into(),
                status: 401,
                message: "unauthorized".into(),
                data: None,
            }
            .http_status(),
            401
        );
    }

    #[test]
    fn calculation_invalid_date_is_400() {
        assert_eq!(
            EngineError::Calculation(PanchangError::InvalidDate("x".into())).http_status(),
            400
        );
    }
}
