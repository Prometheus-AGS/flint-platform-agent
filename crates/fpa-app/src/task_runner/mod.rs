//! The A2A task runner use-case.
//!
//! Resolves a task `kind` to a fabric operation and executes it through the
//! relevant port(s), enforcing gate-derived permissions and emitting an audit
//! record. Holds ports as trait objects so the composition root can inject any
//! adapter (and tests can inject fakes).

use crate::auth::AuthContext;
use crate::catalog::{self, TargetPort};
use crate::error::AppError;
use fpa_domain::{AdminTask, ApplicationDef, Project, ProjectId};
use fpa_ports::{FabricClient, ForgeMetadata, GateAdmin, McpClient, ProjectStore};
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Executes administrative tasks against the fabric through injected ports.
pub struct TaskRunner {
    pub forge: Arc<dyn ForgeMetadata>,
    pub fabric: Arc<dyn FabricClient>,
    pub gate: Arc<dyn GateAdmin>,
    pub mcp: Arc<dyn McpClient>,
    /// Agent-owned persistence for the `Project` aggregate (p5-c001). The Project
    /// has no forge table; `project.create` writes here, not to forge.
    pub projects: Arc<dyn ProjectStore>,
}

impl TaskRunner {
    /// Construct a runner from the four plane ports plus the project store.
    #[must_use]
    pub fn new(
        forge: Arc<dyn ForgeMetadata>,
        fabric: Arc<dyn FabricClient>,
        gate: Arc<dyn GateAdmin>,
        mcp: Arc<dyn McpClient>,
        projects: Arc<dyn ProjectStore>,
    ) -> Self {
        Self {
            forge,
            fabric,
            gate,
            mcp,
            projects,
        }
    }

    /// Validate, authorize, and execute an administrative task.
    ///
    /// Pipeline: catalog lookup → permission check → port dispatch → audit.
    /// A permission denial returns [`AppError::Unauthorized`] and **never** calls
    /// a port.
    ///
    /// # Errors
    /// - [`AppError::UnknownTaskKind`] if `task.kind` is not catalogued.
    /// - [`AppError::Unauthorized`] if the operator lacks the required role.
    /// - [`AppError::Port`] if the downstream plane fails.
    // Skip `task` and `auth` from auto-capture (their Debug could include input
    // payloads / roles); record only the safe `kind` + subject explicitly.
    #[tracing::instrument(skip_all, fields(task.kind = %task.kind, operator = %auth.subject))]
    pub async fn run(
        &self,
        task: &AdminTask,
        auth: &AuthContext,
    ) -> Result<serde_json::Value, AppError> {
        let entry = catalog::lookup(&task.kind)
            .ok_or_else(|| AppError::UnknownTaskKind(task.kind.clone()))?;

        // Permission enforcement BEFORE any port call (Base Rule 33).
        if !auth.has_role(entry.required_role) {
            // Audit the denial; never log claims/secrets — only ids + decision.
            tracing::warn!(
                operator = %auth.subject,
                kind = entry.kind,
                required_role = entry.required_role,
                decision = "denied",
                "task authorization denied"
            );
            return Err(AppError::Unauthorized(format!(
                "task '{}' requires role '{}'",
                entry.kind, entry.required_role
            )));
        }

        // Validate input against the kind's JSON Schema before any port call.
        // Treat null/absent input as an empty object so no-argument calls pass an
        // object schema while required fields are still enforced when expected.
        let schema = entry.input_schema();
        let input = if task.input.is_null() {
            serde_json::json!({})
        } else {
            task.input.clone()
        };
        if let Err(err) = jsonschema::validate(&schema, &input) {
            return Err(AppError::InvalidInput(format!(
                "input for '{}' failed schema: {err}",
                entry.kind
            )));
        }

        // Audit the allow decision, recording identity provenance (p5-c003 G6):
        // whether the identity was signature-verified (direct token) or
        // gate-trusted (injected headers). Never log the token/claims.
        tracing::info!(
            operator = %auth.subject,
            kind = entry.kind,
            decision = "allowed",
            signature_verified = auth.signature_verified,
            "task dispatch"
        );

        // Dispatch to the mapped plane. Concrete per-kind argument mapping lands
        // as the ports gain real implementations; here we route by target so the
        // contract + permission gate are exercised end-to-end.
        let outcome = match entry.target {
            TargetPort::Forge => {
                self.dispatch_forge(entry.kind, &input, auth.bearer.as_deref())
                    .await
            }
            TargetPort::Fabric => self
                .fabric
                .health()
                .await
                .map(|()| serde_json::json!({"ok": true})),
            TargetPort::Gate => self.dispatch_gate(entry.kind).await,
            TargetPort::Mcp => self.dispatch_mcp(entry.kind, &input).await,
            TargetPort::Store => self.dispatch_store(entry.kind, &input).await,
        };

        let result = outcome.map_err(AppError::Port)?;
        tracing::info!(
            operator = %auth.subject,
            kind = entry.kind,
            outcome = "ok",
            signature_verified = auth.signature_verified,
            "task complete"
        );
        Ok(result)
    }

