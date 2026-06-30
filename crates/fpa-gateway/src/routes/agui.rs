//! AG-UI surface — Server-Sent Events stream of agent → UI events.
//!
//! Streams [`fpa_protocol::AgUiEvent`] frames as SSE `data:` lines. This is the
//! surface `flint-gate` proxies, validates/filters, and meters mid-stream — so
//! frames must **never** be buffered: each event flushes as it is produced.
//!
//! The stub emits a `run_start` / `run_end` bracket; real runs splice the agent's
//! event stream (text deltas, tool calls, state snapshots) between them.

use axum::{
    Router,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use fpa_protocol::AgUiEvent;
use futures::stream::{self, Stream, StreamExt as _};
use std::{convert::Infallible, time::Duration};

/// Keep-alive ping interval for idle SSE connections.
const KEEPALIVE_SECS: u64 = 15;

/// Routes for the AG-UI surface.
pub fn router() -> Router {
    Router::new().route("/agui/stream", get(stream))
}

/// `GET /agui/stream` — open an AG-UI event stream.
///
/// Stub: emits a `run_start`/`run_end` bracket. The real handler subscribes to
/// the active agent run and forwards each [`AgUiEvent`] as it is produced.
async fn stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = [
        AgUiEvent::RunStart { model: None },
        AgUiEvent::RunEnd { stop_reason: None },
    ];

    let body = stream::iter(events).map(|event| {
        // `Event::default().json_data` is fallible only on serialize errors;
        // these payloads are infallible, so the stub maps via Event::default.
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_owned());
        Ok(Event::default().event(event_name(&event)).data(json))
    });

    Sse::new(body).keep_alive(KeepAlive::new().interval(Duration::from_secs(KEEPALIVE_SECS)))
}

/// SSE `event:` name for an AG-UI frame (the serde tag discriminant).
fn event_name(event: &AgUiEvent) -> &'static str {
    match event {
        AgUiEvent::RunStart { .. } => "run_start",
        AgUiEvent::TextMessageContent { .. } => "text_message_content",
        AgUiEvent::ToolCall { .. } => "tool_call",
        AgUiEvent::ToolResult { .. } => "tool_result",
        AgUiEvent::StateSnapshot { .. } => "state_snapshot",
        AgUiEvent::RunEnd { .. } => "run_end",
        AgUiEvent::Error { .. } => "error",
        // `AgUiEvent` is `#[non_exhaustive]`; future variants stream as "unknown"
        // until this gateway is taught to name them.
        _ => "unknown",
    }
}
