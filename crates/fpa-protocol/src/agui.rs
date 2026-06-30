//! AG-UI event stream — agent → UI run lifecycle and content.
//!
//! Mirrors the `frf-agentproto::ContentBlock` shape. Compatible with what
//! `flint-gate` validates/filters and meters mid-stream (see gate README).

use serde::{Deserialize, Serialize};

/// A single event emitted on the AG-UI stream.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgUiEvent {
    /// Emitted at the start of an agent run.
    RunStart { model: Option<String> },

    /// Incremental text output from a streaming model response.
    TextMessageContent { delta: String },

    /// A tool invocation by the agent.
    ToolCall {
        tool_name: String,
        input: serde_json::Value,
    },

    /// The result returned from a tool execution.
    ToolResult {
        tool_name: String,
        output: serde_json::Value,
        is_error: bool,
    },

    /// Full agent state snapshot (for resumable sessions).
    StateSnapshot { state: serde_json::Value },

    /// Emitted when an agent run completes.
    RunEnd { stop_reason: Option<String> },

    /// An error emitted by the agent.
    Error {
        message: String,
        code: Option<String>,
    },

    /// Unrecognized or future variant — preserved without loss.
    #[serde(other)]
    Unknown,
}
