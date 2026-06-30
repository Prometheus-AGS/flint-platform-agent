//! Database/schema definitions and realtime parameters within a project.
//!
//! Schema definitions carry A2UI generation hints (forge's metadata-driven UI
//! model) so dynamic forms/dashboards can be constructed from the schema.

use serde::{Deserialize, Serialize};

/// A database/table schema definition, with A2UI generation hints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SchemaDef {
    /// Logical schema/table name.
    pub name: String,
    /// A2UI generation hints (field types, display formats, component hints).
    ///
    /// Opaque `Value` here — the concrete hint vocabulary is owned by forge's
    /// `flint_meta` / `RFC-FORGE-A2UI-001`; this carries it without re-defining it.
    #[serde(default)]
    pub ui_hints: serde_json::Value,
}

/// Realtime parameters for a project (subscription/CDC configuration).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RealtimeParams {
    /// Whether realtime sync is enabled for the project.
    #[serde(default)]
    pub enabled: bool,
    /// Opaque realtime configuration (channels, CDC tables) — typed later.
    #[serde(default)]
    pub config: serde_json::Value,
}

/// An opaque reference into the `prometheus-entity-management` model.
///
/// That project is a TypeScript workspace, not a Rust crate, so this carries
/// only an identifier — no Rust dependency on it.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EntityMetaRef(pub String);
