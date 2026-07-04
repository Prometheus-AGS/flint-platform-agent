## Why

flint-forge advanced (commits `p3-c013`/`p3-c014`) and now serves a **mounted,
Supabase-style dynamic REST CRUD** surface (per-table `GET`/`POST`/`PATCH`/`DELETE`,
Keto+Cedar gated, RLS from bearer). Our `fpa-forge` writes (phase-3) use pg_graphql
`insertInto…Collection` mutations — functional, but forge's first-class REST is
cleaner (proper CRUD verbs, 12 filter operators, `201 Created`). This change syncs
`fpa-forge` to forge's current REST surface.

## What Changes

- Add a REST path to `fpa-forge`: `create_entity` → `POST {base}/<rest-prefix>/<table>`
  with the bearer (forge builds `RlsContext` from it; Keto+Cedar gate server-side).
- Migrate reads where REST is cleaner: `list_tables` may stay on `/openapi.json`
  (metadata), but entity **list/read** can use `GET /<table>` with filters.
- Map forge REST statuses: `201 Created` → success; `401/403` → `Unauthorized`
  (bad token / policy); `404` → `Downstream`; others → `Downstream`; unreachable →
  `Transport`.
- Keep the pg_graphql helper (`graphql_exec`) available but route `project.create`/
  `application.define` through **REST insert** as the primary path.
- **Confirm the exact REST path prefix** (forge's `fdb-reflection` compiles paths
  from the model; Supabase-style `/rest/<table>` per the `Location` header) at
  execute — take it from config, don't hardcode.

## Capabilities

### New Capabilities
- `forge-rest-sync`: `fpa-forge` writes (and entity reads) via forge's REST CRUD surface, keeping the agent in sync with current forge.

### Modified Capabilities

## Impact

- `fpa-forge` (REST client path; `create_entity` routes to REST insert).
- No new deps (reqwest + wiremock present).
- Independent of the gate/auth changes.

## Open Questions
- **REST prefix (execute):** confirm forge's compiled REST path prefix
  (`/rest/<table>` vs `/<table>`) against a running forge or `fdb-reflection`;
  parameterize via config so it's a config fix, not code.
- **Keep both surfaces?** Recommend REST as primary for writes; retain
  `graphql_exec` for reads/queries where GraphQL is a better fit. Confirm at execute.
