//! The administrative task catalog.
//!
//! Maps each A2A task `kind` to the port it dispatches to and the permission it
//! requires. The catalog is the contract between the A2A/MCP surfaces and the
//! ports: surfaces submit a `kind` + input; the runner validates against the
//! catalog before any port call.

/// Which plane a task dispatches to.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPort {
    Forge,
    Fabric,
    Gate,
    Mcp,
}

/// A single catalog entry describing one administrative task kind.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    /// Canonical task kind, e.g. `"forge.table.list"`.
    pub kind: &'static str,
    /// Plane this task dispatches to.
    pub target: TargetPort,
    /// Permission/role required to run it (checked against the operator).
    pub required_role: &'static str,
    /// Human description.
    pub description: &'static str,
    /// JSON Schema (as raw JSON text) for this kind's input, parsed on demand.
    /// A `const` array can't hold a `serde_json::Value`, so the schema is stored
    /// as text and parsed by [`CatalogEntry::input_schema`].
    pub input_schema_json: &'static str,
}

impl CatalogEntry {
    /// Parse this entry's input JSON Schema.
    ///
    /// # Panics
    /// Never in practice — the catalog schemas are compile-time-authored literals;
    /// a malformed one is a programmer error caught by the catalog test.
    #[must_use]
    pub fn input_schema(&self) -> serde_json::Value {
        serde_json::from_str(self.input_schema_json)
            .expect("catalog input_schema_json must be valid JSON (checked by test)")
    }
}

/// Schema for a kind that takes no required input.
const SCHEMA_EMPTY: &str = r#"{"type":"object"}"#;
/// Schema requiring a `project_id` string.
const SCHEMA_PROJECT_ID: &str =
    r#"{"type":"object","required":["project_id"],"properties":{"project_id":{"type":"string"}}}"#;
/// Schema requiring a table `name` string.
const SCHEMA_TABLE_NAME: &str =
    r#"{"type":"object","required":["name"],"properties":{"name":{"type":"string"}}}"#;

/// The seeded administrative task catalog.
///
/// forge-backed entries may return `PortError::Downstream` until forge's gateway
/// ships; they are catalogued now so the surface contract is complete.
pub const CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        kind: "project.create",
        target: TargetPort::Forge,
        required_role: "operator",
        description: "Create a new project artifact.",
        input_schema_json: SCHEMA_EMPTY,
    },
    CatalogEntry {
        kind: "project.inspect",
        target: TargetPort::Forge,
        required_role: "viewer",
        description: "Inspect a project's definition.",
        input_schema_json: SCHEMA_PROJECT_ID,
    },
    CatalogEntry {
        kind: "project.list",
        target: TargetPort::Forge,
        required_role: "viewer",
        description: "List projects visible to the operator.",
        input_schema_json: SCHEMA_EMPTY,
    },
    CatalogEntry {
        kind: "application.define",
        target: TargetPort::Forge,
        required_role: "operator",
        description: "Define an application within a project.",
        input_schema_json: SCHEMA_EMPTY,
    },
    CatalogEntry {
        kind: "application.deploy",
        target: TargetPort::Gate,
        required_role: "admin",
        description: "Deploy an application (route/auth config via gate).",
        input_schema_json: SCHEMA_EMPTY,
    },
    CatalogEntry {
        kind: "forge.table.list",
        target: TargetPort::Forge,
        required_role: "viewer",
        description: "List fabric tables.",
        input_schema_json: SCHEMA_EMPTY,
    },
    CatalogEntry {
        kind: "forge.table.describe",
        target: TargetPort::Forge,
        required_role: "viewer",
        description: "Describe a single fabric table.",
        input_schema_json: SCHEMA_TABLE_NAME,
    },
    CatalogEntry {
        kind: "fabric.health",
        target: TargetPort::Fabric,
        required_role: "viewer",
        description: "Check realtime fabric liveness.",
        input_schema_json: SCHEMA_EMPTY,
    },
];

/// Look up a catalog entry by kind.
#[must_use]
pub fn lookup(kind: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.kind == kind)
}

/// All catalogued kinds (for MCP `tools/list` and discovery).
pub fn kinds() -> impl Iterator<Item = &'static str> {
    CATALOG.iter().map(|e| e.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_and_unknown() {
        assert!(lookup("project.list").is_some());
        assert!(lookup("does.not.exist").is_none());
    }

    #[test]
    fn kinds_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for k in kinds() {
            assert!(seen.insert(k), "duplicate kind {k}");
        }
    }

    #[test]
    fn every_entry_has_valid_input_schema() {
        // Guards the `input_schema()` expect() — every catalog schema must parse.
        for e in CATALOG {
            let schema = e.input_schema();
            assert!(
                schema.is_object(),
                "schema for {} must be an object",
                e.kind
            );
        }
    }
}
