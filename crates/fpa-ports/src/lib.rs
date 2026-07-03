//! `fpa-ports` — Layer 1 trait seams (ports).
//!
//! Each external plane the agent administers is reduced to a port (async trait)
//! defined here. Adapters in `fpa-forge`, `fpa-fabric`, `fpa-gate`, and `fpa-mcp`
//! implement exactly one port each. **No implementations live here** — only the
//! contracts. The app layer depends on these traits, never on the adapters.

pub mod error;
pub mod fabric;
pub mod forge;
pub mod gate;
pub mod mcp;
pub mod project_store;

pub use error::PortError;
pub use fabric::FabricClient;
pub use forge::ForgeMetadata;
pub use gate::GateAdmin;
pub use mcp::McpClient;
pub use project_store::ProjectStore;
