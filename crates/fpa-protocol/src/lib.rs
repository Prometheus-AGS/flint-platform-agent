//! `fpa-protocol` — wire-level payloads for the four agent surfaces this agent speaks:
//! AG-UI, A2A, A2UI, and MCP.
//!
//! House style for every payload (matching `frf-agentproto::ContentBlock`):
//! forward-compatible tagged enums (`#[serde(tag = "type", rename_all = "snake_case")]`)
//! with a `#[serde(other)] Unknown` catch-all so future variants never panic.
//!
//! A2UI types here are a **typed client view** of forge's A2UI registry
//! (`RFC-FORGE-A2UI-001`), which owns the canonical vocabulary — see `CLAUDE.md`
//! → "A2UI Ownership". A2A standard types are adopted from `a2a-protocol-types`
//! and re-exported via [`a2a_std`] so the rest of the workspace depends only on
//! `fpa-protocol`.

pub mod a2a;
pub mod a2a_std;
pub mod a2ui;
pub mod agui;
pub mod error;

pub use a2a::TaskEvent;
pub use a2a_std::{A2aTask, A2aTaskState, task_event_from_state};
pub use a2ui::Component;
pub use agui::AgUiEvent;
pub use error::ProtocolError;
