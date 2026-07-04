//! `fpa-store-pg` — a durable [`ProjectStore`] adapter over Postgres.
//!
//! Agent-owned persistence for the `Project` aggregate (p6-c001). The Project has no
//! forge table, so the agent owns this store: a single `fpa_projects` table holding
//! the aggregate **whole** as a JSONB `body` (`id`/`name`/`schema_version` are
//! denormalized columns). Reads/writes go through a `deadpool-postgres` pool over
//! `tokio-postgres`.
//!
//! Hexagonal: this is an **adapter** crate — it implements the `fpa-ports` port and is
//! wired only by the composition root (`fpa-gateway`). `fpa-app` never depends on it.
//!
//! No `unwrap`/`expect` in this library; all failures map onto [`PortError`].

use async_trait::async_trait;
use deadpool_postgres::{Config, Pool, Runtime};
use fpa_domain::{Project, ProjectId};
use fpa_ports::{PortError, ProjectStore};
use tokio_postgres::NoTls;

/// The table DDL, applied idempotently at connect time.
const SCHEMA_SQL: &str = include_str!("../schema.sql");

/// Postgres-backed [`ProjectStore`].
pub struct PgProjectStore {
    pool: Pool,
}

impl PgProjectStore {
    /// Connect to Postgres at `db_url`, build a pool, and ensure the schema exists.
    ///
    /// # Errors
    /// [`PortError::Transport`] if the URL is unparseable, the pool cannot be built,
    /// or the initial connection / schema bootstrap fails.
    pub async fn connect(db_url: &str) -> Result<Self, PortError> {
        // Validate the URL up front so a malformed one surfaces a clear error
        // before pool construction. The parsed value is discarded — deadpool does
        // the authoritative parse below.
        db_url
            .parse::<tokio_postgres::Config>()
            .map_err(|e| PortError::Transport(format!("invalid FPA_PROJECT_DB_URL: {e}")))?;

        // Hand the whole URL to deadpool so it parses every field itself
        // (sslmode, options, connect_timeout, multi-host, …) — a manual field copy
        // would silently drop those (e.g. downgrade `sslmode=require` to no-TLS).
        let mut cfg = Config::new();
        cfg.url = Some(db_url.to_owned());

        let pool = cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| PortError::Transport(format!("failed to build pg pool: {e}")))?;

        let store = Self { pool };
        store.ensure_schema().await?;
        Ok(store)
    }

    /// Construct from an already-built pool (used by tests that own the pool).
    #[must_use]
    pub fn from_pool(pool: Pool) -> Self {
        Self { pool }
    }

    /// Apply the schema DDL (idempotent — `CREATE TABLE IF NOT EXISTS`).
    ///
    /// # Errors
    /// [`PortError::Transport`] on a pool/connection failure, [`PortError::Downstream`]
    /// if the DDL itself fails.
    pub async fn ensure_schema(&self) -> Result<(), PortError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| PortError::Transport(format!("pg pool get: {e}")))?;
        client
            .batch_execute(SCHEMA_SQL)
            .await
            .map_err(|e| PortError::Downstream(format!("schema bootstrap: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl ProjectStore for PgProjectStore {
    async fn put(&self, project: &Project) -> Result<(), PortError> {
        let body = serde_json::to_value(project)
            .map_err(|e| PortError::Decode(format!("serialize project: {e}")))?;
        let schema_version = i32::try_from(project.schema_version)
            .map_err(|e| PortError::Downstream(format!("schema_version out of range: {e}")))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| PortError::Transport(format!("pg pool get: {e}")))?;
        client
            .execute(
                "INSERT INTO fpa_projects (id, name, schema_version, body) \
                 VALUES ($1, $2, $3, $4) \
                 ON CONFLICT (id) DO UPDATE SET \
                   name = EXCLUDED.name, \
                   schema_version = EXCLUDED.schema_version, \
                   body = EXCLUDED.body, \
                   updated_at = now()",
                &[&project.id.0, &project.name, &schema_version, &body],
            )
            .await
            .map_err(|e| PortError::Downstream(format!("project upsert: {e}")))?;
        Ok(())
    }

    async fn get(&self, id: &ProjectId) -> Result<Option<Project>, PortError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| PortError::Transport(format!("pg pool get: {e}")))?;
        let row = client
            .query_opt("SELECT body FROM fpa_projects WHERE id = $1", &[&id.0])
            .await
            .map_err(|e| PortError::Downstream(format!("project select: {e}")))?;
        match row {
            None => Ok(None),
            Some(row) => {
                let body: serde_json::Value = row.get(0);
                let project = serde_json::from_value(body)
                    .map_err(|e| PortError::Decode(format!("deserialize project: {e}")))?;
                Ok(Some(project))
            }
        }
    }

    async fn list(&self) -> Result<Vec<Project>, PortError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| PortError::Transport(format!("pg pool get: {e}")))?;
        let rows = client
            .query("SELECT body FROM fpa_projects", &[])
            .await
            .map_err(|e| PortError::Downstream(format!("project list: {e}")))?;
        rows.into_iter()
            .map(|row| {
                let body: serde_json::Value = row.get(0);
                serde_json::from_value(body)
                    .map_err(|e| PortError::Decode(format!("deserialize project: {e}")))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fpa_domain::SCHEMA_VERSION;
    use uuid::Uuid;

    // --- Fast tests (no DB): the JSONB body mapping the adapter relies on. ---

    #[test]
    fn project_round_trips_through_jsonb_value() {
        // The adapter stores `serde_json::to_value(project)` and reads it back with
        // `from_value`; this proves that mapping is lossless before any DB is involved.
        let p = Project::new(ProjectId(Uuid::from_u128(0xABCD)), "durable-alpha");
        let body = serde_json::to_value(&p).expect("serialize");
        let back: Project = serde_json::from_value(body).expect("deserialize");
        assert_eq!(back, p);
        assert_eq!(back.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn schema_version_fits_i32() {
        let p = Project::new(ProjectId(Uuid::nil()), "x");
        assert!(i32::try_from(p.schema_version).is_ok());
    }

    // --- Integration test (needs Docker) — restart-survival. #[ignore]d by default. ---
    // Run explicitly with: cargo test -p fpa-store-pg -- --ignored
    // NOT run in environments without a reachable Docker daemon.
    #[tokio::test]
    #[ignore = "requires Docker (testcontainers Postgres); run with --ignored"]
    async fn put_survives_a_fresh_connection() {
        use testcontainers_modules::postgres::Postgres;
        use testcontainers_modules::testcontainers::runners::AsyncRunner;

        let node = Postgres::default()
            .start()
            .await
            .expect("start postgres container");
        let port = node.get_host_port_ipv4(5432).await.expect("container port");
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

        let id = ProjectId(Uuid::from_u128(0x1234));
        let project = Project::new(id, "survives-restart");

        // Write through one store instance...
        {
            let store = PgProjectStore::connect(&url).await.expect("connect 1");
            store.put(&project).await.expect("put");
        }
        // ...then read through a FRESH store instance (new pool) — proves durability.
        {
            let store = PgProjectStore::connect(&url).await.expect("connect 2");
            let got = store.get(&id).await.expect("get").expect("present");
            assert_eq!(got, project);
            // p8-c001: `list()` returns the stored aggregate too.
            let all = store.list().await.expect("list");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0], project);
        }
    }
}
