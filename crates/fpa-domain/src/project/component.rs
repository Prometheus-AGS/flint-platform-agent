//! A2UI component references.
//!
//! **Ownership:** the canonical A2UI component vocabulary is owned by
//! `flint-forge` (`RFC-FORGE-A2UI-001` — the Global A2UI Component Registry).
//! A `ComponentRef` therefore references a registry entry by id + version; it
//! does **not** embed component source and does **not** define an agent-local
//! component vocabulary. The referenced registry is resolved against forge when
//! that service ships.

use serde::{Deserialize, Serialize};

/// Identifier of a component in forge's A2UI registry.
///
/// Placeholder newtype to be resolved against `RFC-FORGE-A2UI-001` when the
/// forge registry is available; the shape (id + version) is intentionally stable.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RegistryComponentId(pub String);

/// A reference to an A2UI component held in the forge registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComponentRef {
    /// Registry component id (forge `RFC-FORGE-A2UI-001`).
    pub component: RegistryComponentId,
    /// Pinned registry version of the component.
    pub version: String,
}
