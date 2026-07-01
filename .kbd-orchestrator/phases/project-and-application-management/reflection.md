# Reflection — project-and-application-management

**Phase:** project-and-application-management
**Date:** 2026-06-30
**Changes:** 4/4 executed (c001–c004), all CI-green and pushed.

> Written against the sycophancy gate: this names deltas and debt, not just
> wins. The phase delivered a working protocol/orchestration shell but the
> ambitious 9-requirement vision is only partially seeded — most of it was
> correctly scoped OUT of this phase.

---

## 1. Goal achievement

The phase had 5 goals (goals.md). Scored against what actually shipped:

| Goal | Verdict | Evidence |
|---|---|---|
| Wire composition root (adapters → `TaskRunner` as Axum state) | **MET** | c001: config, `AppState`, 4 adapters wired, gate identity extractor; live-verified |
| Define A2A administrative task catalog + dispatch `tools/call`/task submission | **MET** | c003: 8-kind catalog, `TaskRunner` dispatch w/ permission+audit; c004: MCP `tools/call` routes through it |
| Project management capabilities (enumerate/inspect/administer via forge port) | **PARTIAL** | Catalog kinds + Project domain model exist; **actual forge calls are unimplemented** (adapters return `PortError::Downstream`) — blocked on forge gateway |
| Application management (lifecycle across forge/fabric/gate) | **PARTIAL** | Catalogued (`application.define/deploy`) + permission-gated; **no real lifecycle logic** yet |
| Surface management ops as A2UI primitives bound to A2A kinds | **NOT MET (deferred)** | A2UI is forge-owned (RFC-FORGE-A2UI-001); Project model references it but no UI shipped — correctly deferred to a UI phase |

**Overall: ~60% — the foundation goals are MET; the capability goals are
PARTIAL because they are gated on flint-forge, exactly as the assessment warned.**

The broader 9-item functional spec (KB, OpenDesign plugin, React/Vite UI, Tauri,
full auth model) was **deliberately scoped out** this phase (assess Q5) and
remains not started — that was the correct call, not a miss.

---

## 2. Delivered changes

| Change | Delivered | Key artifact |
|---|---|---|
| c001 composition-root | env config, `AppState`, gate-only JWT identity extractor | `fpa-gateway/{config,state,identity}.rs` |
| c002 project-domain-model | `Project` aggregate + child types, versioned JSON Schema (schemars) | `fpa-domain/project/`, `schema/project.schema.json` |
| c003 a2a-task-catalog | 8-kind catalog, `TaskRunner` dispatch, permission-before-port, audit | `fpa-app/{catalog,auth,task_runner}.rs` |
| c004 mcp-transport | MCP server (JSON-RPC → catalog/runner) + reqwest client + skill format | `fpa-gateway/routes/mcp.rs`, `fpa-mcp/lib.rs`, `skills/` |

