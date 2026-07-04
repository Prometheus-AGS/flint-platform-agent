//! `fpa-fabric` — adapter implementing [`fpa_ports::FabricClient`].
//!
//! Connects to the Flint Realtime Fabric gateway. Liveness is a public
//! `GET /healthz`. Realtime change events are consumed over the
//! `/ws/v1/subscribe` WebSocket, which serializes [`fpa_ports::EventEnvelope`]
//! JSON frames (WAL bypasses RLS, so the fabric re-queries each row as the
//! subscriber before delivery — see `CLAUDE.md` cross-plane contracts; the
//! agent forwards the operator's bearer and does not assume pre-authorization).

use async_trait::async_trait;
use fpa_ports::{EventEnvelope, EventStream, FabricClient, PortError};
use futures::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{ClientRequestBuilder, Message, http::Uri};
use uuid::Uuid;

/// Client adapter for the Flint Realtime Fabric gateway.
pub struct FabricAdapter {
    /// Endpoint of the fabric gateway.
    pub endpoint: String,
    http: reqwest::Client,
}

impl FabricAdapter {
    /// Construct an adapter pointed at a fabric gateway endpoint.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Build the `ws(s)://…/ws/v1/subscribe?channel=<uuid>` handshake URI from
    /// the configured (`http(s)://`) endpoint. `http` → `ws`, `https` → `wss`;
    /// any other scheme (or a bare host) is treated as plain `ws`.
    fn subscribe_uri(&self, channel: Uuid) -> Result<Uri, PortError> {
        let base = self.endpoint.trim_end_matches('/');
        let ws_base = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{rest}")
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{rest}")
        } else if base.starts_with("ws://") || base.starts_with("wss://") {
            base.to_owned()
        } else {
            format!("ws://{base}")
        };
        format!("{ws_base}/ws/v1/subscribe?channel={channel}")
            .parse::<Uri>()
            .map_err(|e| PortError::Transport(format!("invalid fabric ws uri: {e}")))
    }
}

#[async_trait]
impl FabricClient for FabricAdapter {
    async fn health(&self) -> Result<(), PortError> {
        let url = format!("{}/healthz", self.endpoint.trim_end_matches('/'));
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(PortError::Downstream(format!(
                "fabric /healthz returned {}",
                resp.status()
            )))
        }
    }

    async fn subscribe(
        &self,
        channel: Uuid,
        bearer: Option<&str>,
    ) -> Result<EventStream, PortError> {
        let uri = self.subscribe_uri(channel)?;
        let mut request = ClientRequestBuilder::new(uri);
        if let Some(token) = bearer {
            request = request.with_header("Authorization", format!("Bearer {token}"));
        }

        let (ws, _resp) = connect_async(request)
            .await
            .map_err(|e| PortError::Transport(format!("fabric subscribe connect: {e}")))?;

        // Map the raw WS message stream into a stream of decoded envelopes.
        // Text frames decode to `EventEnvelope`; a decode failure ends the
        // stream with `Decode`; a transport error ends it with `Transport`;
        // Close / non-data frames terminate the stream cleanly (None).
        let stream = ws.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(text)) => Some(
                    serde_json::from_str::<EventEnvelope>(&text)
                        .map_err(|e| PortError::Decode(format!("fabric envelope: {e}"))),
                ),
                Ok(Message::Binary(bytes)) => Some(
                    serde_json::from_slice::<EventEnvelope>(&bytes)
                        .map_err(|e| PortError::Decode(format!("fabric envelope: {e}"))),
                ),
                // Close / Ping / Pong / other non-data frames carry no envelope
                // for us. `filter_map` skips a `None` and keeps polling; the
                // stream ends only when the underlying socket ends.
                Ok(_) => None,
                Err(e) => Some(Err(PortError::Transport(format!("fabric subscribe: {e}")))),
            }
        });

        Ok(stream.boxed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn healthy_returns_ok() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/healthz"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "status": "ok", "version": "1.0" })),
            )
            .mount(&server)
            .await;
        FabricAdapter::new(server.uri())
            .health()
            .await
            .expect("healthy");
    }

    #[tokio::test]
    async fn unhealthy_is_downstream() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/healthz"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        let err = FabricAdapter::new(server.uri()).health().await.unwrap_err();
        assert!(matches!(err, PortError::Downstream(_)));
    }

    #[tokio::test]
    async fn unreachable_is_transport() {
        let err = FabricAdapter::new("http://127.0.0.1:1")
            .health()
            .await
            .unwrap_err();
        assert!(matches!(err, PortError::Transport(_)));
    }

    #[test]
    fn subscribe_uri_maps_scheme_and_appends_channel() {
        let channel = Uuid::nil();
        let adapter = FabricAdapter::new("http://fabric:8080");
        let uri = adapter.subscribe_uri(channel).expect("uri");
        assert_eq!(
            uri.to_string(),
            format!("ws://fabric:8080/ws/v1/subscribe?channel={channel}")
        );

        let secure = FabricAdapter::new("https://fabric:8080/");
        assert!(
            secure
                .subscribe_uri(channel)
                .expect("uri")
                .to_string()
                .starts_with("wss://fabric:8080/ws/v1/subscribe?channel=")
        );
    }

    #[tokio::test]
    async fn subscribe_unreachable_is_transport() {
        // `EventStream` is not `Debug`, so match rather than `unwrap_err`.
        match FabricAdapter::new("http://127.0.0.1:1")
            .subscribe(Uuid::nil(), Some("token"))
            .await
        {
            Err(PortError::Transport(_)) => {}
            Err(other) => panic!("expected Transport, got {other:?}"),
            Ok(_) => panic!("expected a connection error, got a stream"),
        }
    }

    #[tokio::test]
    async fn subscribe_decodes_event_envelope() {
        use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, TenantId};
        use futures::SinkExt;
        use tokio::net::TcpListener;
        use tokio_tungstenite::tungstenite::Message as WsMessage;

        // Build a real envelope the fake server will emit.
        let channel = Channel {
            id: ChannelId::new(),
            tenant_id: TenantId::new(),
            path: "entity/user/updates".to_owned(),
        };
        let envelope = EventEnvelope::new(
            channel,
            Offset(7),
            EventKind::EntityChange,
            serde_json::json!({ "op": "insert", "id": 42 }),
        );
        let expected_id = envelope.id;
        let frame = serde_json::to_string(&envelope).expect("serialize");

        // Fake fabric gateway: accept one WS upgrade, send one text frame, close.
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut ws = tokio_tungstenite::accept_async(stream)
                .await
                .expect("ws upgrade");
            ws.send(WsMessage::Text(frame.into()))
                .await
                .expect("send frame");
            ws.close(None).await.ok();
        });

        let adapter = FabricAdapter::new(format!("http://{addr}"));
        let mut stream = adapter
            .subscribe(Uuid::nil(), Some("test-token"))
            .await
            .expect("subscribe");

        let received = stream.next().await.expect("one item").expect("decoded");
        assert_eq!(received.id, expected_id);
        assert_eq!(received.kind, EventKind::EntityChange);
        assert_eq!(received.offset, Offset(7));

        server.await.expect("server task");
    }
}
