## 1. HTTP client + deps

- [x] 1.1 Add `reqwest` (workspace) to `fpa-forge`; give `ForgeAdapter` a `reqwest::Client`.
- [x] 1.2 Add `wiremock` 0.6 as a dev-dependency for `fpa-forge` (per analysis).

## 2. Metadata reads (no bearer)

- [x] 2.1 `list_tables` ← `GET {base_url}/openapi.json`; parse the compiled doc → table/entity list.
- [x] 2.2 `describe_table` ← extract the named table from the OpenAPI doc (Q3: fall back to GraphQL introspection only if OpenAPI is insufficient; note the decision).
- [x] 2.3 Map transport/decoding failures onto `PortError::{Transport,Decode}`.

## 3. Data reads (bearer → RLS)

- [x] 3.1 Add a GraphQL helper: POST `{base_url}/graphql` with `{query, variables, operationName}` + `Authorization: Bearer` from the threaded credential (p2-c001).
- [x] 3.2 Map forge 401 → `PortError::Unauthorized`; other errors → `Downstream`.
- [x] 3.3 Write-oriented kinds return `PortError::Downstream("write API pending")` — no mutation.

## 4. Verification (wiremock fixtures)

- [x] 4.1 `cargo check/clippy/fmt` green.
- [x] 4.2 wiremock test: `list_tables` parses a sample `/openapi.json` into the expected list.
- [x] 4.3 wiremock test: a data read sends the bearer and returns the mocked GraphQL result.
- [x] 4.4 wiremock test: forge 401 → `PortError::Unauthorized`; forge down → `PortError::Transport`.
- [ ] 4.5 (Optional, manual) live-forge smoke against a locally-run Quarry if available.
