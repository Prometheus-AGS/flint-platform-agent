## Why

`application.deploy` is catalogued `TargetPort::Gate` / `required_role: "admin"` and is the
one Gate **write** in the catalog. Today it refuses cleanly: `is_gate_write_kind` matches it,
so `dispatch_gate` returns `PortError::Downstream("gate route-write not implemented: 'application.deploy'")`
— an honest, declared refusal, **not** a `{}`-returning stub.

The seed for this phase framed it as "a refused stub to promote to a real write." Assessment
found that framing wrong on both counts, and a real write **blocked**:

- It is **not** a stub — it already refuses correctly.
- A **real** write is not honestly implementable this phase. There is **no verified gate
  admin write contract** to build against:
  - The `GateAdmin` port exposes **only** `list_routes` (`crates/fpa-ports/src/gate.rs`) —
    no write method.
  - `GateAdapter` implements **only** `GET /routes` (`crates/fpa-gate/src/lib.rs`); its own
    doc-comment records that only the **read** path was verified from gate source, and the
    stale `flint-gate-client` `/v1/admin` write prefix was deliberately rejected.
  - A real `application.deploy` would need, in order: (a) verify a gate admin route-create /
    deploy endpoint + payload against gate's own source (`../flint-gate`, read-only
    reference — the agent authors nothing there), (b) a new `GateAdmin` write method (new
    port surface), (c) a `GateAdapter` implementation with a wiremock test, (d) a real input
    schema, (e) a `dispatch_gate` write arm, (f) audit. That is a gate-contract-dependent,
    multi-crate change out of honest reach here.

Per "never fake green", Base Rule 5 (no invented APIs), and Base Rules 22/23 (verify a
dependency contract before building against it), `application.deploy` MUST remain a
**declared refused write** this phase — with the blocking dependency documented so the next
owner knows exactly what unblocks it, rather than left as a silent TODO.

## What Changes

1. **Refusal message** (`task_runner/mod.rs`) — tighten the `dispatch_gate` write-refusal
   message for `application.deploy` to name the missing contract, e.g.
   `"gate route-write not implemented: 'application.deploy' — no verified flint-gate admin write endpoint (GateAdmin exposes list_routes only)"`.
   Behavior is unchanged: it still returns `PortError::Downstream` (a refusal), never
   success. This is a message-only improvement (honest, in-scope, no new behavior).

2. **Dependency documentation** — record the gate-write dependency so it is inspectable and
   auditable (Base Rules 10/34):
   - A spec requirement (this delta) stating `application.deploy` is a declared refused write
     and enumerating the six steps a real write requires.
   - A memory entry capturing the blocker (`GateAdmin` = `list_routes` only; gate admin
     write endpoint unverified) so a future phase does not re-derive it.

No catalog change (the entry stays as-is: Gate / admin / refused). No new port. No sibling
repo edit. No behavior change beyond the refusal message text.

## Capabilities

### Modified Capabilities
- `task-catalog`: `application.deploy` is formally specified as a **declared refused write**
  — it remains catalogued and gated at `admin`, still refuses (never succeeds), and its
  refusal now names the missing gate admin write contract as the documented blocker to a
  real implementation.

## Impact

- `crates/fpa-app/src/task_runner/mod.rs` (refusal message string only).
- `.kbd-orchestrator` memory + this spec (dependency documentation).
- No catalog change, no new port, no gateway change, no sibling repo.
- Fewer than 3 files touched and effectively documentation/message-only → artifact-refiner
  QA gate may be skipped per the kbd-execute skip rule.

## Open Questions

- Resolved at Assess: `application.deploy` stays a **declared refused write** this phase; a
  real gate write is a future gate-contract-dependent phase. Refusal message tightening =
  **yes** (cheap, honest, no behavior change).