    /// Route a forge-targeted task kind to the appropriate forge port method,
    /// forwarding the operator's bearer for RLS. Reads map to metadata/queries.
    /// Store-backed kinds (`project.create`, `application.define`) are NOT here —
    /// they route via [`Self::dispatch_store`] (`TargetPort::Store`).
    /// **Unmapped write kinds return a clean `Downstream("write API pending")` —
    /// never a silent read fallback.**
    async fn dispatch_forge(
        &self,
        kind: &str,
        input: &serde_json::Value,
        bearer: Option<&str>,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        match kind {
            // Reads — genuine forge metadata. `project.inspect`/`project.list` are
            // NOT here: they read the agent-owned store (p8-c001, `dispatch_store`).
            "forge.table.describe" => {
                // Thread the validated table `name` through to the port instead of a
                // placeholder (G1 fix, p13-c002). Schema guarantees a `name` string;
                // reject an empty/whitespace value before the port call.
                let name = parse_table_name(input)?;
                self.forge.describe_table(name, bearer).await
            }
            "forge.table.list" => self.forge.list_tables(bearer).await,
            // Any other forge kind that is write-oriented has no mapping yet.
            other => Err(fpa_ports::PortError::Downstream(format!(
                "write API pending: forge kind '{other}' not implemented"
            ))),
        }
    }

    /// Route an agent-owned aggregate write (`TargetPort::Store`) to its handler.
    /// These mutate the `Project` aggregate in the `ProjectStore` — no external plane.
    async fn dispatch_store(
        &self,
        kind: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        match kind {
            "project.create" => self.create_project(input).await,
            "application.define" => self.define_application(input).await,
            "project.inspect" => self.inspect_project(input).await,
            "project.list" => self.list_projects().await,
            other => Err(fpa_ports::PortError::Downstream(format!(
                "store kind '{other}' not implemented"
            ))),
        }
    }

    /// Read a single project aggregate from the store by `project_id` (p8-c001).
    /// Unknown id → clean error; performs no forge call.
    async fn inspect_project(
        &self,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        let id = parse_project_id(input)?;
        let project = self.projects.get(&id).await?.ok_or_else(|| {
            fpa_ports::PortError::Downstream(format!("unknown project '{}'", id.0))
        })?;
        serde_json::to_value(&project).map_err(|e| fpa_ports::PortError::Decode(e.to_string()))
    }

    /// List all stored projects (p8-c001), deterministically ordered by id
    /// (Base Rule 35). Performs no forge call.
    async fn list_projects(&self) -> Result<serde_json::Value, fpa_ports::PortError> {
        let mut projects = self.projects.list().await?;
        projects.sort_by_key(|p| p.id.0);
        Ok(serde_json::json!({ "projects": projects }))
    }

