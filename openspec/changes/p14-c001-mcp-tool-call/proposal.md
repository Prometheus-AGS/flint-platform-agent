## Why

The agent is an MCP **client** that composes downstream MCP servers' tools into its
administrative toolset. p13-c003 added `mcp.tool.list` (read/discovery) but explicitly
**deferred** `mcp.call_tool` as "an invoke/write, out of the catalog this phase". That is
the one real invoke path the client role needs, and it is already wired end-to-end but
unreachable:

- **The port method is real.** `McpClient::call_tool(&self, name: &str, arguments: Value)`
  exists (`crates/fpa-ports/src/mcp.rs`).
- **The adapter is real, not a stub.** `McpClientAdapter::call_tool`
  (`crates/fpa-mcp/src/lib.rs`) issues JSON-RPC `tools/call` with `{name, arguments}` over
  HTTP — genuine downstream invocation.
- **But no catalog kind targets it.** The runner's `TargetPort::Mcp` arm is a *flat*
  `self.mcp.list_tools().await` (`task_runner/mod.rs:127`) — it can only ever list. There is
  no per-kind routing, so `call_tool` is dead code from the catalog's perspective.

Both agent surfaces (A2A `submit`, MCP `tool_definitions`) iterate `catalog::CATALOG`, so a
catalog entry auto-exposes the new kind on both protocols with **no gateway edit**. The only
code the runner needs is to split the flat Mcp arm into a `dispatch_mcp` that routes by kind
— mirroring the existing `dispatch_gate` / `dispatch_forge` / `dispatch_store` helpers.

This is the catalog's first **invoke** (as opposed to a metadata read or an agent-owned
store write). It therefore needs stricter role gating and an explicit audit + idempotency
posture (Base Rules 33/34/35).

## What Changes

1. **Catalog** (`catalog.rs`) — add one invoke entry to `CATALOG`:
   - `mcp.tool.call` → `TargetPort::Mcp`, `required_role: "operator"`, a **real** input
     schema (`SCHEMA_MCP_TOOL_CALL`) requiring a `name` string and an `arguments` object.
     Operator (not viewer): invoking a downstream tool is a state-changing action, not
     benign discovery — floor the role up per the topology/writes house rule. Not admin:
     that tier is reserved for gate/infra topology mutation (`application.deploy`).

2. **Runner** (`task_runner/mod.rs`) — replace the flat `TargetPort::Mcp` arm with a
   `dispatch_mcp(kind, input)` helper that routes by kind:
   - `mcp.tool.list` → `self.mcp.list_tools()` (unchanged behavior).
   - `mcp.tool.call` → parse the validated `{name, arguments}` and call
     `self.mcp.call_tool(name, arguments)`.
   - **unknown Mcp kind → clean `PortError::Downstream(...)`** — no silent `list_tools`
     fallback (mirrors `dispatch_gate` / `dispatch_forge`).
   Add a `parse_tool_call` helper (name + arguments extraction) alongside the existing
   `parse_project_id` / `parse_table_name` helpers.

3. **Audit + idempotency posture** — the existing `run()` pipeline already: checks
   `required_role` **before** any port call (Base Rule 33), validates input against the
   kind's schema, and emits `tracing` allow/complete records with `signature_verified`
   provenance and **no token/claims/secret logging** (Base Rule 34). `mcp.tool.call`
   inherits all of this. This change:
   - Adds a test asserting the tool-call **`arguments` payload is not emitted in logs**
     (the audit record logs the operator + kind + decision, never the arguments).
   - Documents `mcp.tool.call` as **non-idempotent by default** (Base Rule 35): idempotency
     is a property of the downstream tool, not guaranteeable by the agent; the audit record
     is the safety net. No dedup/idempotency-key machinery is added (YAGNI).

4. **Guards** (`task_runner/tests.rs`) — extend `every_mcp_catalog_kind_is_dispatched`
   (`MCP_KINDS = &["mcp.tool.list", "mcp.tool.call"]`) so a future undispatched Mcp kind
   fails the build.

Out of scope (unchanged): a **real** `application.deploy` gate write (blocked — see
p14-c002); `forge.create_entity` (RLS/Cedar; own phase); multi-endpoint MCP server routing
(the `McpClientAdapter` is single-endpoint — `{name, arguments}` only, no server selector;
YAGNI this phase).

## Capabilities

### New Capabilities
- `mcp.tool.call`: an operator can **invoke** a named tool on a downstream MCP server the
  agent composes, passing `arguments`, as an A2A task / MCP tool. The call reaches
  `McpClient::call_tool` (real JSON-RPC `tools/call`), gated at the operator floor and
  audited without logging the arguments.

### Modified Capabilities
- `task-catalog`: the Mcp port's **invoke** side is now reachable through a declared catalog
  kind; the `TargetPort::Mcp` dispatch is no longer flat but a `dispatch_mcp` split with a
  clean unknown-kind refusal and a dispatch-coverage guard over both Mcp kinds.

## Impact

- `crates/fpa-app/src/catalog.rs` (one `CatalogEntry` + one schema const).
- `crates/fpa-app/src/task_runner/mod.rs` (`dispatch_mcp` split; `parse_tool_call` helper).
- `crates/fpa-app/src/task_runner/tests.rs` (extend the Mcp guard; add invoke + arguments-
  not-logged + unknown-kind-refusal tests).
- No gateway change (both surfaces read `catalog::CATALOG`). **No new port** — `call_tool`
  already exists. No sibling repo.
- Depends on `p13-c001` (runner directory-module split) and `p13-c003` (`mcp.tool.list` +
  `every_mcp_catalog_kind_is_dispatched`).

## Open Questions

- Resolved at Assess/Plan: `mcp.tool.call` role = **operator** (invoke, not discovery);
  input schema = `{name, arguments}` single-endpoint; idempotency = **non-idempotent,
  documented, audit is the safety net**.
