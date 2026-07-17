# Assessment — a2a-task-catalog-writes

**Stage:** assess · **Phase:** a2a-task-catalog-writes · **Date:** 2026-07-08
**Method:** live inspection of `crates/fpa-app/src/{catalog.rs, task_runner/mod.rs}`,
`crates/fpa-ports/src/{mcp.rs, gate.rs, error.rs, lib.rs}`, and the
`crates/fpa-{mcp,gate}/src/lib.rs` adapters. The goals.md was a *seed*; this
assessment corrects it against reality.

## Executive summary

Two write/invoke goals were seeded. Inspection splits them cleanly:

- **GW1 `mcp.tool.call` — GREEN / fully implementable this phase, no new port.**
  The port method and the real JSON-RPC adapter already exist end-to-end. Only
  catalog + runner-dispatch + schema + tests are missing.
- **GW2 `application.deploy` real write — RED / NOT honestly implementable this
  phase.** The seed's premise ("promote from refused stub") was **factually
  wrong in one respect and blocked in another**: it is already catalogued
  (Gate/admin) and *already refuses cleanly* (not a `{}`-returning stub), but
  making it a **real** write needs a gate admin **write** endpoint that does not
  exist in the port, the adapter, or (verified) in gate itself. Per "never fake
  green" + Base Rule 5, GW2 must **remain a declared refused write** with the
  dependency documented — not faked.

Net: this phase delivers **one real new write/invoke kind (`mcp.tool.call`)**
plus the write-classification/audit hardening around it. GW2 converts to a
documented-blocker + honest-refusal task, not a real-write task.

## Ground-truth findings

### F1 — `mcp.tool.call` is absent from the catalog (confirms GW1 gap)
`CATALOG` (catalog.rs:72-147) has 10 entries; `mcp.*` has only `mcp.tool.list`
(Mcp/viewer). No `mcp.tool.call`. **Gap real.**

### F2 — `McpClient::call_tool` is a real port method with a real adapter
- Port: `McpClient::call_tool(&self, name: &str, arguments: Value)` (mcp.rs:16-20).
- Adapter: `McpClientAdapter::call_tool` (fpa-mcp/lib.rs:65-71) issues JSON-RPC
  `tools/call` with `{name, arguments}` over HTTP — **genuinely wired, not a stub.**
  So GW1 dispatches to real behavior. ✅ No new port needed.

### F3 — the runner's `TargetPort::Mcp` arm is flat — GW1 needs a runner edit
mod.rs:127 is `TargetPort::Mcp => self.mcp.list_tools().await` — a single-method
route, no per-kind branching (unlike Forge/Gate/Store which have `dispatch_*`
helpers). Adding `mcp.tool.call` therefore **requires** a `dispatch_mcp(kind,
input)` split:
- `mcp.tool.list` → `list_tools()`
- `mcp.tool.call` → parse `{name, arguments}` → `call_tool(name, arguments)`
- unknown mcp kind → clean `Downstream(...)` (no silent `list_tools` fallback,
  mirroring `dispatch_gate`/`dispatch_forge`).
**Correction to the seed:** the seed leaned on p13's "adding a kind = catalog-only
edit." That held for reads whose arm already existed; it does **not** hold here.

### F4 — `application.deploy` is already catalogued AND already refuses cleanly
catalog.rs:101-107 → Gate / **admin** / `SCHEMA_EMPTY`. `is_gate_write_kind`
(mod.rs:311-313) matches `"application.deploy"`, so `dispatch_gate` (mod.rs:297-304)
returns `Downstream("gate route-write not implemented: 'application.deploy'")`.
**It is NOT a `{}`-returning stub** — it is an honest, declared refusal today. The
seed's "promote from refused stub to real write" mischaracterized the current
state (the refusal is correct behavior, and the "stub" framing is inaccurate).

### F5 — there is NO gate admin *write* contract to implement against (GW2 blocker)
- `GateAdmin` port exposes **only** `list_routes` (gate.rs:11-14). No write method.
- `GateAdapter` implements **only** `GET /routes` (fpa-gate/lib.rs:44-68). Its
  own doc-comment (lines 8-11) states only the **read** path was verified from
  gate source; the stale `flint-gate-client` `/v1/admin` write prefix was
  deliberately rejected.
- Therefore a real `application.deploy` write requires, in order: (a) verify a
  gate admin route-**create/deploy** endpoint + payload shape against gate source
  (`../flint-gate`, read-only reference — house rule: author nothing there);
  (b) add a `GateAdmin` write method (**new port surface**); (c) implement it in
  `GateAdapter` with a wiremock test; (d) a real input schema; (e) a
  `dispatch_gate` write arm; (f) admin-floor gating (already admin) + audit.
  **This is a gate-contract-dependent, multi-crate change — out of honest reach
  this phase.** Base Rule 22/23 (verify the dependency contract before building)
  + Rule 5 (no invented APIs) ⇒ **do not** implement GW2 as a real write now.

### F6 — the audit + permission spine already covers a new write kind
`TaskRunner::run` (mod.rs:61-140) already: checks `required_role` before any port
call (Base Rule 33), validates input against the kind's schema, and emits
`tracing` allow/deny/complete records with `signature_verified` provenance and
**no token/claims/secret logging** (Base Rule 34). A new `mcp.tool.call` entry
inherits all of this for free — GW4's audit requirement is **already satisfied**
by the pipeline; the phase's job is to (a) confirm no secret leakage when the
*arguments* payload flows through, and (b) state the idempotency posture.

