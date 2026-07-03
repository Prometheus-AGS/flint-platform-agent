//! In-memory [`ProjectStore`] adapter.
//!
//! Single-process, in-memory this phase (a durable/forge-backed store is a later
//! concern — YAGNI). Keyed by [`ProjectId`], guarded by an async `RwLock`, storing
//! the `Project` aggregate whole. Mirrors [`crate::task_store::TaskStore`].

use fpa_domain::{Project, ProjectId};
use fpa_ports::{PortError, ProjectStore};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// In-memory store of `Project` aggregates.
#[derive(Default)]
pub struct InMemoryProjectStore {
    projects: RwLock<HashMap<ProjectId, Project>>,
}

impl InMemoryProjectStore {
    /// Construct an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl ProjectStore for InMemoryProjectStore {
    async fn put(&self, project: &Project) -> Result<(), PortError> {
        self.projects
            .write()
            .await
            .insert(project.id, project.clone());
        Ok(())
    }

    async fn get(&self, id: &ProjectId) -> Result<Option<Project>, PortError> {
        Ok(self.projects.read().await.get(id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn project(id: u128, name: &str) -> Project {
        Project::new(ProjectId(Uuid::from_u128(id)), name)
    }

    #[tokio::test]
    async fn put_then_get_round_trips_the_aggregate() {
        let store = InMemoryProjectStore::new();
        let p = project(1, "alpha");
        store.put(&p).await.expect("put");
        let got = store
            .get(&ProjectId(Uuid::from_u128(1)))
            .await
            .expect("get")
            .expect("present");
        assert_eq!(got, p);
        assert_eq!(got.schema_version, fpa_domain::SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn get_unknown_is_none() {
        let store = InMemoryProjectStore::new();
        assert!(
            store
                .get(&ProjectId(Uuid::from_u128(99)))
                .await
                .expect("get")
                .is_none()
        );
    }

    #[tokio::test]
    async fn put_overwrites_same_id() {
        let store = InMemoryProjectStore::new();
        store.put(&project(2, "first")).await.expect("put");
        store.put(&project(2, "second")).await.expect("put");
        let got = store
            .get(&ProjectId(Uuid::from_u128(2)))
            .await
            .expect("get")
            .expect("present");
        assert_eq!(got.name, "second");
    }
}
