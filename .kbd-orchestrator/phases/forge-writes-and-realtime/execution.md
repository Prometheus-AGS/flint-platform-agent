# Execution — forge-writes-and-realtime

**Backend:** openspec. **Changes:** 0/3.

## Dispatch order (all independent)
1. p3-c001-forge-write-mutations  ← first
2. p3-c002-fabric-health
3. p3-c003-test-hardening

Per change: implement → ./scripts/ci-check.sh green → wiremock/smoke → mark DONE → next.

## Pre-flight (resolved)
- **Q2 pg_graphql mutation shape (confirmed from forge):**
  `mutation { insertInto<Table>Collection(objects: [{...}]) { records { <cols> } } }`
  (forge research doc shows `insertIntoAccountCollection(objects: [...])`). Builder
  takes a collection name + objects; forge passes to pg_graphql under RLS with
  Keto/Cedar gating server-side. No hardcoded collection set.
- No new runtime deps (reuse reqwest/graphql_query); wiremock dev-dep present pattern.
- WS subscription deferred (not in these 3 changes).

## First pending change
p3-c001-forge-write-mutations.
