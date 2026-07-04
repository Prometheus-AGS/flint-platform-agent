//! Port: persistence for the `Project` aggregate.
//!
//! The `Project` is the hub artifact this agent administers. It is a rich nested
//! aggregate (applications, sub-agents, schemas, realtime params, entity-meta) that
//! forge has no table for — so the agent owns its persistence behind this port
//! (in-memory now, durable later), mirroring the `TaskStore` precedent.
//!
//! The aggregate is stored **whole** — never decomposed into flat rows.

use crate::error::PortError;
use async_trait::async_trait;
use fpa_domain::{Project, ProjectId};

/// Agent-owned persistence of the `Project` aggregate.
///
/// Implementations MUST round-trip the aggregate losslessly (including
/// `schema_version` and every nested collection). A durable backend can replace
/// the interim in-memory adapter without touching call sites — the swap is a
/// composition-root change only.
#[async_trait]
pub trait ProjectStore: Send + Sync {
    /// Persist `project`, keyed by its [`ProjectId`]. Overwrites any prior value
    /// for the same id.
    async fn put(&self, project: &Project) -> Result<(), PortError>;

    /// Fetch the project with `id`, or `Ok(None)` if none is stored.
    async fn get(&self, id: &ProjectId) -> Result<Option<Project>, PortError>;

    /// Return every stored project aggregate. An empty store yields `Ok(vec![])`.
    /// Ordering is unspecified at the port; callers that need determinism sort.
    async fn list(&self) -> Result<Vec<Project>, PortError>;
}
