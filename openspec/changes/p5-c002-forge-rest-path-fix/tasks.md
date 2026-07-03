## 1. Correct the path construction

- [ ] 1.1 Change `rest_insert` to accept `(schema, table)` and build `{base}/{schema}/{table}` (trim trailing slash on base; no `/rest` literal).
- [ ] 1.2 Update `create_entity` signature to take `{schema, table}` (or a small `TableRef { schema, table }`); update the doc comment (currently claims `/rest/<table>`).

## 2. Config + callers

- [ ] 2.1 `FPA_FORGE_REST_PREFIX` → optional path base, default **empty** (forge merges at root). Keep it configurable for future gateway topologies but do not prepend `/rest` by default.
- [ ] 2.2 Update any remaining forge-write callers in `fpa-app` to pass an explicit `{schema, table}`. (`project.create` no longer calls forge writes — see p5-c001.)

## 3. Verification (wiremock)

- [ ] 3.1 `cargo check/clippy/fmt` green.
- [ ] 3.2 wiremock: authorized insert POSTs to `/{schema}/{table}` with bearer, returns result (201).
- [ ] 3.3 wiremock: 403 → `Unauthorized`; missing bearer → `Unauthorized` (no request); unreachable → `Transport`.
- [ ] 3.4 Assert the request path has schema+table segments and no `/rest` segment.
