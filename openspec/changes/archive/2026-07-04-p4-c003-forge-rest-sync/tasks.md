## 1. Forge REST path

- [x] 1.1 Confirm forge's compiled REST prefix (`/rest/<table>` vs `/<table>`) against `fdb-reflection` / a running forge; add config `FPA_FORGE_REST_PREFIX`.
- [x] 1.2 Add a `rest_insert(base, prefix, table, object, bearer)` helper in `fpa-forge` (reqwest POST with bearer).

## 2. Route writes through REST

- [x] 2.1 `create_entity` → `POST {base}{prefix}/<table>` with the bearer; keep `graphql_exec` available but make REST the primary write path.
- [x] 2.2 Map REST statuses: 2xx (incl 201) → success; 401/403 → `Unauthorized`; 404/other → `Downstream`; unreachable → `Transport`; missing bearer → `Unauthorized` (no request).

## 3. Verification (wiremock)

- [x] 3.1 `cargo check/clippy/fmt` green.
- [x] 3.2 wiremock: authorized REST insert POSTs with bearer + returns result (201).
- [x] 3.3 wiremock: 403 → `Unauthorized`; missing bearer → `Unauthorized`; unreachable → `Transport`.
- [x] 3.4 Runner: `project.create` still routes to a forge write (now REST), guard for unmapped writes intact.
