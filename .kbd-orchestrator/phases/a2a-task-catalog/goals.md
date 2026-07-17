# Goals

Pivot phase (manual, via `/kbd-new-phase`). We did **not** take the
`realtime-receipt` reflection's auto-seed (`realtime-receipt-unblock`) because
that phase is **BLOCKED** on an upstream fabric fix (`flint-realtime-fabric#2`,
re-verified OPEN against `main @ 7d3d1e9`). Rather than idle-block, we advance an
unrelated administrative surface the reflection itself named as the alternative:
**expand and correct the A2A administrative task catalog**.

This is agent-owned work — no sibling repo is touched, no fabric dependency.

## Grounding (what exists today)

The A2A task catalog is `crates/fpa-app/src/catalog.rs` (`CATALOG`, 9 entries) +
the runner `crates/fpa-app/src/task_runner.rs` that validates a submitted task
`kind` against the catalog, checks `required_role`, validates input against the
entry's JSON Schema, then dispatches by `TargetPort`. Ports live in
`crates/fpa-ports/src/`. Current entries:

| kind | target | role | dispatch reality |
|---|---|---|---|
| `project.create` | Store | operator | real (agent-owned aggregate) |
| `project.inspect` | Store | viewer | real |
| `project.list` | Store | viewer | real |
| `application.define` | Store | operator | real |
| `application.deploy` | Gate | admin | **stub** (`dispatch_gate` returns empty) |
| `forge.table.list` | Forge | viewer | real (`ForgeMetadata::list_tables`) |
| `forge.table.describe` | Forge | viewer | **bug** — passes `"<unspecified>"`, ignores the `name` input |
| `fabric.health` | Fabric | viewer | real (`FabricClient::health`) |

Ports already present (real seams, some unused by the catalog):
`ForgeMetadata { list_tables, describe_table, create_entity }`,
`GateAdmin { list_routes }`, `McpClient { list_tools, call_tool }`,
`FabricClient { health, subscribe }`, `ProjectStore { put, get, list }`.

## Goals

- **G1 — Fix `forge.table.describe` to honour its input (bug, do first).** The
  entry declares a `PROJECT_ID`/`TABLE_NAME`-style schema but the runner calls
  `describe_table("<unspecified>", bearer)` at `task_runner.rs:151`, silently
  discarding the operator's `name`. Thread the validated input's `name` through
  `dispatch_forge` so the described table is the one requested. Add an
  integration test that submitting `{ "name": "some_table" }` reaches the port
  with `"some_table"`, not the placeholder.

- **G2 — Wire the real `GateAdmin` read into the catalog.** `GateAdmin::list_routes`
  is a real port but no task kind reaches it (`application.deploy` is a Gate
  target but a *write* stub). Add a read kind — proposed `gate.route.list`
  (Gate / **viewer**) — and make `dispatch_gate` call `self.gate.list_routes()`
  instead of returning `{"routes":[]}`. Routes are administered over gate's
  **admin** port only; this is a read, so viewer role is the floor — confirm
  against the role model before finalising. **flint-gate stays the only auth
  boundary — the agent consumes gate-injected identity, never calls Ory.**

- **G3 — Wire the real `McpClient` into the catalog.** `McpClient::list_tools`
  is a real port with a stub dispatch (`self.mcp.list_tools()` reached only if a
  kind targets `Mcp`, and none does). Add `mcp.tool.list` (Mcp / viewer) so the
  agent's MCP-**client** role (composing downstream servers' tools) is
  exercisable as an administrative task. `call_tool` is a *write*/invoke path —
  scope it out of this phase unless G1–G3 land with room to spare.

- **G4 — Close the read-vs-write honesty gap in `dispatch_forge`.** The runner
  already routes unmapped forge kinds to a clean `Downstream("write API
  pending")` rather than a silent read fallback — preserve that. When adding any
  new forge read (e.g. a future `forge.table.query` row-read via a new port
  method), it must be a *declared* catalog entry with its own schema, never an
  implicit fallthrough. No fabricated data, no empty-stub-as-success.

- **G5 — Catalog invariants stay green.** The three existing catalog tests
  (`lookup_known_and_unknown`, `kinds_are_unique`, `every_entry_has_valid_input_schema`)
  must still pass with the new kinds; every new entry gets a real JSON Schema
  (not `SCHEMA_EMPTY` unless genuinely input-free) and a unique kind. Extend the
  invariant tests to cover the new kinds' schema validity.

## Non-goals / scope discipline (Base Rule 2)

- No new ports invented unless a goal genuinely needs one (G2/G3 reuse existing
  seams; only a row-read would need a new `ForgeMetadata` method — deferred).
- No write/mutation task kinds beyond what already exists (`application.deploy`
  stays a stub this phase; `mcp.call_tool` deferred). Reads first — they're
  safe, and they light up the two dead stubs honestly.
- No realtime / receipt work — that's the parked `realtime-receipt-unblock`
  phase, re-entered only when fabric#2 closes.

## Constraints (carried, non-negotiable)

- Hexagonal layering: domain/app never import an adapter; catalog + runner stay
  in `fpa-app`; ports stay trait-only in `fpa-ports`; wiring only in
  `fpa-gateway`.
- No `unwrap()`/`expect()` in library crates (`thiserror` in libs, `anyhow` at
  the binary edge only); `#[non_exhaustive]` on public enums; newtype IDs.
- `clippy::pedantic` + `-D warnings`; `cargo fmt --all`; edition 2024 / toolchain
  1.93. No file over 500 lines — if `catalog.rs` or `task_runner.rs` approaches
  it, split into a directory module.
- Implementation-first: implement all of G1–G5 (or the agreed subset) before
  writing the integration tests; ≤3 `cargo test` runs for the phase. Use
  `cargo check --workspace` sparingly at section boundaries.
- Never log JWTs/claims/tenant identifiers/secrets. flint-gate is the only auth
  boundary. Never fake green — a stub that returns `{}` is not a passing task.

## Success criteria

1. `forge.table.describe` provably describes the requested table (test proves the
   input reaches the port).
2. `gate.route.list` and `mcp.tool.list` exist in `CATALOG`, dispatch to their
   real ports, and are covered by runner integration tests (with a fake/adapter
   double asserting the call happened).
3. `dispatch_gate` / the mcp arm no longer return hardcoded empty stubs for the
   cataloged read kinds.
4. All catalog invariant tests pass; `cargo check --workspace` clean;
   `cargo clippy --workspace --all-targets -- -D warnings` clean.