    /// Persist a new `Project` aggregate to the agent-owned store (p7-c001).
    ///
    /// Builds the aggregate from validated input: `name` required; `id` from
    /// `project_id` or a fresh v4; `schema_version` is **server-owned** (always the
    /// current `SCHEMA_VERSION` — a client value is ignored). The nested collections
    /// (`applications`/`sub_agents`/`schemas`/`realtime`/`entity_meta`) are
    /// serde-deserialized into their typed forms; a malformed nested item is rejected
    /// BEFORE any store write. Performs no forge write — the Project has no forge table.
    async fn create_project(
        &self,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        // Input is schema-validated upstream: `name` is required.
        let name = input
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                fpa_ports::PortError::Downstream("project.create requires 'name'".to_owned())
            })?;
        // Deterministic when the caller supplies `project_id`; else a fresh v4.
        let id = match input.get("project_id").and_then(serde_json::Value::as_str) {
            Some(s) => ProjectId(Uuid::parse_str(s).map_err(|e| {
                fpa_ports::PortError::Downstream(format!("invalid project_id: {e}"))
            })?),
            None => ProjectId(Uuid::new_v4()),
        };
        // Start from an empty aggregate (server owns id + schema_version), then map
        // any nested input onto it via serde — deserialization IS the validation.
        let mut project = Project::new(id, name);
        deser_field(input, "applications", &mut project.applications)?;
        deser_field(input, "sub_agents", &mut project.sub_agents)?;
        deser_field(input, "schemas", &mut project.schemas)?;
        deser_field(input, "entity_meta", &mut project.entity_meta)?;
        if let Some(rt) = input.get("realtime") {
            project.realtime = serde_json::from_value(rt.clone()).map_err(|e| {
                fpa_ports::PortError::Downstream(format!("invalid 'realtime': {e}"))
            })?;
        }
        self.projects.put(&project).await?;
        serde_json::to_value(&project).map_err(|e| fpa_ports::PortError::Decode(e.to_string()))
    }

    /// Define (upsert) an application within an existing project (p7-c002).
    ///
    /// Loads the project by `project_id` from the store, upserts the supplied
    /// `ApplicationDef` by its id (replace same-id, else append — immutably), and
    /// persists the mutated aggregate. An unknown `project_id` is rejected; no
    /// project is created implicitly.
    async fn define_application(
        &self,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        let project_id = parse_project_id(input)?;
        let app: ApplicationDef = input
            .get("application")
            .ok_or_else(|| {
                fpa_ports::PortError::Downstream(
                    "application.define requires 'application'".to_owned(),
                )
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    fpa_ports::PortError::Downstream(format!("invalid 'application': {e}"))
                })
            })?;

        let mut project = self.projects.get(&project_id).await?.ok_or_else(|| {
            fpa_ports::PortError::Downstream(format!("unknown project '{}'", project_id.0))
        })?;

        // Immutable upsert by application id: keep all others, drop the same id, append.
        let mut apps: Vec<ApplicationDef> = project
            .applications
            .into_iter()
            .filter(|a| a.id != app.id)
            .collect();
        apps.push(app);
        project.applications = apps;

        self.projects.put(&project).await?;
        serde_json::to_value(&project).map_err(|e| fpa_ports::PortError::Decode(e.to_string()))
    }

    /// Dispatch a gate-targeted task kind (p5-c003 G3). Read kinds call
    /// `list_routes`; **write kinds refuse cleanly** rather than silently listing
    /// routes.
    ///
    /// `application.deploy` is catalogued (so the surface contract is complete)
    /// but **cannot be honestly implemented yet**: the [`GateAdmin`] port exposes
    /// only `list_routes` — there is no verified flint-gate admin write endpoint
    /// to drive a route/auth mutation. It therefore refuses with a message naming
    /// the missing contract, rather than returning a fake success (p14-c002).
    ///
    /// [`GateAdmin`]: fpa_ports::GateAdmin
    async fn dispatch_gate(&self, kind: &str) -> Result<serde_json::Value, fpa_ports::PortError> {
        if is_gate_write_kind(kind) {
            return Err(fpa_ports::PortError::Downstream(format!(
                "gate route-write not implemented: '{kind}' — no verified flint-gate \
                 admin write endpoint (the GateAdmin port exposes list_routes only)"
            )));
        }
        self.gate.list_routes().await
    }

    /// Dispatch an MCP-targeted task kind (p14-c001). `mcp.tool.list` is a read
    /// (tool discovery); `mcp.tool.call` is an **invoke** — it parses `{name,
    /// arguments}` and forwards them to the downstream MCP server via
    /// [`McpClient::call_tool`].
    ///
    /// `mcp.tool.call` is **non-idempotent**: a retried invoke may act on the
    /// downstream twice. Any Mcp kind without a mapping returns a clean
    /// `Downstream(...)` — it is **never** silently downgraded to a `list_tools`
    /// read, which would misrepresent an unimplemented invoke as a successful
    /// listing.
    async fn dispatch_mcp(
        &self,
        kind: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value, fpa_ports::PortError> {
        match kind {
            "mcp.tool.list" => self.mcp.list_tools().await,
            "mcp.tool.call" => {
                let (name, arguments) = parse_tool_call(input)?;
                // Never log `arguments` — they may carry operator secrets/PII.
                self.mcp.call_tool(name, arguments).await
            }
            other => Err(fpa_ports::PortError::Downstream(format!(
                "mcp kind '{other}' not implemented"
            ))),
        }
    }
}

