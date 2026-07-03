//! `fpa-gate` — adapter implementing [`fpa_ports::GateAdmin`].
//!
//! Administers Flint Gate via its **admin** server (`:4457`). That port must
//! never be exposed publicly; administrative calls go over a trusted path only
//! (see `CLAUDE.md` → gate dual-server model).
//!
//! Path verified from gate source (p4-c001): gate's `admin_router` mounts routes
//! **bare** (`/health`, `/routes`, …) and `main.rs` serves it directly on the
//! admin listener with **no `/v1/admin` nest** — so the real path is `GET /routes`
//! on the admin base URL. (`flint-gate-client`'s `/v1/admin` prefix is stale and
//! is deliberately NOT used; it also drags a WS-heavy dep tree.)

use async_trait::async_trait;
use fpa_ports::{GateAdmin, PortError};

/// Admin-API client adapter for Flint Gate.
pub struct GateAdapter {
    /// Base URL of the gate **admin** server (private).
    pub admin_url: String,
    /// Optional bearer for the admin API (admin token).
    admin_token: Option<String>,
    http: reqwest::Client,
}

impl GateAdapter {
    /// Construct an adapter pointed at the gate admin base URL.
    #[must_use]
    pub fn new(admin_url: impl Into<String>) -> Self {
        Self {
            admin_url: admin_url.into(),
            admin_token: None,
            http: reqwest::Client::new(),
        }
    }

    /// Set the admin bearer token used for admin-API calls.
    #[must_use]
    pub fn with_admin_token(mut self, token: Option<String>) -> Self {
        self.admin_token = token.filter(|t| !t.trim().is_empty());
        self
    }
}

#[async_trait]
impl GateAdmin for GateAdapter {
    async fn list_routes(&self) -> Result<serde_json::Value, PortError> {
        // Bare `/routes` on the admin base URL (verified from gate source).
        let url = format!("{}/routes", self.admin_url.trim_end_matches('/'));
        let mut req = self.http.get(&url);
        if let Some(token) = &self.admin_token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;
        match resp.status() {
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => Err(
                PortError::Unauthorized("gate admin rejected the request".to_owned()),
            ),
            s if s.is_success() => resp
                .json()
                .await
                .map_err(|e| PortError::Decode(e.to_string())),
            s => Err(PortError::Downstream(format!("gate /routes returned {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_routes_returns_gate_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/routes"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "routes": [] })),
            )
            .mount(&server)
            .await;
        let out = GateAdapter::new(server.uri())
            .list_routes()
            .await
            .expect("routes");
        assert!(out.get("routes").is_some());
    }

    #[tokio::test]
    async fn unauthorized_maps_to_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/routes"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let err = GateAdapter::new(server.uri())
            .list_routes()
            .await
            .unwrap_err();
        assert!(matches!(err, PortError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn unreachable_is_transport() {
        let err = GateAdapter::new("http://127.0.0.1:1")
            .list_routes()
            .await
            .unwrap_err();
        assert!(matches!(err, PortError::Transport(_)));
    }
}
