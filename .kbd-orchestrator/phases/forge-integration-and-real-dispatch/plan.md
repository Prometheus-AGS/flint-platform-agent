# Plan — forge-integration-and-real-dispatch

**Phase:** forge-integration-and-real-dispatch
**Date:** 2026-06-30
**Backend:** OpenSpec (4 changes scaffolded + validated)
**Scope:** read-only forge integration + close 3 phase-1 debts

> Ordered change list the Execute stage drives one task per turn. Library
> annotations from `library-candidates.json`; ordering follows the proposal
> dependency edges.

---

## Dependency graph

```
p2-c001-credential-threading ──► p2-c002-forge-read-integration
p2-c003-catalog-input-validation   (independent)
p2-c004-task-store                 (independent)
```

- **c001** is the root — c002 cannot forward a bearer to forge without it.
- **c002** depends on c001.
- **c003** and **c004** are independent of both and of each other (parallelizable).

Linear order for a single developer: **c001 → c002 → c003 → c004** (c003/c004
could interleave or run first; placed after the forge chain since they're smaller
debt-closers).

---

## Ordered change list

### 1. `p2-c001-credential-threading`  ·  root
**Goal:** Thread the operator's gate bearer surfaces → `TaskRunner` → ports; propagate MCP caller identity; never log the token.
- **Library:** none (internal design). **Decision: `AuthContext.bearer`** (not a separate request-context).
- **Risk:** medium — touches `fpa-app` + `fpa-ports` + `fpa-gateway` (cross-cutting signature change). Mitigation: small, additive `Option<String>` field.
- **Recommended agent:** general Rust + `rust-reviewer`; `security-reviewer` for the no-log-token guarantee.
- **Open Q at execute:** does `POST /mcp` carry a gate JWT (Q4)? If not, MCP keeps a scoped default.
- **Gate to next:** bearer flows to the forge dispatch path; no-token-in-logs test passes.

### 2. `p2-c002-forge-read-integration`  ·  needs c001
**Goal:** Implement `fpa-forge` against forge's `/openapi.json` (metadata) + `/graphql` (bearer→RLS data). Read-only.
- **Library:** `reqwest` (already present) — adopt; `wiremock` 0.6 — adopt (dev-dep, gate precedent) for fixtures.
- **Risk:** medium — real HTTP integration + error mapping (401→Unauthorized). Mitigation: wiremock fixtures, no live forge needed.
- **Recommended agent:** general Rust + `docs-lookup` (forge OpenAPI/GraphQL shape); `rust-reviewer`.
- **Open Q at execute:** OpenAPI vs GraphQL introspection for `describe_table` (Q3).
- **Gate to next:** `list_tables`/data-read/401/down wiremock tests pass.

### 3. `p2-c003-catalog-input-validation`  ·  independent
**Goal:** Per-kind input schemas on the catalog → `AppError::InvalidInput`; real MCP `tools/list` schemas.
- **Library:** possibly `jsonschema` (verify MSRV/version at execute — the phase-1 lesson) OR minimal hand-rolled required-field check (preferred).
- **Risk:** low. Mitigation: prefer the minimal check; only add a validator crate if justified.
- **Recommended agent:** general Rust + `tdd-guide`; `rust-reviewer`.
- **Gate:** invalid-input rejected (no port call) + valid passes + tools/list schema tests.

### 4. `p2-c004-task-store`  ·  independent
**Goal:** In-memory `RwLock<HashMap>` task store; A2A `status`/`cancel` reflect real state.
- **Library:** none (`tokio` present).
- **Risk:** low.
- **Recommended agent:** general Rust + `tdd-guide`; `rust-reviewer`.
- **Gate to phase reflect:** submit→status, unknown→404, cancel-non-terminal/terminal tests pass.

---

## Cross-cutting (every change)

- Hexagonal rule intact: domain/app import no adapter; wiring only in `fpa-gateway`.
- CI gate green per change (`./scripts/ci-check.sh`).
- **Never log the bearer or claims** (extend the c003 audit-skip discipline).
- Read-only: no forge mutations this phase.
- One commit per change (or per task group).

## Open decisions to resolve at execute (do not silently pick)
1. **c001:** MCP caller identity — does `POST /mcp` carry a gate JWT? (Q4)
2. **c002:** `describe_table` via OpenAPI vs GraphQL introspection. (Q3)
3. **c003:** minimal shape-check vs `jsonschema` crate (verify MSRV first — phase-1 lesson).

## Execute order

```
1) apply p2-c001-credential-threading
2) apply p2-c002-forge-read-integration
3) apply p2-c003-catalog-input-validation
4) apply p2-c004-task-store
```

First change to apply: **`p2-c001-credential-threading`**.
