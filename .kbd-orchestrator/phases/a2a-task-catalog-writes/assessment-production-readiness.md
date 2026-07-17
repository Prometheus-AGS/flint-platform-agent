# Production-Readiness Assessment — Flint Platform Agent

**Scope:** Full production release functionality (broader than the active phase).
**Method:** Code inspection + test execution + sibling-repo state check.
**Stance:** Sycophancy-corrected. No flattering framing. Gaps named, not softened.

---

## Verdict: NOT PRODUCTION-READY

The agent is a well-architected scaffold with genuine working subsystems, but it
cannot be deployed as the administrative agent for the Flint fabric today. Four
hard blockers and several near-term gaps stand between the current state and a
production release.

---

## What Actually Works (stated plainly, not praised)

| Subsystem | State | Evidence |
|---|---|---|
| Hexagonal layering | Clean | `fpa-domain`/`fpa-app` import zero adapter crates (verified by dep inspection) |
| `TaskRunner` use-case layer | Real | 11-entry catalog, RBAC-before-dispatch, JSON-Schema input validation, per-port dispatch, audit. 44 tests. |
| Forge HTTP adapter | Real | `list_tables`/`describe_table`/`create_entity`/GraphQL — wiremock-tested (11 tests) |
| Fabric adapter | Real | `GET /healthz` + WebSocket subscribe with frame decoding (6 tests, real WS server) |
| Gate adapter | Real but narrow | `list_routes` only (3 tests, wiremock) |
| MCP client adapter | Real but unconfigured | JSON-RPC `tools/list`/`tools/call` (1 test — thin) |
| Durable Postgres store | Real | `deadpool-postgres`, JSONB upsert, idempotent schema (3 tests, 1 ignored/testcontainers) |
| Gateway composition | Real | Env-var config with fail-fast + secret redaction, JWKS verification with single-flight + alg-confusion defence, HS256 path, trusted-header trust path with spoofing guard. 19 tests + 6 integration. |
| Protocol types | Real | `#[non_exhaustive]` + `#[serde(other)] Unknown` throughout |
| OpenSpec planning | 95.6% complete | 43/45 changes done; 18 real specs; 14/15 phases fully complete |

**Test suite at time of assessment: 43 pass, 1 FAIL, 0 ignored-run.**

---

## Hard Blockers (must fix before any production release)

### B1. The test suite is RED — 1 failing test

```
task_runner::tests::mcp::mcp_tool_call_never_logs_arguments ... FAILED
crates/fpa-app/src/task_runner/tests/mcp.rs:225
"the invoke must still be audited by kind"
```

This test was added for `p14-c001-mcp-tool-call` (the currently-pending change,
0/17 tasks done) but the implementation doesn't exist yet. The test asserts
`mcp.tool.call` appears in the audit log — but `mcp.tool.call` is not in the
catalog. **A red test suite is a non-negotiable release blocker.** Either land
p14-c001 or remove the test; shipping with `cargo test --workspace` failing
violates the project's own CI contract (`scripts/ci-check.sh`).

### B2. The MCP client is wired to an empty string — runtime failure guaranteed

`crates/fpa-gateway/src/state.rs` constructs `McpClientAdapter::new(String::new())`.
The comment says *"No downstream MCP server configured yet; placeholder."* In a
real deployment, `mcp.tool.list` and `mcp.tool.call` will return
`PortError::Transport` with no operator-facing guidance about the missing
configuration. There is **no env var, no config field, no startup warning** for
this. An operator will hit a cryptic transport error and have to read source to
diagnose it.

**Fix:** Add `FPA_MCP_ENDPOINT` to `GatewayConfig`, fail-fast at startup if
missing (or log a prominent warning), and wire it into `state.rs`.

### B3. The AG-UI surface is a bracket-only stub — no agent run loop exists

