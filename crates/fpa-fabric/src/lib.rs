//! `fpa-fabric` — adapter implementing [`fpa_ports::FabricClient`].
//!
//! Connects to the Flint Realtime Fabric gateway. Liveness is a public
//! `GET /healthz`. (Realtime subscriptions over `/ws/v1/subscribe` are a later
//! phase — WAL bypasses RLS, so events are not assumed pre-authorized for an
//! arbitrary viewer; see `CLAUDE.md` cross-plane contracts.)

use async_trait::async_trait;
use fpa_ports::{FabricClient, PortError};

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
}
