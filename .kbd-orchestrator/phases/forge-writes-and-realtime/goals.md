# Goals — forge-writes-and-realtime

Seeded from `forge-integration-and-real-dispatch/reflection.md` §6.

**Context verified (2026-07-01):** both prerequisites from the reflection are
resolved against sibling source:
- **Forge mutations exist** — pg_graphql provides Query **and Mutation** types;
  forge's `fdb-app` runs a Keto gate + PEP check before delegating mutations. So
  writes = GraphQL mutations under RLS. **Unblocked.**
- **Fabric endpoint confirmed** — `frf-gateway` serves `GET /healthz`,
  `GET /ws/v1/subscribe` (WS subscriptions), + gRPC (agent/signal/sync). So
  `fpa-fabric::health` → `/healthz`; subscriptions → `/ws/v1/subscribe`.

## Goals

- Implement forge **write** operations via GraphQL mutations (bearer → RLS) for
  `project.create` / `application.define`; **add the explicit write-kind guard**
  so unimplemented writes fail cleanly (closes reflection debt #1).
- Implement `fpa-fabric::health` against `GET {fabric}/healthz` (real liveness),
  replacing the `PortError::Downstream` stub (closes carried debt #5, fabric half).
- (Stretch) a fabric subscription over `/ws/v1/subscribe` that feeds AG-UI SSE.
- **Harden test coverage** for the phase-2 checkbox-overstated items (debt #2):
  unit tests for bearer-carried/none and MCP `tools/list` schema advertisement.

## Deferred (still out of scope)

gate admin real adapter, OpenDesign plugin, React/Vite UI + generator, Tauri,
knowledge base, full JWT/JWKS signature verification, durable task store.

## Open question (resolve at assess)

Are forge's create mutations named/shaped per-entity by pg_graphql (e.g.
`insertIntoProjectsCollection`), and does the agent need the exact mutation names
or can it pass through operator-supplied GraphQL? (Determines whether the agent
builds mutations or proxies them.)