/// Whether a gate-targeted task kind performs a write (route/auth mutation).
///
/// Write kinds must refuse (see [`TaskRunner::dispatch_gate`]) until gate
/// route-writes are implemented. Read kinds fall through to `list_routes`.
fn is_gate_write_kind(kind: &str) -> bool {
    matches!(kind, "application.deploy")
}

/// Parse a **required** `project_id` (a UUID string) from task input into a
/// [`ProjectId`]. Missing/non-string → `Downstream("… requires 'project_id'")`;
/// unparseable → `Downstream("invalid project_id: …")`. Shared by the store-backed
/// kinds that operate on an existing project (`application.define`,
/// `project.inspect`). `project.create` treats `project_id` as *optional* and does
/// not use this.
fn parse_project_id(input: &serde_json::Value) -> Result<ProjectId, fpa_ports::PortError> {
    input
        .get("project_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| fpa_ports::PortError::Downstream("requires 'project_id'".to_owned()))
        .and_then(|s| {
            Uuid::parse_str(s)
                .map(ProjectId)
                .map_err(|e| fpa_ports::PortError::Downstream(format!("invalid project_id: {e}")))
        })
}

/// Parse a `mcp.tool.call` input into `(name, arguments)` (p14-c001).
///
/// The catalog schema (`SCHEMA_MCP_TOOL_CALL`) already guarantees both `name`
/// (string) and `arguments` (object) are present, but this re-checks at the port
/// boundary so a schema/dispatch drift can never forward a malformed call. `name`
/// is borrowed from `input`; `arguments` is cloned (owned by the returned tuple)
/// for the port call. Returns `Downstream(...)` on either violation. The returned
/// `arguments` are **never logged** — the caller forwards them straight to the port.
fn parse_tool_call(
    input: &serde_json::Value,
) -> Result<(&str, serde_json::Value), fpa_ports::PortError> {
    let name = input
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            fpa_ports::PortError::Downstream("mcp.tool.call requires a non-empty 'name'".to_owned())
        })?;
    let arguments = input.get("arguments").cloned().ok_or_else(|| {
        fpa_ports::PortError::Downstream("mcp.tool.call requires 'arguments'".to_owned())
    })?;
    if !arguments.is_object() {
        return Err(fpa_ports::PortError::Downstream(
            "mcp.tool.call 'arguments' must be an object".to_owned(),
        ));
    }
    Ok((name, arguments))
}

/// Parse a **required**, non-empty table `name` (a string) from task input.
///
/// The catalog schema (`SCHEMA_TABLE_NAME`) already guarantees `name` is present
/// and a string; this additionally rejects an empty/whitespace-only value with
/// `Downstream("… requires a non-empty 'name'")` so an empty table never reaches
/// the forge port. Used by the `forge.table.describe` dispatch (p13-c002 G1).
fn parse_table_name(input: &serde_json::Value) -> Result<&str, fpa_ports::PortError> {
    input
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            fpa_ports::PortError::Downstream(
                "forge.table.describe requires a non-empty 'name'".to_owned(),
            )
        })
}

/// Deserialize an optional nested array field from `input` into `target`, if present.
///
/// Absent → leave `target` untouched (its `#[serde(default)]` empty). Present but
/// malformed → a `Downstream` error naming the field (rejected before any store
/// write). This is where "serde IS the validation" is enforced for the nested
/// `Project` collections (p7-c001).
fn deser_field<T: serde::de::DeserializeOwned>(
    input: &serde_json::Value,
    field: &str,
    target: &mut T,
) -> Result<(), fpa_ports::PortError> {
    if let Some(v) = input.get(field) {
        *target = serde_json::from_value(v.clone())
            .map_err(|e| fpa_ports::PortError::Downstream(format!("invalid '{field}': {e}")))?;
    }
    Ok(())
}
