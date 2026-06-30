//! The A2A task runner use-case.
//!
//! Resolves a task `kind` to a fabric operation and executes it through the
//! relevant port(s). Holds ports as trait objects so the composition root can
//! inject any adapter (and tests can inject fakes).

use crate::error::AppError;
use fpa_domain::AdminTask;
use fpa_ports::{FabricClient, ForgeMetadata, GateAdmin, McpClient};
use std::sync::Arc;

/// Executes administrative tasks against the fabric through injected ports.
pub struct TaskRunner {
    pub forge: Arc<dyn ForgeMetadata>,
    pub fabric: Arc<dyn FabricClient>,
    pub gate: Arc<dyn GateAdmin>,
    pub mcp: Arc<dyn McpClient>,
}

impl TaskRunner {
    /// Construct a runner from the four plane ports.
    #[must_use]
    pub fn new(
        forge: Arc<dyn ForgeMetadata>,
        fabric: Arc<dyn FabricClient>,
        gate: Arc<dyn GateAdmin>,
        mcp: Arc<dyn McpClient>,
    ) -> Self {
        Self {
            forge,
            fabric,
            gate,
            mcp,
        }
    }

    /// Validate and execute an administrative task.
    #[tracing::instrument(skip(self), fields(task.kind = %task.kind))]
    pub async fn run(&self, task: &AdminTask) -> Result<serde_json::Value, AppError> {
        // Catalog dispatch lands here: match `task.kind` → port call.
        todo!("dispatch {task:?} to the appropriate port")
    }
}
