//! Sub-agent definitions within a project.

use crate::ids::SubAgentId;
use serde::{Deserialize, Serialize};

/// A sub-agent belonging to a project (a downstream agent the project defines).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SubAgentDef {
    pub id: SubAgentId,
    pub name: String,
    /// The agent's role/purpose within the project.
    pub role: String,
    /// Opaque configuration (model routing, tools, etc.) — typed in a later phase.
    #[serde(default)]
    pub config: serde_json::Value,
}
