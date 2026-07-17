# Goals

Seeded by `/kbd-next-phase` from the `a2a-task-catalog` reflection's recommended
next phase. This is the **natural continuation** of the p13 read phase: that phase
deliberately scoped writes out (reads-first, to safely light up dead stubs). This
phase catalogs the **write / invoke** paths whose ports already exist.

Agent-owned work — no sibling repo touched, no fabric dependency, no new ports
(same discipline as p13). The p13 read work is the template; writes raise the bar
on role gating, idempotency, and audit.

> **These goals are a seed, not a spec.** `/kbd-assess` re-inspects the live code
> before anything is planned — treat the entries below as the intended direction,
> and correct them against `crates/fpa-app/src/{catalog.rs, task_runner/}` and
> `crates/fpa-ports/src/` during assessment.

## Grounding (what p13 left in place)

After p13 (`a2a-task-catalog`), the catalog (`crates/fpa-app/src/catalog.rs`,
`CATALOG`) has 10 entries; the runner is split into
`crates/fpa-app/src/task_runner/{mod.rs, tests.rs}` (both < 500 lines). The runner
validates a submitted `kind` against the catalog, checks `required_role`, validates
input against the entry's JSON Schema, then dispatches by `TargetPort`. Writes are
classified explicitly (`is_gate_write_kind`, `GATE_READ_KINDS`) so a Gate kind that
is neither a known read nor a known write fails the build.

Relevant ports already present (real seams, some write methods still uncatalogued):
- `McpClient { list_tools (catalogued: mcp.tool.list), call_tool (**uncatalogued write/invoke**) }`
- `GateAdmin { list_routes (catalogued: gate.route.list) }` — `application.deploy`
  targets Gate but is still a **refused write stub** (`dispatch_gate` does not perform it).
- `ForgeMetadata { list_tables, describe_table, create_entity (**uncatalogued write**) }`
- `ProjectStore { put, get, list }` — already exercised by real write kinds
  (`project.create`, `application.define`).

## Goals (seed — confirm/adjust in assess)

- **GW1 — Catalog `mcp.tool.call` (the MCP invoke path).** `McpClient::call_tool`
  is a real port method with no catalog kind. Add a write/invoke kind that dispatches
  to it. Unlike a read, this **invokes** a downstream MCP server's tool — it needs a
  real input schema (tool name + arguments), an **admin-floor** (or at least operator)
  role, and an explicit dispatch arm. Add a runner integration test with a fake
  `McpClient` asserting the call reached the port with the right tool name + args.

- **GW2 — Promote `application.deploy` from stub to a real gate write** *(only if the
  role model + gate admin contract permit).* Today it targets Gate / admin but
  `dispatch_gate` refuses/stubs it. Wire it to a real `GateAdmin` write over the
  **admin port only**. **flint-gate stays the only auth boundary — the agent consumes
  gate-injected identity, never calls Ory.** If the gate admin write contract isn't
  stable enough to implement honestly, keep it a *declared* refused write (no
  fake-green) and document the dependency rather than faking success.

- **GW3 — Write-classification stays honest and total.** Every new write kind is
  registered in the runner's write classification (mirroring `is_gate_write_kind` /
  the guard tests) so no write can silently fall through to a read path or an empty
  stub. Extend the guard tests to cover the new write kinds. No fabricated data, no
  empty-stub-as-success (Base Rule 5).

- **GW4 — Writes carry audit + idempotency consideration (Base Rules 34/35).** A
  write/invoke task must be auditable (the runner already emits an audit record —
  confirm it captures the write's identity, kind, input shape without logging
  secrets) and its determinism/idempotency posture must be explicit. Where a write is
  naturally idempotent, note it; where it isn't, the audit trail is the safety net.
  Never log JWTs/claims/tenant identifiers/secrets.

- **GW5 — Catalog invariants stay green.** The existing catalog + guard tests
  (`lookup_known_and_unknown`, `kinds_are_unique`, `every_entry_has_valid_input_schema`,
  `every_*_catalog_kind_is_dispatched/classified`) must pass with the new write kinds;
  every new entry gets a real JSON Schema (not `SCHEMA_EMPTY` unless genuinely
  input-free) and a unique kind.

## Non-goals / scope discipline (Base Rule 2)

- **No new ports** unless a goal genuinely needs one (GW1/GW2 reuse `call_tool` /
  `GateAdmin`). `forge.create_entity` is a real port method but catalog-exposing it is
  **out of scope this phase** unless GW1–GW2 land with room to spare — writes to the
  fabric via forge carry RLS/Cedar implications that deserve their own assessment.
- No realtime / receipt work — that's the parked `realtime-receipt-unblock` phase,
  re-entered only when `flint-realtime-fabric#2` closes.
- No broadening of the gate admin surface beyond `application.deploy` (route CRUD,
  auth-provider inspection = the separate `gate-admin-surface-expansion` candidate).

## Constraints (carried from p13, non-negotiable)

- Hexagonal layering: domain/app never import an adapter; catalog + runner stay in
  `fpa-app`; ports stay trait-only in `fpa-ports`; wiring only in `fpa-gateway`.
- No `unwrap()`/`expect()` in library crates (`thiserror` in libs, `anyhow` at the
  binary edge only); `#[non_exhaustive]` on public enums; newtype IDs.
- `clippy::pedantic` + `-D warnings`; `cargo fmt --all`; edition 2024 / toolchain
  1.93. No file over 500 lines — split into a directory module if approached.
- **Topology/infra-revealing or state-changing operations default their role UP**
  — writes floor at operator, gate/infra writes at admin (Base Rule 33). See the
  `gate.route.list = operator` precedent from p13.
- Implementation-first: implement all goals before writing integration tests; ≤3
  `cargo test` runs for the phase; `cargo check --workspace` sparingly at section
  boundaries.
- flint-gate is the only auth boundary. Never fake green — a stub that returns `{}`
  is not a passing task. Writes must be auditable (Base Rule 34).

## Success criteria (seed)

1. `mcp.tool.call` exists in `CATALOG`, dispatches to `McpClient::call_tool` with a
   real schema, and a runner test proves the tool name + args reach the port.
2. `application.deploy` either performs a real gate admin write (test proves the port
   was called) **or** remains an honestly-declared refused write with a documented
   dependency — never a silent empty-stub-as-success.
3. Every new write kind is registered in the runner's write classification and
   covered by a guard test; no write falls through to a read path.
4. All catalog invariant + guard tests pass; `cargo check --workspace` clean;
   `cargo clippy --workspace --all-targets -- -D warnings` clean.