`crates/fpa-gateway/src/routes/agui.rs` emits `run_start` → `run_end` with
nothing between. There is no LLM streaming, no tool-call execution, no
state-snapshot emission. **The AG-UI protocol surface is non-functional.** If
"production release" includes the AG-UI surface (and the project brief lists it
as one of four protocol surfaces), this is a hard blocker.

If the AG-UI surface is explicitly out of scope for the first release (this agent
is *administrative*, not conversational), then document that exclusion
explicitly in the release notes — but as written, the surface exists, responds,
and does nothing useful.

### B4. `application.deploy` (and all gate writes) cannot work — by design

`GateAdmin` exposes only `list_routes()`. There is no route-write method, no
deploy method, no auth-provider inspection. `dispatch_gate` refuses all write
kinds with a named `PortError::Downstream(...)` — which is the *honest* posture
(no fake-green), but the capability is absent. If "production release" includes
application deployment via the agent, this is a blocker on the **gate admin
contract**, not on this codebase alone.

The current p14-c002 change tightens the refusal *message* but does not add the
capability. Adding it requires a verified flint-gate admin write endpoint that
does not exist yet.

---

## Near-Term Gaps (not hard blockers, but a production release would surface them)

### N1. `fpa-cli` is a 16-line no-op

```rust
fn main() {
    tracing_subscriber::fmt().init();
    println!("fpa — Flint Platform Agent CLI (scaffold)");
}
```

No subcommands, no logic, no dependencies on any other crate. If operations
requires a CLI (for store migrations, key rotation, health checks, catalog
inspection), it does not exist. If the CLI is not needed for v1, remove it from
the workspace or mark it explicitly as not-shipped.

### N2. No migration framework

`crates/fpa-store-pg/schema.sql` is 10 lines, applied idempotently at adapter
connect time via `batch_execute`. This works for a single-table owned store with
a stable schema. It becomes a liability the moment the `Project` aggregate schema
evolves — there is no `migrations/` directory, no version tracking, no rollback.
For a production system with durable state, this needs at minimum a versioned
schema check and a migration story.

### N3. Two stale doc comments contradict the code

1. **Workspace `Cargo.toml` lint table (lines 100–102):** Claims `todo!()` stubs
   exist *"until the todo!() bodies are implemented"* and *"todo!() stubs panic
   by definition."* **There are zero `todo!()` or `unimplemented!()` calls in the
   entire workspace.** The comment is stale and misleading — it makes the
   codebase look less complete than it is.

2. **`crates/fpa-gateway/src/routes/a2a.rs` module doc:** Claims the handler *"is
   a stub… do not yet drive the runner."* **It does drive the runner** — `submit`
   calls `TaskRunner::run()`, records terminal state, maps `AppError` to HTTP
   statuses. The doc is stale.

### N4. `fpa-mcp` has 1 unit test

The MCP client adapter (`tools/list`, `tools/call`, error-envelope mapping) has
exactly one test: the empty-endpoint transport-error guard. The actual
success/error mapping is exercised only indirectly through `fpa-app` fakes and
gateway integration tests. Direct unit coverage of the JSON-RPC request/response
cycle is missing.

### N5. 22 completed changes are unarchived

OpenSpec changes from phases p1–p12 (22 changes) are marked complete in their
phase `progress.json` but still sit in `openspec/changes/` rather than
`openspec/changes/archive/`. The archival pattern was applied to p1–p6 and p13
but not to later phases. This clutters the active changes directory and makes
the true pending-work count harder to audit.

### N6. Anomalous archive: `p13-c003` has 0/13 task boxes checked

`openspec/changes/archive/2026-07-08-p13-c003-gate-mcp-read-kinds/tasks.md` was
archived and marked complete in the phase progress ledger, but **all 13 task
checkboxes are still `[ ]`**. Either the work was done and the boxes were never
flipped (process error), or the work was not done and the orchestrator recorded
false completion (integrity error). This needs resolution before release —
verify `gate.route.list` / `mcp.tool.list` actually work (they appear to, based
on the code audit, but the discrepancy must be explained).

