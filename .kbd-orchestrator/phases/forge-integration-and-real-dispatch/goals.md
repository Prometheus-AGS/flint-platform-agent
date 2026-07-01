# Goals — forge-integration-and-real-dispatch

Seeded from `project-and-application-management/reflection.md` §6.

**Context update (verified 2026-06-30):** flint-forge's `fdb-gateway` is no longer
a stub — it exposes real routes (`/healthz`, `/openapi.json`, `/rpc/vector`,
GraphQL, RLS-from-bearer, `PgGraphQl`/`PgVectorRpc`) with zero `todo!()`. So this
phase is **integrate**, not mock-then-integrate.

## Goals

- Implement `fpa-forge` against forge's real HTTP surface (start read-only):
  wire `list_tables`/`describe_table` to forge GraphQL / `/openapi.json` so
  `project.list` and `project.inspect` return real fabric data (closes debt #1, partial).
- Forward the operator's gate-issued JWT as the bearer to forge (forge does
  `rls_from_bearer`), so RLS applies — no claim fabrication (Base Rule 33).
- Add per-kind **input-schema validation** to the catalog; exercise
  `AppError::InvalidInput`; advertise real input schemas in MCP `tools/list`
  (closes debt #2).
- Add an in-memory **task store** so A2A `status`/`cancel` reflect real state
  (closes debt #3).
- Propagate **gate identity into the MCP surface** (`tools/call` runs as the real
  caller, not a hardcoded viewer+operator) (closes debt #6).

## Deferred (still out of scope)

Fabric/gate real adapters, OpenDesign plugin (#6), React/Vite UI + generator (#7),
Tauri (#8), knowledge base (#5), full JWT signature verification against gate's
JWKS (debt #5 — revisit when gate's key endpoint is confirmed).

## Open question (resolve at assess)

Is forge's `fdb-gateway` deployable locally (needs Postgres 18 + sqlx) for
integration testing, or do we test `fpa-forge` against recorded fixtures this phase?
