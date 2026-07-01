# Assessment — forge-integration-and-real-dispatch

**Phase:** forge-integration-and-real-dispatch
**Date:** 2026-06-30
**Method:** Inspection of this repo + flint-forge's now-implemented `fdb-gateway`.

---

## 0. Bottom line

**This phase is genuinely unblocked.** The previous phase's central risk (forge
gateway is a stub) is **gone** — flint-forge's Quarry gateway (`flint-quarry`,
`:8080`) now serves real routes with a clear bearer→RLS contract. `fpa-forge` can
be implemented against a concrete HTTP surface. The work is real integration
code, not scaffolding.

The one genuine unknown is **local testability** (does forge run locally with
Postgres 18?), which determines whether we integration-test against a live forge
or against recorded fixtures this phase.

---

## 1. Forge's real surface (verified contract)

`flint-forge/crates/fdb-gateway/src/main.rs` (311 lines, **zero `todo!()`**):

| Route | Auth | Use for |
|---|---|---|
| `GET /healthz` | none | liveness + `schema_version` |
| `GET /openapi.json` | **none** (public, Supabase-style) | **table/schema metadata** → `list_tables` / `describe_table` |
| `POST /graphql` (+ WS) | **`Authorization: Bearer` → `rls_from_bearer`** | data queries under RLS → real `project.*` reads |
| `POST /rpc/vector` | bearer → RLS | vector search (later; KB phase) |

**Bearer→RLS contract (confirmed in source):** `/graphql` and `/rpc/vector`
extract the bearer, call `rls_from_bearer(&bearer)` → 401 on missing/invalid.
Request body is standard GraphQL `{ query, variables, operationName }`.

**This is exactly the gate-only-auth model:** `fpa-forge` forwards the operator's
**gate-issued JWT** as the bearer; forge verifies it and applies RLS. The agent
fabricates nothing (Base Rule 33; the c001 gate-identity work already captures the
raw token path — need to thread it through).

---

## 2. Current state of this agent (baseline from phase 1)

| Component | State | Gap for this phase |
|---|---|---|
| `fpa-forge` adapter | `ForgeMetadata::{list_tables,describe_table}` return `PortError::Downstream("not implemented")` | **Implement against `/openapi.json` (metadata) + `/graphql` (data).** No HTTP client yet (`ForgeAdapter` holds only `base_url`) |
| Gate identity (`fpa-gateway`) | Extracts `OperatorContext` from gate JWT; **raw token not retained** | Need to carry the raw bearer through to `fpa-forge` for forwarding |
| `TaskRunner` dispatch | Routes `project.list`→`list_tables`, `project.inspect`→`describe_table` | Works; just needs the adapter to do real I/O + receive the bearer |
| Catalog input validation | `AppError::InvalidInput` unused; entries carry no input schema | **Add per-kind input schemas + validation** (debt #2) |
| A2A `status`/`cancel` | Placeholders; no task store | **Add in-memory task store** (debt #3) |
| MCP `tools/call` identity | Hardcoded `viewer+operator` | **Propagate real gate identity** (debt #6) |

---

## 3. Gap analysis per goal

Legend: ✅ ready · 🟡 partial · ❌ to build

| Goal | Status | Gap |
|---|---|---|
| Implement `fpa-forge` (read-only) against forge HTTP | ❌ | Add `reqwest` client to `ForgeAdapter`; `list_tables` ← `GET /openapi.json` (parse compiled schema); `describe_table` ← filter that doc or a GraphQL introspection/query. **Design choice: metadata via OpenAPI vs GraphQL introspection — decide at analyze/spec.** |
| Forward gate JWT as bearer for RLS | 🟡 | c001 verifies the token but discards the raw string. Thread the raw bearer from `OperatorContext` → `AuthContext` → `TaskRunner` → `fpa-forge`. Requires a port-signature change (the runner/ports currently pass no credential). **Cross-cutting.** |
| Per-kind input-schema validation | ❌ | Extend `CatalogEntry` with an input JSON Schema; validate in `TaskRunner::run` before dispatch → `AppError::InvalidInput`; surface real `inputSchema` in MCP `tools/list`. |
| In-memory task store (`status`/`cancel`) | ❌ | Add a task store to `AppState`/app layer keyed by `TaskId`; `submit` records, `status` reads, `cancel` transitions. Concurrency: `Arc<RwLock<HashMap<…>>>` or a dedicated store type. |
| Propagate gate identity into MCP | 🟡 | MCP `tools/call` must build `AuthContext` from the caller's gate identity, not a constant. Depends on how MCP callers present identity (header on `POST /mcp`?) — **open question.** |

---

## 4. Key design decisions to surface (for analyze/spec)

1. **Credential threading is the crux.** Forwarding the bearer to forge means the
   ports must carry a per-request credential. Options: (a) add a `bearer:
   Option<String>` to `AuthContext` and pass it through `TaskRunner::run` to the
   forge port method; (b) a request-scoped context object. This touches
   `fpa-app` + `fpa-ports` + `fpa-forge` + both surfaces. **Do this deliberately.**
2. **Metadata source:** `/openapi.json` (public, simplest, no auth) vs GraphQL
   introspection (needs bearer, richer). Recommend OpenAPI for `list_tables`
   (public + already compiled), GraphQL for actual data reads.
3. **MCP identity:** does the MCP transport carry a gate JWT (Authorization header
   on `POST /mcp`)? If yes, reuse the c001 extractor. If MCP hosts can't send one,
   the MCP surface needs its own trust model. **Confirm.**

---

## 5. Recommendations / things to watch

- **Don't over-reach on writes.** Keep this phase **read-only** (`project.list`,
  `project.inspect`, `forge.table.*`). `project.create`/`application.deploy` need
  forge write APIs (GraphQL mutations under RLS) — a follow-on.
- **Reuse the sibling HTTP+JWT stack** already adopted (`reqwest` 0.12 rustls +
  `jsonwebtoken` 9) — no new deps expected this phase.
- **RLS is forge's job, not ours** — never construct RLS context here; only
  forward the verified bearer. Re-confirm nothing logs the token (c003 audit fix
  already covers the runner path; extend to the forge adapter).
- **Testing without live forge:** if forge won't run locally, use `wiremock`/
  recorded fixtures for `fpa-forge` unit tests. Decide at analyze.

---

## 6. Open questions for analyze/plan

- **Q1.** Is forge's `fdb-gateway` runnable locally (Postgres 18 + sqlx + a seeded
  schema), or do we test `fpa-forge` against fixtures this phase?
- **Q2.** Credential threading shape — add `bearer` to `AuthContext`, or a
  separate request-context? (Determines the port signature change.)
- **Q3.** Metadata via `/openapi.json` vs GraphQL introspection for
  `describe_table`.
- **Q4.** Do MCP `tools/call` callers present a gate JWT on `POST /mcp`? Gates how
  debt #6 (MCP identity) is closed.

---

## 7. Stage handoff

Phase is unblocked: forge's Quarry gateway is real (OpenAPI + bearer→RLS GraphQL).
Core work = implement `fpa-forge` against it + thread the operator's gate bearer
through the ports (the cross-cutting crux), plus close input-validation, task-store,
and MCP-identity debts. Open for analyze/plan: local-testability (Q1), credential
threading shape (Q2), metadata source (Q3), MCP caller identity (Q4).
