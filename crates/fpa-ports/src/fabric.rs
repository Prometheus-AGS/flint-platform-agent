//! Port: Flint Realtime Fabric.
//!
//! Realtime spine — subscribe to change/agent-event streams to surface live
//! fabric activity in AG-UI. Implemented by `fpa-fabric`.
//!
//! The subscribe stream carries the fabric's real wire type,
//! [`frf_domain::EventEnvelope`] (verified against
//! `flint-realtime-fabric/crates/frf-gateway/src/routes/subscribe.rs`, which
//! serializes `EventEnvelope` JSON frames on `/ws/v1/subscribe`). We reuse it
//! rather than defining an agent-local vocabulary (Base Rule 12).

use crate::error::PortError;
use async_trait::async_trait;
use futures::stream::BoxStream;
use uuid::Uuid;

/// The fabric's stamped event envelope + its payload-kind discriminant —
/// re-exported so the app layer and the adapter share exactly one type.
pub use frf_domain::{EventEnvelope, EventKind};

/// A stream of fabric events as they arrive over a subscription. Each item is a
/// decoded [`EventEnvelope`] or the [`PortError`] that ended the stream.
pub type EventStream = BoxStream<'static, Result<EventEnvelope, PortError>>;

/// Realtime fabric access (subscriptions, realtime operations).
#[async_trait]
pub trait FabricClient: Send + Sync {
    /// Report fabric liveness / connection health.
    async fn health(&self) -> Result<(), PortError>;

    /// Subscribe to a fabric channel's realtime event stream.
    ///
    /// Connects to the fabric gateway's `/ws/v1/subscribe` WebSocket for the
    /// given `channel` (the gateway requires a UUID), forwarding `bearer` as an
    /// `Authorization: Bearer` header. Yields each [`EventEnvelope`] as it
    /// arrives; transport/decode failures end the stream as [`PortError`].
    ///
    /// # Errors
    ///
    /// Returns a [`PortError`] if the initial connection/handshake fails (e.g.
    /// the fabric is unreachable, or the bearer is rejected with 401).
    async fn subscribe(
        &self,
        channel: Uuid,
        bearer: Option<&str>,
    ) -> Result<EventStream, PortError>;
}
