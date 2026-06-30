//! `fpa-protocol` — wire-level payloads for the four agent surfaces this agent speaks:
//! AG-UI, A2A, A2UI, and MCP.
//!
//! House style for every payload (matching `frf-agentproto::ContentBlock`):
//! forward-compatible tagged enums (`#[serde(tag = "type", rename_all = "snake_case")]`)
//! with a `#[serde(other)] Unknown` catch-all so future variants never panic.
//!
//! The A2UI primitives defined here are the **platform-wide** component vocabulary
//! reused by every downstream Prometheus agent (see `CLAUDE.md` → A2UI Primitive Mandate).

pub mod a2a;
pub mod a2ui;
pub mod agui;
pub mod error;

pub use a2a::TaskEvent;
pub use a2ui::Component;
pub use agui::AgUiEvent;
pub use error::ProtocolError;
