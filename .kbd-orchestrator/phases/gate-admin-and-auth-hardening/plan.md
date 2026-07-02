# Plan — gate-admin-and-auth-hardening

**Phase:** gate-admin-and-auth-hardening
**Date:** 2026-07-02
**Backend:** OpenSpec (3 changes scaffolded + validated)
**Scope:** last plane (gate) + auth hardening + forge REST drift-sync

> Ordered change list the Execute stage drives one task per turn. All paths/config
> values were verified against **current** sibling source during spec (deep
> re-review) and parameterized via config, so residual mismatches are config fixes.

---

## Dependency graph

```
p4-c001-gate-admin-list-routes       (independent)
p4-c002-jwt-verification-hardening   (independent)
p4-c003-forge-rest-sync              (independent)
```

No inter-change dependencies. Ordered by value/risk: highest-risk auth first (most
scrutiny + review), then the drift-sync, then the small gate read.

---

## Ordered change list

### 1. `p4-c002-jwt-verification-hardening`  ·  highest risk (auth)
**Goal:** Position-dependent identity — trust configured gate headers, verify direct tokens against the IdP JWKS; retire the unverified decode.
- **Library:** reuse `jsonwebtoken` 9.3.1 `jwk` (no new crate) + `reqwest` (present).
- **Risk:** **high** — it's the auth path. A mistake weakens security. Mitigation: fail-safe default (no trusted headers + no key ⇒ reject), extensive tests, security review.
- **Config (verified):** `FPA_TRUSTED_IDENTITY_HEADERS` (gate's per-route injected headers) + `FPA_JWKS_URL` (the IdP JWKS gate uses — NOT a gate endpoint).
- **Recommended agent:** general Rust + **`security-reviewer`** (mandatory here) + `rust-reviewer`.
- **Open Q at execute:** which IdP (Kratos/Hydra) `FPA_JWKS_URL` points at; runtime reachability.
- **Gate:** trusted-header path, valid/invalid direct token, no-key→reject, JWKS-cache tests pass.

### 2. `p4-c003-forge-rest-sync`  ·  drift-sync
**Goal:** Migrate `fpa-forge` writes from pg_graphql to forge's new REST CRUD.
- **Library:** reuse `reqwest` (present); `wiremock` dev-dep present.
- **Risk:** medium — reuses c002-phase-3's HTTP+error patterns; the unknown is the REST path prefix (parameterized via `FPA_FORGE_REST_PREFIX`).
- **Recommended agent:** general Rust + `docs-lookup` (forge REST shape); `rust-reviewer`.
- **Open Q at execute:** confirm forge's compiled REST prefix (`/rest/<table>` vs `/<table>`) against `fdb-reflection` / running forge.
- **Gate:** wiremock authorized-insert / 403 / missing-bearer / unreachable + runner project.create-routes-to-REST tests pass.

### 3. `p4-c001-gate-admin-list-routes`  ·  small, last plane
**Goal:** Real `fpa-gate::list_routes` → bare `GET /routes` on the admin port.
- **Library:** reuse `reqwest` (present); `wiremock` dev-dep. **Reject** `flint-gate-client` git-dep (WS-heavy, cross-org, stale prefix).
- **Risk:** low — one GET + error mapping. Path resolved from source (bare `/routes`).
- **Recommended agent:** general Rust + `tdd-guide`; `rust-reviewer`.
- **Gate to phase reflect:** wiremock 200/401/unreachable tests pass; completes the last unimplemented plane.

---

## Cross-cutting (every change)

- Hexagonal rule intact; adapter wiring only in `fpa-gateway`.
- CI gate green per change (`./scripts/ci-check.sh`).
- **Never log tokens/claims**; forge/gate remain the authz authority (agent forwards bearer / trusts configured gate headers).
- **All external paths/URLs come from config** (verified from source; config-fixable).
- One commit per change.

## Open decisions to resolve at execute (all verified-from-source, config-parameterized)
1. **c002:** which IdP `FPA_JWKS_URL` targets (Kratos/Hydra) + reachability.
2. **c003:** forge REST path prefix (`/rest/<table>` vs `/<table>`).
3. **c001:** re-confirm bare `/routes` against a running gate if available (source says bare; client prefix is stale).

## Deferred (not this plan)
Gate route/api-key/signing-key **writes** (`POST/DELETE`), forge update/delete,
fabric WS subscriptions, OpenDesign, UI, Tauri, KB, durable task store.

## Execute order

```
1) apply p4-c002-jwt-verification-hardening   (security-reviewed)
2) apply p4-c003-forge-rest-sync
3) apply p4-c001-gate-admin-list-routes
```

First change to apply: **`p4-c002-jwt-verification-hardening`**.
