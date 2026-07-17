## Why

Two adapter capabilities are wired but unreachable because no catalog kind targets them:

- **Gate reads.** `dispatch_gate` already calls `self.gate.list_routes()` for non-write
  kinds — it is *not* a stub. But the only Gate kind in the catalog is `application.deploy`
  (a write, correctly refused), so `list_routes` can never be reached. The guard test
  `every_gate_catalog_kind_is_classified` even carries an empty `GATE_READ_KINDS = &[]`
  placeholder explicitly waiting for a Gate read kind.
- **MCP reads.** The runner has a `TargetPort::Mcp => self.mcp.list_tools()` dispatch arm,
  but no catalog kind targets `Mcp`, so the arm is dead.

Both surfaces (A2A `submit`, MCP `tool_definitions`) already iterate `catalog::CATALOG`, so
adding catalog entries auto-exposes the new kinds on both protocols with **no gateway edit**.

## What Changes

1. **Catalog** (`catalog.rs`) — add two read entries to `CATALOG`:
   - `gate.route.list` → `TargetPort::Gate`, `required_role: "operator"`, `SCHEMA_EMPTY`.
     Operator (not viewer): gate routes reveal operational topology (upstreams, auth
     pipelines, streaming config) — an information-disclosure surface. Default the role up
     for a read that leaks structure.
   - `mcp.tool.list` → `TargetPort::Mcp`, `required_role: "viewer"`, `SCHEMA_EMPTY`.
     A tool listing is benign discovery metadata; viewer is appropriate.
2. **Runner** (`task_runner/mod.rs`) — confirm `is_gate_write_kind` returns `false` for
   `gate.route.list` (it already does — only `application.deploy` is a write), so the read
   branch returns `list_routes()`. Confirm `TargetPort::Mcp` dispatch routes `mcp.tool.list`
   to the existing `list_tools()` arm. Register `gate.route.list` in `GATE_READ_KINDS`.
3. **Guards** (`task_runner/tests.rs`) — add `every_mcp_catalog_kind_is_dispatched`
   (`MCP_KINDS = &["mcp.tool.list"]`) mirroring the Gate/Store guards, so a future
   undispatched Mcp kind fails loudly.

Writes stay out of scope: `mcp.call_tool` and `forge.create_entity` are deferred; only
read kinds are added this change.

## Capabilities

### New Capabilities
- `gate.route.list`: an operator can enumerate flint-gate routes as an A2A task / MCP tool,
  reading via gate's admin-port adapter (flint-gate remains the only auth boundary — the
  agent never calls Ory).
- `mcp.tool.list`: any viewer can list the tools exposed by downstream MCP servers the
  agent composes, as an A2A task / MCP tool.

### Modified Capabilities
- `task-catalog`: the read-side of the Gate and Mcp ports is now reachable through declared
  catalog kinds, and Mcp kinds gain a dispatch-coverage guard.

## Impact

- `crates/fpa-app/src/catalog.rs` (two `CatalogEntry`s).
- `crates/fpa-app/src/task_runner/mod.rs` (`GATE_READ_KINDS` registration; confirm dispatch).
- `crates/fpa-app/src/task_runner/tests.rs` (new Mcp guard + dispatch tests).
- No gateway change (both surfaces read `catalog::CATALOG`). No new port. No sibling repo.
- Depends on `p13-c001` (the runner directory-module split).

## Open Questions

- Resolved at Plan: `gate.route.list` role = **operator** (topology disclosure); Mcp
  dispatch guard = **added**.
