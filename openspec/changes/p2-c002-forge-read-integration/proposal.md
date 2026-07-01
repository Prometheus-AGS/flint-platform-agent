## Why

`fpa-forge` returns `PortError::Downstream("not implemented")` for everything, so
the agent cannot actually read fabric state. Forge's Quarry gateway is now real
(`GET /openapi.json` for metadata, `POST /graphql` for RLS data). This change
implements `fpa-forge` against that surface so `project.list` / `project.inspect`
/ `forge.table.*` return real data.

## What Changes

- Give `ForgeAdapter` a `reqwest` client.
- `list_tables` ← `GET {forge}/openapi.json` (public, pre-compiled schema); extract the table/entity list.
- `describe_table` ← the OpenAPI doc for the named table (fall back to a GraphQL introspection query only if OpenAPI lacks needed detail — Q3).
- Data reads (where required) ← `POST {forge}/graphql` with `{query, variables, operationName}`, forwarding the operator's bearer (`Authorization: Bearer`) from `p2-c001`.
- Map forge failures onto `PortError` (`Unauthorized` on 401, `Downstream` on other errors, `Decode` on bad bodies).
- **Read-only this phase** — no forge mutations (`project.create`/`application.deploy` stay `Downstream("write API pending")`).

## Capabilities

### New Capabilities
- `forge-read-integration`: `fpa-forge` reads fabric metadata (OpenAPI) and data (GraphQL under RLS) from forge's Quarry gateway, forwarding the operator bearer.

### Modified Capabilities

## Impact

- `fpa-forge` (reqwest client + real method bodies), `fpa-forge/Cargo.toml` (add `reqwest`; add `wiremock` dev-dep).
- Depends on `p2-c001` (bearer forwarding).
- No runtime deps beyond `reqwest` (already in the workspace).

## Open Questions
- **Q3:** `/openapi.json` vs GraphQL introspection for `describe_table`. Recommend OpenAPI first; introspection only if insufficient.
- **Q1:** tests run against `wiremock` fixtures (no live forge required); a live-forge smoke test is optional/manual (needs Postgres 18).