### N7. `realtime-receipt-unblock` phase is dead/blocked

The phase has only `assessment.md` + `goals.md` + `progress.json` — no plan, no
execution, no reflect. The assessment verdict: "no agent-side work actionable
this phase; gated on upstream `flint-realtime-fabric#2` (still OPEN)." This is
a parked watch phase. If the realtime receipt capability is needed for
production, it is blocked on an upstream issue that is not in this repo's
control.

### N8. Two `real-sibling-smoke` partials blocked on `flint-forge#7`

`p10-c002` (2 tasks blocked) and `p10-c006` (build blocked) are honestly
documented as partial — blocked on upstream `flint-forge#7` (duplicate migration
versions + `.dockerignore` bug). The agent's own `make smoke-real-forge` target
is documented as BLOCKED on the same issue. If production deployment depends on
the real-sibling smoke stack passing, this is an upstream blocker.

---

## Sibling-Dependency Readiness

The agent is an interface over three sibling planes. Their readiness directly
affects what the agent can actually *do* in production:

| Sibling | Crates | Tests | `todo!()` count | Readiness for this agent |
|---|---|---|---|---|
| `flint-forge` | 30 | 503 | 3+ (in fdb-realtime, fdb-reflection) | Substantially built. **Blocked** on forge#7 (migration + dockerignore) for real-sibling smoke. A2UI registry (RFC-FORGE-A2UI-001) is an RFC, not built — the agent's `RegistryComponentId` is a forward-compat placeholder. |
| `flint-realtime-fabric` | 24 | 196 | — | Substantially built. **Blocked** on fabric#2 for realtime-receipt unblock. The vendored `frf-domain` types (344 LOC) are real and match the `/ws/v1/subscribe` wire format. |
| `flint-gate` | 3 | 529 | — | Substantially built. The agent's `GateAdmin` port exposes only `list_routes` — the gate admin **write** contract needed for `application.deploy` is not verified/stable enough for the agent to consume. |

**Bottom line:** The siblings are real, not vaporware, but the agent depends on
specific upstream contracts (gate admin writes, forge migration stability,
fabric realtime receipt) that are not yet deliverable. The agent correctly
refuses to fake these — but a production release cannot ship capabilities that
depend on unfinished upstream work.

---

## Security Posture (genuinely strong — stated without inflation)

This is the area where the codebase is closest to production-ready:

- **JWKS verification** is real: TTL cache, single-flight refresh (verified by a
  16-caller concurrency test), algorithm-confusion defence (server-fixed allowlist),
  empty-set poisoning defence.
- **HS256 path** is real and separate from the JWKS path.
- **Trusted-header trust path** has a spoofing guard (`FPA_BEHIND_TRUSTED_GATE`
  must be explicitly set).
- **No unverified-decode fallback** — the code never decodes JWTs without
  verifying the signature.
- **Secret redaction** is manual and thorough: `GatewayConfig`'s `Debug` impl
  redacts 4 secret fields; bearer tokens are redacted in logs.
- **RBAC is before port dispatch** — role check happens before any port call,
  not after.
- **No `unwrap()`/`expect()` in library crates** (verified by grep; the 43
  occurrences are all in `#[cfg(test)]`).
- **`clippy::pedantic` + `-D warnings`** is the CI gate and is green (the one
  test failure is a logic failure, not a lint failure).

This is not beginner security. It is deliberate, tested, and defense-in-depth.

---

## Gap Summary: What Production Release Requires That Does Not Exist

