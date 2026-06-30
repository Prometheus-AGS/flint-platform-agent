//! `fpa-domain` — Layer 0 of the Flint Platform Agent.
//!
//! Pure domain types with serde only and **zero infrastructure dependencies**.
//! Nothing here may import a port, an adapter, or the interface crates.
//!
//! See `CLAUDE.md` → "Architecture: The Absolute Dependency Rule".

pub mod ids;
pub mod project;
pub mod task;

pub use ids::{ApplicationId, OperatorId, ProjectId, SessionId, SubAgentId, TaskId};
pub use project::{Project, SCHEMA_VERSION};
pub use task::{AdminTask, TaskState};
