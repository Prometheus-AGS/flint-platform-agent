//! Application definitions within a project.
//!
//! An application is a collection of A2UI components plus modules and WASM plugin
//! references. Source is referenced, not embedded.

use crate::ids::ApplicationId;
use crate::project::component::ComponentRef;
use serde::{Deserialize, Serialize};

/// An application definition: composed UI + modules + plugins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ApplicationDef {
    pub id: ApplicationId,
    pub name: String,
    /// A2UI components composing this application (forge registry refs).
    #[serde(default)]
    pub components: Vec<ComponentRef>,
    /// Application modules (logical grouping; opaque metadata this phase).
    #[serde(default)]
    pub modules: Vec<ModuleRef>,
    /// WASM component/plugin references (resolved by the Kiln runtime).
    #[serde(default)]
    pub plugins: Vec<PluginRef>,
}

/// A reference to an application module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ModuleRef {
    pub name: String,
    /// Free-form module metadata (typed per module kind in a later phase).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A reference to a WASM component/plugin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PluginRef {
    pub name: String,
    /// Locator for the component (OCI/IPFS/registry — resolved by Kiln).
    pub locator: String,
    pub version: String,
}
