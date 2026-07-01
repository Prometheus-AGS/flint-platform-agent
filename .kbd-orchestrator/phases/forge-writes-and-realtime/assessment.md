# Assessment — forge-writes-and-realtime

**Phase:** forge-writes-and-realtime
**Date:** 2026-07-01
**Method:** Inspection of this repo + forge (`fdb-app`, pg_graphql) + fabric (`frf-gateway`).

---

## 0. Bottom line

**Low-risk, unblocked, mostly reuse.** Writes go through the **same forge
`/graphql` endpoint** the agent already calls (c002's `graphql_query`) — forge's
`Quarry` enforces authz (Keto + Cedar PEP) **server-side**, so the agent just
posts the mutation with the operator bearer and forge decides. Fabric health is a
trivial `GET /healthz`. The remaining work is the honest debt paydown from phase 2
(write-kind guard, test hardening).

---

## 1. Verified contracts

### Forge writes (mutations)
- pg_graphql exposes **Query and Mutation** types on `POST /graphql`; forge's
  `fdb-app::Quarry` runs `KetoCheck::check()` then `Pep::check()` (Cedar) before
  the executor, returning a typed 403 (`ForbiddenError`) on denial.
- **Implication:** the agent does **not** replicate the Keto/Cedar gate — it
  forwards the operator bearer and posts the mutation; forge is the authz
  authority. Reuse c002's `graphql_query` helper verbatim (it already sends
  `Authorization: Bearer` and maps 401→Unauthorized; extend mapping for 403).
- Mutation **names** are pg_graphql-generated (`insertInto<Table>Collection`
  convention) and discoverable via introspection — the agent should not hardcode
  a fixed set. **Open question Q1:** build known mutations for `project.create`/
  `application.define`, or proxy operator-supplied GraphQL? (Recommend: pass a
  typed input and build the standard pg_graphql mutation string; keep it thin.)

### Fabric health
- `frf-gateway` `GET /healthz` → `{"status":"ok","version":"…"}`. `fpa-fabric::health`
  becomes a one-liner mirroring c002's forge OpenAPI GET (no bearer needed).
- Subscriptions: `GET /ws/v1/subscribe` (WebSocket). AG-UI feed is the **stretch**
  goal; WS client is more work than health.

---

## 2. Current state of this agent (baseline from phase 2)

| Component | State | Gap for this phase |
|---|---|---|
| `fpa-forge` reads | ✅ real (OpenAPI + GraphQL, bearer→RLS) | Add a **mutation** path reusing `graphql_query`; map forge 403 → `Unauthorized`/`Downstream` |
| `dispatch_forge` (runner) | write kinds fall through to `list_tables` | **Add write-kind guard**; route `project.create`/`application.define` to a mutation call (debt #1) |
| `fpa-fabric::health` | `PortError::Downstream("not implemented")` | Implement `GET /healthz` (carried debt #5, fabric half) |
| Fabric subscriptions | none | Stretch: WS client → AG-UI SSE |
| Test coverage | smoke-verified, some unit gaps | Add unit tests for bearer-carried/none + MCP schema advertisement (debt #2) |

---

## 3. Gap analysis per goal

Legend: ✅ ready · 🟡 partial · ❌ to build

| Goal | Status | Gap |
|---|---|---|
| Forge write mutations (`project.create`/`application.define`) + write-kind guard | ❌ | Add a `graphql_mutation` path (reuse `graphql_query`); build pg_graphql `insertInto…Collection` from typed input; guard remaining write kinds → `Downstream("write API pending")`. Bearer required (no write without operator identity). |
| `fpa-fabric::health` real | ❌ (trivial) | reqwest `GET {fabric}/healthz`; 2xx → `Ok(())`, else `Downstream`; unreachable → `Transport`. Add `reqwest` to `fpa-fabric` + wiremock dev-dep. |
| Fabric subscription → AG-UI (stretch) | ❌ | WS client to `/ws/v1/subscribe`; bridge frames to AG-UI SSE. Larger; keep stretch. |
| Harden phase-2 test gaps | ❌ | Unit tests: `AuthContext` bearer carried / `None`; MCP `tools/list` advertises real per-kind schema. |

---

## 4. Key design decisions (for analyze/spec)

1. **Q1 — mutation build vs proxy.** Recommend the agent **builds** the standard
   pg_graphql mutation from a typed input (`project.create` → `insertIntoProjectsCollection`),
   keeping the catalog/tool contract typed. Proxying raw GraphQL would bypass the
   catalog's input schema + permission model. Decide at spec.
2. **403 mapping.** forge returns 403 on Keto/Cedar denial; map to
   `PortError::Unauthorized` (distinct from a 401 missing-bearer) so the surface
   reports "forbidden by policy" vs "no identity".
3. **Write permission at the agent vs forge.** The catalog already gates
   `project.create`=operator / `application.deploy`=admin. Keep that as a *fast
   local pre-check*; forge's Keto/Cedar remains the **authority**. Don't remove
   the local check — defense in depth — but never treat it as sufficient.

---

## 5. Recommendations / watch items

- **Reuse, don't rebuild:** `graphql_query` already does bearer + error mapping;
  a mutation is the same POST with a mutation string. Keep `fpa-forge` DRY.
- **Health before subscriptions:** land `fpa-fabric::health` (small, closes real
  debt) before attempting the WS subscription (stretch, larger surface).
- **Pay the debt this phase:** the write-kind guard and the phase-2 test gaps are
  first-class goals here, not "if time permits" — they were honest misses.
- **Still no artifact-refiner QA** (subsystem absent) — CI gate + wiremock/smoke
  remain the enforcement; note the same process gap.

---

## 6. Open questions for analyze/plan

- **Q1:** build typed pg_graphql mutations vs proxy operator GraphQL? (Recommend build.)
- **Q2:** exact pg_graphql collection names for `projects`/`applications` — confirm
  from a live forge introspection or fixture at spec/execute.
- **Q3:** is the WS subscription in-scope this phase or explicitly deferred to a
  later realtime phase? (Recommend: health in-scope, WS stretch/deferred.)

---

## 7. Stage handoff

Low-risk phase: writes reuse c002's forge `/graphql` path (forge enforces
Keto/Cedar; agent forwards bearer + posts mutation), fabric health is a trivial
GET. Core work = mutation path + write-kind guard (debt #1) + fabric health
(debt #5) + phase-2 test hardening (debt #2). Open for analyze: build-vs-proxy
mutations (Q1), collection names (Q2), WS scope (Q3).
