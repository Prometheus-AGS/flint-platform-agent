## 1. Mutation path in fpa-forge

- [ ] 1.1 Add `graphql_mutation(http, base_url, bearer, collection, input) -> Result<Value, PortError>` reusing `graphql_query`'s bearer + status handling.
- [ ] 1.2 Build the pg_graphql `insertInto<Collection>Collection` mutation string from the collection name + typed input (collection name is a parameter, not hardcoded).
- [ ] 1.3 Map forge 403 → `PortError::Unauthorized("policy denied")`; 401 → `Unauthorized`; other non-2xx → `Downstream`.

## 2. Wire write kinds + guard

- [ ] 2.1 In `fpa-app` `dispatch_forge`, route `project.create` / `application.define` to the mutation path (map catalog input → collection + fields).
- [ ] 2.2 Add the write-kind guard: any write kind not mapped returns `PortError::Downstream("write API pending")` — remove the silent read fallback for writes.
- [ ] 2.3 Keep the catalog role pre-check (operator/admin) as a fast local gate; forge remains authoritative.

## 3. Verification (wiremock)

- [ ] 3.1 `cargo check/clippy/fmt` green.
- [ ] 3.2 wiremock: authorized mutation sends the bearer + returns forge result.
- [ ] 3.3 wiremock: forge 403 → `Unauthorized` (policy); missing bearer → `Unauthorized` (no request).
- [ ] 3.4 Test: an unmapped write kind → `Downstream("write API pending")`, no read call.
