//! The `Project` artifact — the hub aggregate this agent administers.
//!
//! A project is a collection of A2UI component references, sub-agent definitions,
//! application definitions, database/schema definitions (with A2UI generation
//! hints), realtime parameters, and entity-management metadata references.
//!
//! Per Base Rule 39 the artifact is **typed, versioned, and host-portable**: it
//! carries a [`SCHEMA_VERSION`] and round-trips losslessly through serde. The
//! published JSON Schema is derived under the `schema` feature.

pub mod application;
pub mod component;
pub mod schema;
pub mod sub_agent;

use crate::ids::ProjectId;
use serde::{Deserialize, Serialize};

pub use application::{ApplicationDef, ModuleRef, PluginRef};
pub use component::{ComponentRef, RegistryComponentId};
pub use schema::{EntityMetaRef, RealtimeParams, SchemaDef};
pub use sub_agent::SubAgentDef;

/// Current `Project` artifact schema version.
///
/// Migration policy: **additive by default** — new optional fields and new
/// `#[non_exhaustive]` enum variants do not bump the major; removals/renames do.
pub const SCHEMA_VERSION: u32 = 1;

/// The Project aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Project {
    pub id: ProjectId,
    /// Artifact schema version (see [`SCHEMA_VERSION`]).
    pub schema_version: u32,
    pub name: String,
    /// Application definitions owned by the project.
    #[serde(default)]
    pub applications: Vec<ApplicationDef>,
    /// Sub-agents defined by the project.
    #[serde(default)]
    pub sub_agents: Vec<SubAgentDef>,
    /// Database/schema definitions (carry A2UI generation hints).
    #[serde(default)]
    pub schemas: Vec<SchemaDef>,
    /// Realtime configuration for the project.
    #[serde(default)]
    pub realtime: RealtimeParams,
    /// References into the entity-management model (opaque identifiers).
    #[serde(default)]
    pub entity_meta: Vec<EntityMetaRef>,
}

impl Default for RealtimeParams {
    fn default() -> Self {
        Self {
            enabled: false,
            config: serde_json::Value::Null,
        }
    }
}

impl Project {
    /// Create an empty project at the current schema version.
    #[must_use]
    pub fn new(id: ProjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            schema_version: SCHEMA_VERSION,
            name: name.into(),
            applications: Vec::new(),
            sub_agents: Vec::new(),
            schemas: Vec::new(),
            realtime: RealtimeParams::default(),
            entity_meta: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample() -> Project {
        let mut p = Project::new(ProjectId(Uuid::nil()), "demo");
        p.entity_meta.push(EntityMetaRef("customers".into()));
        p
    }

    #[test]
    fn round_trips_through_json() {
        let p = sample();
        let json = serde_json::to_string(&p).expect("serialize");
        let back: Project = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p, back);
    }

    #[test]
    fn new_sets_current_schema_version() {
        assert_eq!(
            Project::new(ProjectId(Uuid::nil()), "x").schema_version,
            SCHEMA_VERSION
        );
    }

    #[test]
    fn defaults_let_minimal_json_parse() {
        // Only the required fields present; collections default to empty.
        let json = r#"{"id":"00000000-0000-0000-0000-000000000000","schema_version":1,"name":"m"}"#;
        let p: Project = serde_json::from_str(json).expect("minimal parse");
        assert!(p.applications.is_empty());
        assert!(!p.realtime.enabled);
    }

    #[cfg(feature = "schema")]
    #[test]
    fn checked_in_schema_matches_generated() {
        // The committed schema must equal the freshly generated one, so the
        // schema never drifts from the types. Regenerate with the gen-schema test
        // helper if this fails after an intentional type change.
        let generated = serde_json::to_value(schemars::schema_for!(Project)).expect("gen");
        let committed: serde_json::Value =
            serde_json::from_str(include_str!("../../schema/project.schema.json"))
                .expect("committed schema parses");
        assert_eq!(
            committed, generated,
            "project.schema.json is stale — regenerate it from the Project type"
        );
    }
}