| # | Gap | Severity | Owner | Blocked on |
|---|---|---|---|---|
| B1 | Test suite is red (1 failing test for unimplemented p14-c001) | **Blocker** | This repo | Nothing — land the change or remove the test |
| B2 | MCP client wired to empty string, no config, no startup warning | **Blocker** | This repo | Nothing — add `FPA_MCP_ENDPOINT` config |
| B3 | AG-UI surface is a bracket stub, no agent run loop | **Blocker** | This repo | Scope decision: is AG-UI in v1? |
| B4 | `application.deploy` / gate writes have no port method | **Blocker** | This repo + gate | flint-gate admin write contract stabilization |
| N1 | `fpa-cli` is 16-line no-op | Near-term | This repo | Scope decision |
| N2 | No migration framework for durable store | Near-term | This repo | Nothing |
| N3 | Two stale doc comments contradict the code | Near-term | This repo | Nothing |
| N4 | `fpa-mcp` unit coverage is thin (1 test) | Near-term | This repo | Nothing |
| N5 | 22 completed changes unarchived | Hygiene | This repo | Nothing |
| N6 | `p13-c003` archive has 0/13 task boxes checked | Integrity | This repo | Investigation |
| N7 | Realtime receipt blocked upstream | Watch | Upstream | flint-realtime-fabric#2 |
| N8 | Real-sibling-smoke partials blocked upstream | Watch | Upstream | flint-forge#7 |

---

## Recommended Next Actions (ordered by dependency)

1. **Land p14-c001 (`mcp.tool.call`)** or remove the failing test. The test
   suite must be green before anything else matters.
2. **Add `FPA_MCP_ENDPOINT` config** to `GatewayConfig` and wire it in
   `state.rs`. Fail-fast at startup if MCP kinds are catalogued but no endpoint
   is configured.
3. **Make a scope decision on AG-UI**: is it in v1? If yes, the agent run loop
   is the largest single piece of missing work. If no, document the exclusion
   and consider removing the route (or returning `501 Not Implemented` with a
   clear message instead of an empty bracket).
4. **Make a scope decision on `application.deploy`**: if it's in v1, the gate
   admin write contract must be stabilized and consumed. If not, the current
   honest-refusal posture is acceptable for v1 — but the release notes must say
   so.
5. **Fix the two stale doc comments** (Cargo.toml lint table, a2a.rs module
   doc). These are 5-minute fixes that improve accuracy immediately.
6. **Resolve the `p13-c003` archive anomaly**: verify the work was done (the
   code audit suggests it was) and flip the task boxes, or document what
   happened.
7. **Archive the 22 completed-but-unarchived changes** to clean the active
   changes directory.
8. **Add direct unit tests for `fpa-mcp`** (JSON-RPC request/response cycle).
9. **Add a migration framework** if the `Project` schema is expected to evolve.

---

## What This Codebase Is Not

To prevent misframing (sycophancy pattern S-01):

- It is **not** a conversational LLM agent. It has no model client, no prompt
  pipeline, no tool-call loop. It is an administrative task runner.
- It is **not** a complete fabric management plane. It manages Projects and
  Applications (its own owned aggregate) and inspects/lists fabric state. It
  cannot deploy, configure routes, manage auth providers, or invoke forge
  writes through the catalog (the port method exists but is uncatalogued).
- It is **not** deployable as-is. The empty MCP endpoint alone makes
  `mcp.tool.list`/`call` fail at runtime, and the red test suite means CI
  (`./scripts/ci-check.sh`) will not pass.

## What This Codebase Genuinely Is

- A clean, well-tested (93 tests, 43 passing) hexagonal Rust workspace with
  real adapters, real security hygiene, and honest refusals where contracts are
  missing.
- ~5,400 LOC of non-vaporware Rust that compiles clean under `clippy::pedantic`
  on edition 2024 / MSRV 1.93.
- The furthest-along of the four Prometheus Flint planes on architecture
  discipline (hexagonal verification, secret redaction, JWKS single-flight).
- 95.6% through its own OpenSpec plan (43/45 changes done).

The gap between "scaffold" and "production release" is real but bounded. It is
not a research problem — it is a list of named, actionable items, most of which
are in this repo's control.
