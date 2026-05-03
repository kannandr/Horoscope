//! Muhurta (auspicious-time) usage app on top of the Panchang calculation
//! core.
//!
//! Architecture
//! ------------
//! `muhurta-engine` is **not** part of `panchang-core`. The core engine
//! answers "what is the Panchang at time T at place P?" and nothing else.
//! Muhurta is one *consumer* of those answers: it asks the core for
//! sunrise, angas, hora, and day periods, then applies a transparent rule
//! set (and, in future, a learned model) to score time windows.
//!
//! Phase 2 wire boundary
//! ---------------------
//! [`PanchangClient`] is the seam. The production binary
//! (`muhurta-api`) wires in [`McpPanchangClient`], which calls
//! `panchang-mcp` over JSON-RPC — exactly the surface a learned model
//! consumes. Tests and dev-mode `cargo run` can use
//! [`InProcessPanchangClient`] for a zero-network path.

mod client;
mod search;
mod types;

pub use crate::client::{InProcessPanchangClient, McpPanchangClient, PanchangClient};
pub use crate::search::search_muhurta;
pub use crate::types::{
    EngineError, MuhurtaSearchRequest, MuhurtaSearchResponse, MuhurtaWindow,
};

pub use panchang_core::{AyanamshaId, EngineId, ErrorResponse, PanchangError};