### F7 — idempotency posture (Base Rule 35)
`mcp.tool.call` invokes an arbitrary downstream tool — idempotency is the
downstream tool's property, **not guaranteable by the agent**. Correct posture:
document it as **non-idempotent by default; the audit record is the safety net**
(F6). No dedup/idempotency-key machinery is in scope (YAGNI, Base Rule 2) — a
claim we can't enforce would be worse than an honest "not idempotent."

## Goal-by-goal verdict

| Goal | Verdict | Gap / action |
|---|---|---|
| **GW1** catalog `mcp.tool.call` | **GAP — do it** | Add catalog entry (Mcp, admin-floor role, real `{name, arguments}` schema) + `dispatch_mcp` split (F3) + runner test with a fake `McpClient` asserting name+args reach the port. |
| **GW2** real `application.deploy` write | **BLOCKED — reframe** | Keep it a **declared refused write** (F4). Document the gate-write dependency (F5) in the spec + a memory. Do **not** fake. Optional: tighten the refusal message to name the missing contract. **No new port this phase.** |
| **GW3** write-classification honest & total | **PARTIAL — extend** | `mcp.tool.call` is an invoke, not a read → ensure `dispatch_mcp` refuses unknown mcp kinds cleanly (no `list_tools` fallback) and add a guard test that every Mcp catalog kind is dispatched (mirror the gate `every_gate_catalog_kind_is_classified` test). |
| **GW4** audit + idempotency | **MOSTLY MET — confirm + document** | Pipeline already audits (F6). Action: a test asserting the `arguments` payload is **not** logged; document non-idempotent posture (F7). No new audit machinery. |
| **GW5** catalog invariants green | **MET by construction** | New entry gets a real (non-empty) schema + unique kind; existing invariant tests (`kinds_are_unique`, `every_entry_has_valid_input_schema`) cover it. |

## Role decision (Base Rule 33) — pre-answered for plan/spec

`mcp.tool.call` **invokes** a downstream tool (a state-changing action, not a
metadata read). Per the house rule ([[topology-reads-default-role-up]], extended
to writes/invokes flooring higher): **`required_role: "operator"`** as the floor,
matching other `TargetPort` writes (`project.create`/`application.define` are
`operator`). Not `viewer` (it is not benign discovery), not `admin` (that tier is
reserved for infra/gate topology mutation like `application.deploy`). Record the
reason inline mirroring the `gate.route.list` comment.

## Scope corrections vs. the seed (carry into spec/plan)

1. **GW1 requires a runner edit** (a `dispatch_mcp` split), not a catalog-only
   edit — the p13 "catalog-only" shortcut does not apply (F3).
2. **GW2 is not a real-write task** — it is a documented-blocker + honest-refusal
   task. The seed's "no new ports" holds precisely **because** GW2 is not
   implemented as a write (a real write would have *needed* a new port — F5).
3. `forge.create_entity` stays out of scope (RLS/Cedar; own phase) — unchanged.

## Open questions (for plan / spec)

- **Q1:** `mcp.tool.call` role — confirm **operator** floor (recommended above)
  vs. admin. *Recommendation: operator.* Non-blocking; defaulting to operator.
- **Q2:** Multiple downstream MCP servers — the current `McpClient` is a single
  endpoint. Does `mcp.tool.call` need a server selector in its input, or is
  single-endpoint fine for now? *Inspection:* `McpClientAdapter` is single-endpoint
  (one `endpoint` field). **Recommendation:** single-endpoint this phase (YAGNI);
  schema takes `{name, arguments}` only; multi-server routing is a later phase.
- **Q3:** Should the GW2 refusal message be tightened to name the missing gate
  write contract? *Recommendation:* yes, a one-line message improvement — cheap,
  honest, in-scope; no behavior change.

## Success criteria (refined from the seed)

1. `mcp.tool.call` in `CATALOG` (Mcp / operator / real `{name, arguments}` schema);
   runner `dispatch_mcp` routes it to `McpClient::call_tool`; a test with a fake
   `McpClient` proves the tool **name + arguments** reach the port unaltered.
2. Unknown Mcp kinds refuse cleanly (no `list_tools` fallback); a guard test
   asserts every Mcp catalog kind is dispatched.
3. `application.deploy` remains a **declared refused write** — a test asserts it
   still refuses (never returns success); the gate-write dependency is documented
   (spec + memory). No new port added.
4. A test asserts `mcp.tool.call` `arguments` are not emitted in logs; the
   non-idempotent posture is documented.
5. `cargo check --workspace` clean; `cargo clippy --workspace --all-targets -- -D
   warnings` clean; catalog invariant + new guard tests pass (within the ≤3
   `cargo test` budget).

## Handoff to next stage

Next: `/kbd-analyze` (likely `--skip` — no external library research needed; this
is agent-owned wiring against ports that already exist) → `/kbd-spec`. The spec
will carry: one implementation change (GW1 + GW3/GW4 hardening) and one
documentation/honest-refusal change (GW2 reframe). Recommended: **2 changes**.
