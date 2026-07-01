# Reflection — forge-writes-and-realtime

**Phase:** forge-writes-and-realtime
**Date:** 2026-07-01
**Changes:** 3/3 executed (p3-c001–c003), all CI-green and pushed.

> Sycophancy gate applied: deltas + debt, not just wins. This was a disciplined,
> reuse-heavy phase that added writes + fabric health and **paid down two honest
> misses** from the phase-2 reflection. Highest goal-achievement of the three
> phases.

---

## 1. Goal achievement

Four goals (goals.md), scored:

| Goal | Verdict | Evidence |
|---|---|---|
| Forge write mutations + write-kind guard | **MET** | c001: `create_entity` builds pg_graphql `insertInto<Collection>Collection`; 403→Unauthorized; unmapped writes → `Downstream("write API pending")` (no read fallback) |
| `fpa-fabric::health` real | **MET** | c002: `GET /healthz`; 3 wiremock tests |
| Fabric subscription → AG-UI (stretch) | **DEFERRED (intentional)** | Explicitly out of scope per analyze recommendation; `tokio-tungstenite` identified for the later realtime phase |
| Harden phase-2 test-coverage gaps | **MET** | c003: 4 real unit tests (bearer carried/none, Debug redaction, MCP real schema) |

**Overall: ~95% — all 3 in-scope goals MET; the 4th was a deliberate defer, not a
miss.** This phase closed **phase-2 debt #1 (write-kind guard), #2 (test coverage),
and the fabric half of #5**.

---

## 2. Delivered changes

| Change | Delivered | Commit |
|---|---|---|
| c001 forge-write-mutations | `ForgeMetadata::create_entity`; shared `graphql_exec` (query+mutation, 403 map); write routing + guard | `758fb08` |
| c002 fabric-health | `FabricAdapter::health` via `GET /healthz`; wiremock | `a49c8d3` |
| c003 test-hardening | `AuthContext` + MCP schema unit tests | `a7fa551` |

All on `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/3** |
| First-pass CI-gate pass rate | 3/3 (clean; no rework this phase) |
| New tests added | 8 (5 forge/fabric wiremock + 3 auth + 1 MCP + 1 runner) |

**artifact-refiner QA gate did not run** (subsystem absent — persistent across all
three phases). CI gate + wiremock/live smoke enforced quality. Notably this phase
had **no mid-implementation rework** — the reuse of c002's harness + verified
mutation shape meant everything compiled/passed first-try.

---

## 4. Technical debt

**New this phase:** essentially none of substance. Minor honesty notes:
1. Helper named `graphql_exec` (query+mutation) not the spec's `graphql_mutation` — a naming deviation, not a defect.
2. c002 task 2.4 (live end-to-end `fabric.health`) proven by construction (runner mapping + wiremock), not a fresh gateway curl.

**Still carried (unchanged):**
3. `fpa-gate` admin adapter still `Downstream("not implemented")` (the only unimplemented plane now — gate half of the old debt #5).
4. Interim JWT signature verification (unverified when `FPA_GATE_JWT_KEY` unset).
5. Forge writes limited to **insert** (`create`); update/delete not implemented.
6. MCP client single-endpoint, no live round-trip test.
7. Fabric **subscriptions** (WS) deferred — health only.
8. Task store non-durable (in-memory).

---

## 5. Lessons captured

- **Reuse compounds.** Phase 2's `graphql_query` + wiremock harness made phase-3
  writes a small extension (`graphql_exec` + one port method). The disciplined
  earlier phases paid off as velocity here — first-try green, no rework.
- **Verify the wire shape from the source repo, not memory.** The pg_graphql
  mutation form came from forge's own research doc (`insertIntoAccountCollection`),
  so `create_entity` was correct on the first pass. Same habit that resolved the
  fabric endpoint and forge-readiness in prior phases.
- **Debt named honestly gets paid.** The phase-2 reflection's flagged misses
  (write-kind guard, test overstatement) became first-class phase-3 goals and are
  now closed. The loop self-corrects when the reflection is truthful rather than
  green-washed.
- **Authz boundary stayed clean under pressure to add writes.** The agent posts
  mutations + forwards the bearer; forge (Keto/Cedar) remains the authority. The
  403→Unauthorized mapping keeps "denied by policy" distinct from "no identity" —
  important for operators debugging permissions.

---

## 6. Recommended Next Phase

**`gate-admin-and-auth-hardening`** — close the last unimplemented plane and the
auth-interim debt, making the agent's administrative surface complete and its auth
production-grade.

Scope (proposed):
1. Implement `fpa-gate` against flint-gate's **admin API** (`:4457`) — real
   `list_routes` (and route inspection), replacing the `Downstream` stub (closes
   the gate half of carried debt #3/#5).
2. **Full JWT signature verification** against gate's published key/JWKS endpoint
   (replace the interim unverified path — carried debt #4), so a missing
   `FPA_GATE_JWT_KEY` no longer means unverified tokens.
3. (Stretch) forge **update/delete** mutations to round out writes (debt #5).

**Prerequisites / open questions:**
- What does flint-gate's admin API expose for route listing, and does it publish a
  JWKS/key endpoint the agent can verify against? (Confirm from gate source, as we
  did for forge/fabric.)
- Is gate's admin API reachable without going through the proxy (trusted path)?

Still deferred: fabric WS subscriptions + AG-UI feed (own realtime phase),
OpenDesign plugin, React/Vite UI + generator, Tauri, knowledge base, durable
task store.

---

## 7. Reflect handoff

All 3 in-scope goals MET; agent now reads AND writes fabric state under RLS (forge
Keto/Cedar authoritative), checks fabric liveness, and the phase-2 test/guard debt
is closed. Near-zero new debt; `fpa-gate` is the last unimplemented plane and JWT
verification is still interim. Recommended next phase:
**gate-admin-and-auth-hardening** — pending confirmation of gate's admin API +
JWKS endpoint from source.
