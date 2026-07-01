//! `fpa-forge` — adapter implementing [`fpa_ports::ForgeMetadata`].
//!
//! Talks to Flint Forge's Quarry gateway to read fabric metadata and data:
//! - **metadata** ← `GET {base}/openapi.json` (public, pre-compiled schema);
//! - **data** ← `POST {base}/graphql` forwarding the operator's gate bearer as
//!   `Authorization: Bearer` so forge applies RLS (`rls_from_bearer`).
//!
//! Forge is the source of truth for what entities exist. The adapter forwards the
//! operator's verified bearer; it never fabricates a credential or applies RLS
//! itself. Read-only this phase — no forge mutations.

use async_trait::async_trait;
use fpa_ports::{ForgeMetadata, PortError};
use serde_json::{Value, json};

/// HTTP client adapter for Flint Forge Quarry.
pub struct ForgeAdapter {
    /// Base URL of the Quarry gateway.
    pub base_url: String,
    http: reqwest::Client,
}

impl ForgeAdapter {
    /// Construct an adapter pointed at a Quarry gateway base URL.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Fetch the compiled OpenAPI document (public, no bearer).
    async fn openapi(&self) -> Result<Value, PortError> {
        let url = format!("{}/openapi.json", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(PortError::Downstream(format!(
                "forge /openapi.json returned {}",
                resp.status()
            )));
        }
        resp.json()
            .await
            .map_err(|e| PortError::Decode(e.to_string()))
    }

    /// Extract the list of table/entity names from an OpenAPI document.
    ///
    /// Quarry (Supabase-style) exposes one path per table; the schema/component
    /// names are the entity list. Falls back to the raw `components.schemas` keys.
    fn tables_from_openapi(doc: &Value) -> Value {
        let names: Vec<String> = doc
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(Value::as_object)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        json!({ "tables": names })
    }
}

#[async_trait]
impl ForgeMetadata for ForgeAdapter {
    async fn list_tables(&self, _bearer: Option<&str>) -> Result<Value, PortError> {
        // Metadata is public; no bearer required.
        let doc = self.openapi().await?;
        Ok(Self::tables_from_openapi(&doc))
    }

    async fn describe_table(&self, name: &str, _bearer: Option<&str>) -> Result<Value, PortError> {
        // Describe from the public OpenAPI component schema for `name`.
        let doc = self.openapi().await?;
        doc.get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(|s| s.get(name))
            .cloned()
            .ok_or_else(|| PortError::Downstream(format!("no such table: {name}")))
    }
}

/// POST a GraphQL query to forge under the operator's bearer (RLS applies).
///
/// Public helper for data reads: forwards `Authorization: Bearer <bearer>` so
/// forge's `rls_from_bearer` builds the RLS context. A missing bearer is a hard
/// error — the agent never queries data without the operator's identity.
///
/// # Errors
/// - [`PortError::Unauthorized`] on a missing bearer or a forge 401.
/// - [`PortError::Transport`] / [`PortError::Decode`] on transport/body failures.
pub async fn graphql_query(
    http: &reqwest::Client,
    base_url: &str,
    bearer: Option<&str>,
    query: &str,
    variables: Value,
) -> Result<Value, PortError> {
    let bearer = bearer.ok_or_else(|| {
        PortError::Unauthorized("forge data read requires the operator bearer".to_owned())
    })?;
    let url = format!("{}/graphql", base_url.trim_end_matches('/'));
    let body = json!({ "query": query, "variables": variables, "operationName": Value::Null });
    let resp = http
        .post(&url)
        .bearer_auth(bearer)
        .json(&body)
        .send()
        .await
        .map_err(|e| PortError::Transport(e.to_string()))?;
    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(PortError::Unauthorized(
            "forge rejected the bearer".to_owned(),
        ));
    }
    if !resp.status().is_success() {
        return Err(PortError::Downstream(format!(
            "forge /graphql returned {}",
            resp.status()
        )));
    }
    resp.json()
        .await
        .map_err(|e| PortError::Decode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn list_tables_parses_openapi() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "components": { "schemas": { "customers": {}, "orders": {} } }
            })))
            .mount(&server)
            .await;

        let adapter = ForgeAdapter::new(server.uri());
        let out = adapter.list_tables(None).await.expect("list");
        let tables = out["tables"].as_array().expect("array");
        assert_eq!(tables.len(), 2);
        assert!(tables.iter().any(|t| t == "customers"));
    }

    #[tokio::test]
    async fn describe_table_returns_component_schema() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "components": { "schemas": { "customers": { "type": "object" } } }
            })))
            .mount(&server)
            .await;

        let adapter = ForgeAdapter::new(server.uri());
        let schema = adapter
            .describe_table("customers", None)
            .await
            .expect("describe");
        assert_eq!(schema["type"], "object");
        assert!(adapter.describe_table("missing", None).await.is_err());
    }

    #[tokio::test]
    async fn graphql_forwards_bearer_and_returns_data() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(header("authorization", "Bearer tok-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"ok": true}})))
            .mount(&server)
            .await;

        let out = graphql_query(
            &reqwest::Client::new(),
            &server.uri(),
            Some("tok-123"),
            "{ __typename }",
            Value::Null,
        )
        .await
        .expect("graphql");
        assert_eq!(out["data"]["ok"], true);
    }

    #[tokio::test]
    async fn graphql_missing_bearer_is_unauthorized() {
        let err = graphql_query(
            &reqwest::Client::new(),
            "http://unused",
            None,
            "{ __typename }",
            Value::Null,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, PortError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn forge_401_maps_to_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let err = graphql_query(
            &reqwest::Client::new(),
            &server.uri(),
            Some("bad"),
            "{ x }",
            Value::Null,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, PortError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn forge_unreachable_is_transport_error() {
        // Nothing listening on this port.
        let adapter = ForgeAdapter::new("http://127.0.0.1:1");
        let err = adapter.list_tables(None).await.unwrap_err();
        assert!(matches!(err, PortError::Transport(_)));
    }
}
