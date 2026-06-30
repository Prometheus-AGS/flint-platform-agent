//! Typed identifier newtypes.
//!
//! Per `CLAUDE.md` quality gates, identifiers are `#[repr(transparent)]` newtype
//! wrappers rather than bare `Uuid`/`String`, so the type system prevents mixing
//! an operator id with a session id.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies an operator (the human administering the fabric).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperatorId(pub Uuid);

/// Identifies an agent session (one operator ↔ agent conversation/run context).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

/// Identifies an A2A administrative task.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TaskId(pub Uuid);

/// Identifies a project (the hub artifact administered by this agent).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProjectId(pub Uuid);

/// Identifies an application definition within a project.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ApplicationId(pub Uuid);

/// Identifies a sub-agent definition within a project.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SubAgentId(pub Uuid);
