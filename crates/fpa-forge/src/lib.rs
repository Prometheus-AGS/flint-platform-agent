//! `fpa-forge` — adapter implementing [`fpa_ports::ForgeMetadata`].
//!
//! Talks to Flint Forge's Quarry gateway to read fabric metadata and data:
//! - **metadata** ← `GET {base}/openapi.json` (public, pre-compiled schema);
//! - **data** ← `POST {base}/graphql` forwarding the operator's gate bearer as
//!   `Authorization: Bearer` so forge applies RLS (`rls_from_bearer`).
//!
//! Writes (`create_entity`) go through forge's **REST CRUD** surface. Forge's
//! reflection router is merged at the gateway **root** (no `/rest` prefix) and
//! generates one route per table as `POST /{schema}/{table}` (source-verified:
//! `fdb-reflection/compilers/rest/mod.rs` + `fdb-gateway/main.rs`). So the write
//! path is `POST {base}[{path_base}]/{schema}/{table}`, under the operator bearer.
//! `graphql_exec` remains available for GraphQL queries. Forge (RLS + Keto/Cedar)
//! is the authorization authority; the adapter forwards the bearer and never
//! fabricates a credential or replicates authz.

use async_trait::async_trait;
use fpa_ports::{ForgeMetadata, PortError};
use serde_json::{Value, json};

/// HTTP client adapter for Flint Forge Quarry.
pub struct ForgeAdapter {
    /// Base URL of the Quarry gateway.
    pub base_url: String,
    /// Optional path base prepended before `/{schema}/{table}`. Empty by default
    /// (forge merges the REST router at the gateway root). Config-driven
    /// (`FPA_FORGE_REST_PREFIX`) only for non-default gateway topologies — it does
    /// **not** default to `/rest`.
    path_base: String,
    http: reqwest::Client,
}

