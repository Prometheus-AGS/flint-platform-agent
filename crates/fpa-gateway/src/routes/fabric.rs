//! Fabric realtime surface — bridges a fabric subscription to Server-Sent Events.
//!
//! `GET /fabric/subscribe?channel=<uuid>` opens a [`fpa_ports::FabricClient`]
//! subscription to the fabric spine and forwards each [`fpa_ports::EventEnvelope`]
//! as an SSE `data:` frame. This is the inbound bridge deferred by the p10-c004
//! subscribe client — the agent now *surfaces* fabric change events over HTTP so
//! a host (or the smoke) can observe write → fabric → agent end-to-end.
//!
//! Like AG-UI, frames must never be buffered: each envelope flushes as it arrives.

use crate::{api_error::ApiError, identity::OperatorContext, state::AppState};
use axum::{
    Router,
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use futures::stream::{Stream, StreamExt as _};
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc, time::Duration};
use uuid::Uuid;

/// Keep-alive ping interval for idle SSE connections.
const KEEPALIVE_SECS: u64 = 15;

/// Query parameters for a fabric subscription.
#[derive(Debug, Deserialize)]
struct SubscribeParams {
    /// Fabric channel UUID to subscribe to (the fabric gateway requires a UUID).
    channel: Uuid,
}

/// Routes for the fabric realtime surface.
pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/fabric/subscribe", get(subscribe))
}

/// `GET /fabric/subscribe?channel=<uuid>` — stream fabric events as SSE.
///
/// Requires an authenticated operator. The bearer forwarded to the fabric gateway
/// (which enforces its own subscribe authz) is the configured `FPA_FABRIC_BEARER`
/// when set — for deployments where fabric verifies against a different IdP than
/// the one that authenticated the operator — otherwise the operator's own bearer.
/// Each `EventEnvelope` is emitted as an SSE frame named by its `kind`. A
/// subscription that fails to connect returns an error before the stream opens.
async fn subscribe(
    operator: OperatorContext,
    State(state): State<Arc<AppState>>,
    Query(params): Query<SubscribeParams>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let fabric_bearer = state
        .config
        .fabric_bearer
        .as_deref()
        .unwrap_or(&operator.bearer);
    let stream = state
        .runner
        .fabric
        .subscribe(params.channel, Some(fabric_bearer))
        .await
        .map_err(|e| map_port_error(&e))?;

    // Map each decoded envelope to an SSE frame. A per-item stream error becomes a
    // terminal `error` frame carrying the message, then the stream ends.
    let body = stream.map(|item| {
        let event = match item {
            Ok(envelope) => {
                let name = event_kind_name(&envelope);
                let json = serde_json::to_string(&envelope)
                    .unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_owned());
                Event::default().event(name).data(json)
            }
            Err(e) => Event::default()
                .event("error")
                .data(format!("{{\"error\":{:?}}}", e.to_string())),
        };
        Ok(event)
    });

    Ok(Sse::new(body).keep_alive(KeepAlive::new().interval(Duration::from_secs(KEEPALIVE_SECS))))
}

/// SSE `event:` name for a fabric envelope — its `EventKind` discriminant.
fn event_kind_name(envelope: &fpa_ports::EventEnvelope) -> &'static str {
    use fpa_ports::EventKind;
    match envelope.kind {
        EventKind::EntityChange => "entity_change",
        EventKind::AgentEvent => "agent_event",
        EventKind::SyncOp => "sync_op",
        EventKind::Presence => "presence",
        EventKind::Signal => "signal",
        EventKind::Custom(_) => "custom",
        _ => "unknown",
    }
}

/// Map a fabric [`fpa_ports::PortError`] to an [`ApiError`] with a client-safe
/// message (never leaks internal detail). A rejected bearer → 403; an
/// unreachable/failed connect → 502.
fn map_port_error(err: &fpa_ports::PortError) -> ApiError {
    use fpa_ports::PortError;
    match err {
        PortError::Unauthorized(_) => ApiError::forbidden("fabric rejected the subscription"),
        _ => ApiError::bad_gateway("fabric subscribe failed"),
    }
}
