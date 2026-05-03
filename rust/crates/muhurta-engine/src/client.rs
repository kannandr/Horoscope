//! Client abstraction over the Panchang calculation core.
//!
//! `muhurta-engine` deliberately does **not** talk to `panchang-core` directly
//! during search. It depends on this trait, which has two implementations:
//!
//! 1. [`McpPanchangClient`] — production: calls the **panchang-mcp**
//!    JSON-RPC server. This is the wire boundary the platform docs talk
//!    about — the muhurta usage app is a real MCP consumer, exactly like
//!    a future learned model would be.
//! 2. [`InProcessPanchangClient`] — development / tests: invokes
//!    `panchang_core::*` functions directly, no network. Convenient for
//!    unit tests and `cargo run` without a running MCP service.
//!
//! Both paths return identical `panchang_core::*` response shapes, so the
//! scoring code does not care which one it is wired to.

use async_trait::async_trait;
use panchang_core::{
    civil_day, panchang_day, snapshot, CivilDayRequest, CivilDayResponse, PanchangDayRequest,
    PanchangDayResponse, SnapshotRequest, SnapshotResponse,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::types::EngineError;

/// Minimum data surface `muhurta-engine` needs from the Panchang core.
/// Add new methods here only when scoring genuinely needs them — the
/// surface area is part of the contract with `panchang-mcp`.
#[async_trait]
pub trait PanchangClient: Send + Sync {
    async fn snapshot(&self, req: SnapshotRequest) -> Result<SnapshotResponse, EngineError>;
    async fn panchang_day(
        &self,
        req: PanchangDayRequest,
    ) -> Result<PanchangDayResponse, EngineError>;
    async fn civil_day(&self, req: CivilDayRequest) -> Result<CivilDayResponse, EngineError>;
}

/// Calls the panchang-mcp JSON-RPC server over HTTPS. This is the
/// production wire path.
#[derive(Debug, Clone)]
pub struct McpPanchangClient {
    http: HttpClient,
    endpoint: String,
    shared_secret: Option<String>,
}

impl McpPanchangClient {
    /// `base_url` should be the MCP service root (e.g.
    /// `https://panchang-stg-mcp.example.com`). The `/mcp` suffix is
    /// appended automatically.
    pub fn new(base_url: impl Into<String>, shared_secret: Option<String>) -> Self {
        let mut endpoint = base_url.into();
        // Tolerate trailing slashes in env vars.
        while endpoint.ends_with('/') {
            endpoint.pop();
        }
        endpoint.push_str("/mcp");
        Self {
            http: HttpClient::new(),
            endpoint,
            shared_secret: shared_secret.filter(|s| !s.trim().is_empty()),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn call_tool<R: for<'de> Deserialize<'de>>(
        &self,
        tool: &str,
        arguments: Value,
    ) -> Result<R, EngineError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": tool, "arguments": arguments }
        });
        let mut req = self
            .http
            .post(&self.endpoint)
            .header("content-type", "application/json")
            .json(&body);
        if let Some(secret) = &self.shared_secret {
            req = req.header("authorization", format!("Bearer {}", secret));
        }
        let resp = req.send().await.map_err(|e| EngineError::Transport {
            tool: tool.to_string(),
            cause: e.to_string(),
        })?;
        let status = resp.status();
        let envelope: Value = resp.json().await.map_err(|e| EngineError::Transport {
            tool: tool.to_string(),
            cause: format!("read body: {e}"),
        })?;
        if let Some(err) = envelope.get("error") {
            return Err(EngineError::McpRpc {
                tool: tool.to_string(),
                status: status.as_u16(),
                message: err
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown MCP error")
                    .to_string(),
                data: err.get("data").cloned(),
            });
        }
        let structured = envelope
            .get("result")
            .and_then(|r| r.get("structuredContent"))
            .ok_or_else(|| EngineError::Decode {
                tool: tool.to_string(),
                detail: "MCP result missing structuredContent".to_string(),
            })?
            .clone();
        serde_json::from_value::<R>(structured).map_err(|e| EngineError::Decode {
            tool: tool.to_string(),
            detail: e.to_string(),
        })
    }
}

#[async_trait]
impl PanchangClient for McpPanchangClient {
    async fn snapshot(&self, req: SnapshotRequest) -> Result<SnapshotResponse, EngineError> {
        let arguments = serde_json::to_value(&req).map_err(|e| EngineError::Decode {
            tool: "calculate_panchang_snapshot".to_string(),
            detail: format!("encode request: {e}"),
        })?;
        self.call_tool("calculate_panchang_snapshot", arguments).await
    }

    async fn panchang_day(
        &self,
        req: PanchangDayRequest,
    ) -> Result<PanchangDayResponse, EngineError> {
        let arguments = serde_json::to_value(&req).map_err(|e| EngineError::Decode {
            tool: "calculate_panchang_day".to_string(),
            detail: format!("encode request: {e}"),
        })?;
        self.call_tool("calculate_panchang_day", arguments).await
    }

    async fn civil_day(&self, req: CivilDayRequest) -> Result<CivilDayResponse, EngineError> {
        let arguments = serde_json::to_value(&req).map_err(|e| EngineError::Decode {
            tool: "list_civil_day_segments".to_string(),
            detail: format!("encode request: {e}"),
        })?;
        self.call_tool("list_civil_day_segments", arguments).await
    }
}

/// Direct in-process implementation. Useful for tests and for `cargo run`
/// without a separate MCP container. Production traffic should go through
/// [`McpPanchangClient`] — that is what makes the boundary real.
#[derive(Debug, Default, Clone, Copy)]
pub struct InProcessPanchangClient;

#[async_trait]
impl PanchangClient for InProcessPanchangClient {
    async fn snapshot(&self, req: SnapshotRequest) -> Result<SnapshotResponse, EngineError> {
        snapshot(req).map_err(EngineError::from)
    }

    async fn panchang_day(
        &self,
        req: PanchangDayRequest,
    ) -> Result<PanchangDayResponse, EngineError> {
        panchang_day(req).map_err(EngineError::from)
    }

    async fn civil_day(&self, req: CivilDayRequest) -> Result<CivilDayResponse, EngineError> {
        civil_day(req).map_err(EngineError::from)
    }
}