impl ForgeAdapter {
    /// Construct an adapter pointed at a Quarry gateway base URL (root-mounted REST).
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            path_base: String::new(),
            http: reqwest::Client::new(),
        }
    }

    /// Set an optional REST path base (from config, e.g. `FPA_FORGE_REST_PREFIX`).
    /// Prepended before `/{schema}/{table}`. Leave unset for a root-mounted forge.
    #[must_use]
    pub fn with_rest_prefix(mut self, prefix: impl Into<String>) -> Self {
        let p = prefix.into();
        let trimmed = p.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            trimmed.clone_into(&mut self.path_base);
        }
        self
    }

    /// POST a row to forge's REST table endpoint (`{base}[{path_base}]/{schema}/
    /// {table}`) under the operator bearer. Forge (RLS + Keto/Cedar) authorizes
    /// server-side; a missing bearer is rejected (writes require the operator).
    async fn rest_insert(
        &self,
        schema: &str,
        table: &str,
        object: Value,
        bearer: Option<&str>,
    ) -> Result<Value, PortError> {
        let bearer = bearer.ok_or_else(|| {
            PortError::Unauthorized("forge write requires the operator bearer".to_owned())
        })?;
        let url = format!(
            "{}{}/{}/{}",
            self.base_url.trim_end_matches('/'),
            self.path_base,
            schema,
            table
        );
        let resp = self
            .http
            .post(&url)
            .bearer_auth(bearer)
            .json(&object)
            .send()
            .await
            .map_err(|e| PortError::Transport(e.to_string()))?;
        match resp.status() {
            reqwest::StatusCode::UNAUTHORIZED => Err(PortError::Unauthorized(
                "forge rejected the bearer".to_owned(),
            )),
            reqwest::StatusCode::FORBIDDEN => Err(PortError::Unauthorized(
                "forge policy denied (Keto/Cedar)".to_owned(),
            )),
            s if s.is_success() => resp
                .json()
                .await
                // 201 with an empty/non-JSON body is still a success.
                .or_else(|_| Ok(serde_json::json!({ "created": true }))),
            s => Err(PortError::Downstream(format!(
                "forge REST insert returned {s}"
            ))),
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

    async fn create_entity(
        &self,
        schema: &str,
        table: &str,
        object: Value,
        bearer: Option<&str>,
    ) -> Result<Value, PortError> {
        // Primary write path: forge's REST insert `POST /{schema}/{table}` (root-
        // mounted, source-verified from forge). `graphql_exec` remains available
        // for queries; REST is the write path.
        self.rest_insert(schema, table, object, bearer).await
    }
}

/// POST a GraphQL **query** to forge under the operator's bearer (RLS applies).
///
/// Thin wrapper over [`graphql_exec`] for read queries.
///
/// # Errors
/// See [`graphql_exec`].
pub async fn graphql_query(
    http: &reqwest::Client,
    base_url: &str,
    bearer: Option<&str>,
    query: &str,
    variables: Value,
) -> Result<Value, PortError> {
    graphql_exec(http, base_url, bearer, query, variables).await
}

/// POST an arbitrary GraphQL operation (query or mutation) to forge's `/graphql`
/// under the operator's bearer. Forge applies RLS + (for mutations) Keto/Cedar;
/// this helper forwards the bearer and maps status codes onto [`PortError`].
///
/// A missing bearer is rejected — the agent never calls `/graphql` without the
/// operator's identity.
///
/// # Errors
/// - [`PortError::Unauthorized`] on a missing bearer, a forge **401** (bad token),
///   or a forge **403** (Keto/Cedar policy denial — distinguished in the message).
/// - [`PortError::Downstream`] on other non-2xx statuses.
/// - [`PortError::Transport`] / [`PortError::Decode`] on transport/body failures.
pub async fn graphql_exec(
    http: &reqwest::Client,
    base_url: &str,
    bearer: Option<&str>,
    operation: &str,
    variables: Value,
) -> Result<Value, PortError> {
    let bearer = bearer.ok_or_else(|| {
        PortError::Unauthorized("forge /graphql requires the operator bearer".to_owned())
    })?;
    let url = format!("{}/graphql", base_url.trim_end_matches('/'));
    let body = json!({ "query": operation, "variables": variables, "operationName": Value::Null });
    let resp = http
        .post(&url)
        .bearer_auth(bearer)
        .json(&body)
        .send()
        .await
        .map_err(|e| PortError::Transport(e.to_string()))?;
    match resp.status() {
        reqwest::StatusCode::UNAUTHORIZED => {
            return Err(PortError::Unauthorized(
                "forge rejected the bearer".to_owned(),
            ));
        }
        reqwest::StatusCode::FORBIDDEN => {
            return Err(PortError::Unauthorized(
                "forge policy denied (Keto/Cedar)".to_owned(),
            ));
        }
        s if !s.is_success() => {
            return Err(PortError::Downstream(format!(
                "forge /graphql returned {s}"
            )));
        }
        _ => {}
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

    #[tokio::test]
    async fn create_entity_posts_schema_qualified_rest_insert_with_bearer() {
        // p5-c002: writes go to forge's real REST route `POST /{schema}/{table}`
        // (root-mounted, no /rest prefix). 201 Created with the inserted row.
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/flint_meta/applications"))
            .and(header("authorization", "Bearer tok"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "id": "a-1" })))
            .mount(&server)
            .await;

        let adapter = ForgeAdapter::new(server.uri());
        let out = adapter
            .create_entity(
                "flint_meta",
                "applications",
                json!({"name": "p1"}),
                Some("tok"),
            )
            .await
            .expect("create");
        assert_eq!(out["id"], "a-1");
    }

    #[tokio::test]
    async fn create_entity_has_no_rest_prefix_by_default() {
        // Guard against regressing to the old `/rest/<table>` path: a mock mounted
        // at `/rest/...` must NOT match; only the schema/table path is used.
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/flint_a2ui/components"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "ok": true })))
            .mount(&server)
            .await;
        let adapter = ForgeAdapter::new(server.uri());
        adapter
            .create_entity("flint_a2ui", "components", json!({}), Some("tok"))
            .await
            .expect("create at root-mounted schema/table");
    }

    #[tokio::test]
    async fn create_entity_honors_optional_path_base() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/flint_meta/applications"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "ok": true })))
            .mount(&server)
            .await;
        let adapter = ForgeAdapter::new(server.uri()).with_rest_prefix("/api");
        adapter
            .create_entity("flint_meta", "applications", json!({}), Some("tok"))
            .await
            .expect("create with path base");
    }

    #[tokio::test]
    async fn create_entity_forge_403_is_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/flint_meta/applications"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;
        let adapter = ForgeAdapter::new(server.uri());
        let err = adapter
            .create_entity("flint_meta", "applications", json!({}), Some("tok"))
            .await
            .unwrap_err();
        assert!(matches!(err, PortError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn create_entity_missing_bearer_is_unauthorized() {
        let adapter = ForgeAdapter::new("http://unused");
        let err = adapter
            .create_entity("flint_meta", "applications", json!({}), None)
            .await
            .unwrap_err();
        assert!(matches!(err, PortError::Unauthorized(_)));
    }
}
