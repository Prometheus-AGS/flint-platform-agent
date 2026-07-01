# Analysis — forge-writes-and-realtime

**Phase:** forge-writes-and-realtime
**Date:** 2026-07-01
**Mode:** stack-specified
**Inputs:** `assessment.md`.

> Minimal analyze phase. Writes and fabric-health are **reuse** of existing code
> and deps; the only genuinely new library question is the WebSocket client for
> the **stretch** subscription goal.

## 1. Landscape summary

Nothing new is required for the core goals:

- **Forge writes** = the same `POST /graphql` + bearer path from c002. A mutation
  is a query with a mutation string; `graphql_query` already does bearer + error
  mapping. Only addition: map forge **403** (Keto/Cedar policy denial) →
  `PortError::Unauthorized`, distinct from 401 (missing bearer). Forge enforces
  authz server-side; the agent forwards and posts — it does **not** replicate the
  Keto/Cedar gate.
- **Fabric health** = `reqwest GET /healthz` (present dep), mirroring c002's forge
  OpenAPI GET. `wiremock` (dev-dep, already MSRV-verified) covers tests.

The one new-dependency question — a **WS client** for the stretch subscription —
resolves on a strong consistency signal: **`tokio-tungstenite`** is used by both
`flint-realtime-fabric` (frf-gateway, atproto bridge) and `flint-gate`
(client/core). If the WS goal is pulled in, that's the house pick (siblings pin
0.26; latest 0.29 — reconcile at execute). Recommendation: **defer WS**; land the
real debt (writes + health + tests) first.

## 2. Build-vs-adopt calls

| Gap | Verdict | Confidence |
|---|---|---|
| Forge write mutations | **Reuse `graphql_query`** (no new dep); map 403→Unauthorized | high |
| Fabric health | **Reuse `reqwest` GET /healthz** (+ wiremock dev-dep) | high |
| WS subscription (stretch) | `tokio-tungstenite` **if in-scope**; recommend defer | medium |
| Write-kind guard + test hardening | **Build** (internal, no dep) | high |

## 3. The decision that matters (for spec)

**Q1 — build typed mutations vs proxy raw GraphQL.** Recommend the agent **builds**
the standard pg_graphql mutation (`project.create` → `insertIntoProjectsCollection`)
from a typed catalog input. Proxying raw operator GraphQL would bypass the
catalog's input-schema validation (c003) and the local permission pre-check —
losing the typed contract. Building keeps the tool/catalog model intact while
forge remains the authz authority. **Decide at spec.**

## 4. Evidence (tiered)

- **Tier 3 (cargo search):** `tokio-tungstenite` 0.29.0 current.
- **Tier 3 (sibling inspection):** `tokio-tungstenite` in fabric `frf-gateway`
  (0.26) + atproto bridge, and gate client/core → house-standard WS client.
  Forge writes confirmed to reuse `/graphql` (pg_graphql Mutation + Quarry
  Keto/Cedar gate). Fabric `/healthz` → `{status,version}`.
- **Tiers 1/2/4:** not needed (integration against known contracts).

## 5. Open questions (carried to spec)

1. **Q1:** build typed pg_graphql mutations (recommended) vs proxy raw GraphQL.
2. **Q2:** exact pg_graphql collection names for `projects`/`applications` —
   confirm via forge introspection or fixture at spec/execute.
3. **Q3:** WS subscription in-scope or deferred (recommend defer; if kept, adopt
   `tokio-tungstenite`).

## 6. Handoff to Spec

Adopt set is empty for the core goals (all reuse). Spec must decide Q1
(build-vs-proxy mutations), keep the phase focused on writes + fabric health +
phase-2 test hardening, and treat the WS subscription as an explicit stretch/defer
(with `tokio-tungstenite` as the pick if pulled in).