Commits `e075089`, `a993f55`, `cb4d34a`, `d5cf8da` (+ doc corrections), all on `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/4** |
| First-pass CI-gate pass rate | 4/4 (after in-loop fixes) |
| Changes requiring rework mid-implementation | 2 (c003 MSRV, c004 rmcp reversal) |

**The artifact-refiner QA gate did not run** — the refiner/hooks subsystem
(`$KBD_ORCHESTRATOR_ROOT/shared/lib/`) is **not installed** in this environment,
so no `.refiner/artifacts/` logs exist. Quality was instead enforced by the
project CI gate (`./scripts/ci-check.sh`: fmt + clippy::pedantic -D warnings +
check + test) and live smoke tests, run green per change. This is a **process
gap**, not a quality gap — but it means the formal constraint-violation report is
unavailable for this phase.

---

## 4. Technical debt introduced (carry-forwards)

Explicitly tracked, not hidden:

1. **Adapters are all `PortError::Downstream("not implemented")`** — forge/fabric/gate/mcp-client do no real I/O yet. Intentional (they're blocked on the sibling planes) but the agent can't actually administer anything until they're built.
2. **Per-kind input-schema validation missing** (`AppError::InvalidInput` unused; catalog entries carry no input schema; MCP tools advertise a bare `{"type":"object"}`).
3. **A2A `status`/`cancel` are placeholders** — no in-memory task store; only `submit` runs end-to-end.
4. **MCP client: single endpoint only** (no multi-server composition); **no live round-trip test** (only the empty-endpoint error path).
5. **JWT signature verification is interim** — decodes claims unverified when `FPA_GATE_JWT_KEY` unset; gate's real key/JWKS endpoint not wired.
6. **MCP surface has no gate-identity propagation** — `tools/call` runs as a hardcoded `viewer+operator` context; a real caller identity path is needed.
7. **MSRV jumped 1.85 → 1.93** for one dependency (`a2a-protocol-types` 0.6). Real toolchain-floor cost for the whole workspace + any consumer.
8. **`frf-agentproto` `proto-v1` tag is unpushed** on the fabric remote — a git-dep on it would break CI; avoided this phase but a latent blocker.

---

## 5. Lessons captured (for the knowledge base)

- **Verify a dependency's MSRV before adopting, not after** (Base Rule 22). `a2a-protocol-types` 0.6 silently required rustc 1.93; caught only at `cargo add`. Cost: a workspace-wide MSRV bump. Lesson: run `cargo add --dry-run` and read `rust-version` during *analyze*, not *execute*.
- **Consult the house skill before choosing an external crate.** The analyze/spec stage picked `rmcp` for the MCP server; the canonical `mcp-server` skill mandates hand-rolled JSON-RPC over Axum. The reversal wasted a build cycle. Lesson: load the relevant `*-server`/pattern skill during analyze, not execute.
- **`#[non_exhaustive]` cross-crate matching needs a wildcard** — recurred in AG-UI, `AppError`, and A2A mapping. Confirms the forward-compat contract works; budget for the wildcard arm every time.
- **`todo!()` in a request-path adapter panics the request.** Replaced all four with `PortError::Downstream`. Lesson: scaffolding on a live path should return handled errors from day one, never `todo!()`.
- **Env-var config is untestable in parallel** — refactored `from_env` into a pure `from_lookup(closure)`. Reusable pattern: separate parsing from the env read.
- **The A2UI ownership and gate-only-auth corrections came from the user, mid-flight** — the original CLAUDE.md got both wrong. Lesson: cross-repo ownership claims must be verified against the owning repo's RFCs during assess, not asserted.

---

## 6. Recommended Next Phase

**`forge-integration-and-real-dispatch`** — turn the shell into a working
administrator by implementing the adapters against flint-forge, closing the
highest-value carry-forwards. Rationale: everything PARTIAL above is gated on
this; nothing else (UI, OpenDesign, Tauri) delivers value until the agent can
actually read/administer the fabric.

Scope (proposed):
1. Implement `fpa-forge` against forge's REST/GraphQL contract (start read-only:
   `list_tables`/`describe_table` → real `project.list`/`inspect`).
2. Wire per-kind **input-schema validation** into the catalog (closes debt #2).
3. Add the in-memory **task store** so `status`/`cancel` work (debt #3).
4. Propagate **gate identity into the MCP surface** (debt #6).
5. Resolve the **`frf-agentproto` tag** (push `proto-v1` or pin a `main` SHA) if
   fabric parity is needed (debt #8).

**Prerequisite / open question:** is flint-forge's `fdb-gateway` far enough along
to integrate against, or do we build a mock forge to develop against first? This
gates whether next phase is "integrate" or "mock-and-integrate-later."

Defer still: OpenDesign plugin (#6), React/Vite UI + generator (#7), Tauri (#8),
knowledge base (#5) — all remain blocked or lower-priority than real dispatch.

---

## 7. Reflect handoff

Deltas: foundation MET, capabilities PARTIAL (all gated on flint-forge, as
predicted); 8 tracked debts, notably all-adapters-unimplemented and a
workspace MSRV bump. Corrective actions: next phase = real forge integration +
close input-validation/task-store/identity-propagation debts. Recommended next
phase: **forge-integration-and-real-dispatch** — pending confirmation of forge
gateway readiness.
