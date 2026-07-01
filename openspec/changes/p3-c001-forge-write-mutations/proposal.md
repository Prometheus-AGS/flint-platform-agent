## Why

The agent reads fabric state but cannot write. Forge exposes pg_graphql
**Mutation** types on the same `/graphql` endpoint (enforcing Keto + Cedar
server-side), so writes are unblocked. Also, the phase-2 honest miss: write task
kinds currently fall through to a read (`list_tables`) instead of a proper write
or a clean refusal. This change adds real forge writes and the missing write-kind
guard.

## What Changes

- Add a `graphql_mutation` path to `fpa-forge` (reuse `graphql_query`'s bearer +
  error handling): build a typed pg_graphql mutation string from a typed input.
- Implement `project.create` and `application.define` as forge mutations
  (`insertInto<Table>Collection`), forwarding the operator bearer (RLS + forge's
  Keto/Cedar gate apply server-side — the agent does not replicate authz).
- Map forge **403** (policy denial) → `PortError::Unauthorized` (distinct from 401
  missing-bearer).
- **Write-kind guard:** any write-oriented kind not yet implemented returns
  `PortError::Downstream("write API pending")` — never a silent read fallback
  (closes phase-2 debt #1).
- Keep the catalog's local role pre-check (fast fail); forge remains the authz
  authority (defense in depth).

## Capabilities

### New Capabilities
- `forge-write-mutations`: Typed forge write operations via pg_graphql mutations under the operator bearer, with a clean guard for unimplemented writes.

### Modified Capabilities

## Impact

- `fpa-forge` (mutation path + `dispatch_forge` write routing/guard in `fpa-app`).
- No new dependencies (reuse `reqwest`; `wiremock` dev-dep present).

## Open Questions
- **Q1 (RESOLVED per analyze recommendation):** the agent **builds** typed
  pg_graphql mutations from catalog input (keeps c003 input-schema + local
  permission), not raw-GraphQL proxy.
- **Q2:** exact pg_graphql collection names for `projects`/`applications` — confirm
  via forge introspection/fixture at execute; the mutation builder must accept the
  collection name so it isn't hardcoded wrong.
