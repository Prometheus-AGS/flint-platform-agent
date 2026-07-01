# Plan — forge-writes-and-realtime

**Phase:** forge-writes-and-realtime
**Date:** 2026-07-01
**Backend:** OpenSpec (3 changes scaffolded + validated)
**Scope:** forge writes + fabric health + phase-2 debt paydown (WS deferred)

> Ordered change list the Execute stage drives one task per turn. Library
> annotations from `library-candidates.json`. All three changes are independent —
> ordering is by value/risk, not dependency.

---

## Dependency graph

```
p3-c001-forge-write-mutations   (independent)
p3-c002-fabric-health           (independent)
p3-c003-test-hardening          (independent)
```

No inter-change dependencies. Any order is valid; the sequence below is
highest-value-first (real writes), then the trivial health win, then the
test-debt paydown.

---

## Ordered change list

### 1. `p3-c001-forge-write-mutations`  ·  highest value
**Goal:** Typed forge writes (`project.create` / `application.define`) via pg_graphql mutations under the operator bearer; write-kind guard.
- **Library:** reuse `graphql_query` (reqwest, present) — no new dep; `wiremock` dev-dep present.
- **Risk:** medium — mutation string construction + 403 mapping + guard. Mitigation: reuse c002's bearer/error path; wiremock fixtures.
- **Decision applied:** build typed mutations (Q1); forge remains authz authority (Keto/Cedar), agent forwards bearer.
- **Recommended agent:** general Rust + `docs-lookup` (pg_graphql mutation shape); `rust-reviewer` + `security-reviewer` (authz path).
- **Open Q at execute:** exact pg_graphql collection names (Q2) — confirm via forge introspection/fixture; builder takes the name as a parameter.
- **Gate:** wiremock authorized-mutation / 403→Unauthorized / missing-bearer / unmapped-write-guard tests pass.

### 2. `p3-c002-fabric-health`  ·  small, closes real debt
**Goal:** `fpa-fabric::health` ← `GET /healthz`.
- **Library:** reuse `reqwest` (present); add `wiremock` dev-dep. No new runtime dep.
- **Risk:** low — near-copy of c002's forge OpenAPI GET.
- **Recommended agent:** general Rust + `tdd-guide`; `rust-reviewer`.
- **Gate:** wiremock 200→Ok / 503→Downstream / unreachable→Transport; `fabric.health` task returns `{"ok":true}` end-to-end.

### 3. `p3-c003-test-hardening`  ·  debt paydown
**Goal:** Unit tests for bearer-carried/none, MCP schema advertisement, Debug redaction (phase-2 debt #2).
- **Library:** none (test-only).
- **Risk:** low — no production change.
- **Recommended agent:** `tdd-guide`; `rust-reviewer`.
- **Gate to phase reflect:** new unit tests pass; CI gate green.

---

## Cross-cutting (every change)

- Hexagonal rule intact; adapter wiring only in `fpa-gateway`.
- CI gate green per change (`./scripts/ci-check.sh`).
- **Never log the bearer/claims**; forge is the authz authority (agent forwards, doesn't replicate Keto/Cedar).
- Reuse over rebuild (`graphql_query`, wiremock harness).
- One commit per change.

## Open decisions to resolve at execute
1. **c001 / Q2:** exact pg_graphql collection names for `projects` / `applications` (confirm via introspection or fixture; builder is parameterized so a wrong guess is a config fix, not a rewrite).

## Deferred (not this plan)
WS subscription → AG-UI (`tokio-tungstenite`) — deferred to a later realtime
phase per the analyze recommendation.

## Execute order

```
1) apply p3-c001-forge-write-mutations
2) apply p3-c002-fabric-health
3) apply p3-c003-test-hardening
```

First change to apply: **`p3-c001-forge-write-mutations`**.
