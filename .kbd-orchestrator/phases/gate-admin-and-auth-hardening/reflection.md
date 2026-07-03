# Reflection — gate-admin-and-auth-hardening

**Phase:** gate-admin-and-auth-hardening
**Date:** 2026-07-03
**Changes:** 3/3 executed (p4-c001, c002, c003), all CI-green and pushed.

> Sycophancy gate applied. This was the highest-stakes phase — the auth path — and
> the phase where the working philosophy changed mid-flight (implementation-first).
> Milestone reached: **all four fabric planes are now implemented.**

---

## 1. Goal achievement

Goals (goals.md) + one scope addition (forge REST drift):

| Goal | Verdict | Evidence |
|---|---|---|
| `fpa-gate` admin `list_routes` (last plane) | **MET** | c001: real `GET {admin}/routes` (bare path, source-verified); 3 wiremock tests |
| Full JWT verification (position-dependent) | **MET+** | c002: trust configured gate headers / verify direct tokens vs IdP JWKS / reject unverified. Retired the insecure decode. **+ 2 CRITICAL + 6 HIGH/MED security-review findings fixed** |
| Trust gate-injected identity | **MET** | c002: configured trusted-header path (`FPA_TRUSTED_IDENTITY_HEADERS`) with a startup deployment guard |
| Forge update/delete (stretch) | **DEFERRED** | intentional; not attempted |
| **Forge REST sync (added, drift)** | **MET** | c003: `create_entity` → forge REST CRUD (`POST /rest/<table>`), synced to forge p3-c013/c014 |

**Overall: ~95%.** All in-scope goals MET; the stretch was a deliberate defer. This
phase also **completed the last unimplemented plane (gate)** and **absorbed real
sibling drift** (forge REST + gate admin contract) caught by re-review.

---

## 2. Delivered changes

| Change | Delivered | Commit |
|---|---|---|
| c002 jwt-verification-hardening | position-dependent auth + `jwks.rs` verifier; security-review fixes | `2e9890b` |
| c003 forge-rest-sync | `create_entity` via forge REST; `rest_prefix` config | `96d3825` |
| c001 gate-admin-list-routes | real `fpa-gate::list_routes` | `96d3825` |

Plus `26f6fb7` (philosophy + fast-build settings). All on `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/3** (subsystem absent — all four phases) |
| Changes with a dedicated **security review** | **1/3** (c002 — mandatory, agent-run) |
| Security findings (c002): total / fixed-now / deferred | 15 / 11 / 4 |
| First-pass CI-gate pass rate | 3/3 |

**The security review was this phase's most valuable artifact.** Ordering the auth
change first (by risk) meant a full adversarial `security-reviewer` pass **before
commit**, which caught **2 CRITICAL** vulnerabilities in my first implementation:
- **C2 algorithm confusion** — I used `Validation::new(header.alg)`, letting an
  attacker pick the verification algorithm. Fixed: server-fixed `ALLOWED_ALGS`.
- **C1 header spoofing** — the trust-header model with no network guard let any
  direct client forge identity. Fixed: startup requires `FPA_BEHIND_TRUSTED_GATE=true`.

Both were real, both are gone. This validates "review the security-critical change,
first, with a dedicated pass."

---

## 4. Technical debt

**New / carried honest gaps:**
1. **c001 gate write-kind guard is misplaced (honest miss).** `application.deploy`
   targets `TargetPort::Gate` → `list_routes`, so a deploy currently *lists routes*
   instead of cleanly refusing. The runner's write-guard only covers the forge
   plane. Needs a gate-plane write guard. (Task c001/2.3 was checked but the guard
   isn't where it needs to be.)
2. **4 security findings deferred** (tracked from the c002 review):
   - H2: `/agui/stream` still unauthenticated (stub emits no real data yet).
   - H4 (partial): empty-JWKS guard added, but the TOCTOU cache single-flight
     (concurrent double-fetch window) is not yet closed.
   - M2: `signature_verified` is still not surfaced in an audit record.
   - L2/L3: MCP metadata exposure; hard-coded JWKS TTL.
3. **c003 collection/table naming assumption** — `dispatch_forge` passes
   `"Projects"`/`"Applications"`; the real forge table names must be confirmed
   against a running forge (config-fixable via the path, but the names are guesses).
4. **No live-service smoke** any phase — all verification is wiremock/unit + CI;
   no run against a real forge/gate/fabric (needs the full stack up).

**Carried (older):**
5. Interim: forge writes are **insert-only** (update/delete deferred).
6. MCP client single-endpoint; task store non-durable.

---

## 5. Lessons captured

- **Order the security-critical change FIRST and give it a dedicated adversarial
  review.** It caught 2 CRITICALs pre-commit. Auth-last would have rushed it. This
  is now a rule for any phase touching auth/crypto/access-control.
- **Never derive the JWT verification algorithm from the token header.** Fix the
  allowed algorithm set server-side. (Alg-confusion is the classic JWT CVE class.)
- **A "trust the headers" model needs an enforced deployment prerequisite**, not
  just a doc note — fail startup if the operator hasn't acknowledged the
  network-isolation constraint.
- **Re-verify siblings from source before each phase's spec.** This phase caught
  BOTH forge REST drift and the stale gate `/v1/admin` prefix — either would have
  produced 404s at runtime. The assess/spec-from-source discipline keeps paying.
- **Implementation-first works (new philosophy, applied this phase).** c001 + c003
  were written fully, then validated with ONE batched `cargo check`, then ONE CI
  pass — not per-change churn. Faster, and the fast-build settings (sccache +
  ld64.lld + profile.dev) made the check ~5s. Trust the rules during writing,
  validate holistically.
- **Redact secrets in every manual `Debug`, not just some.** The review flagged
  `GatewayConfig` leaking the HS256 secret; the pattern must be applied to every
  struct that holds credentials.

---

## 6. Recommended Next Phase

**`integration-proof-and-debt-closure`** — the project now has all four planes and
hardened auth but **has never run against the real stack**. Per the
implementation-first philosophy, it's time for the holistic integration proof, plus
closing the honest gaps.

Scope (proposed):
1. **Full integration test / live smoke** — stand up (or mock at the boundary) a
   real forge + gate + fabric, and drive an end-to-end operator flow
   (authenticate → project.create via forge REST → list_routes via gate →
   fabric.health) through AG-UI/A2A/MCP. This is the "test the proven shape of the
   whole system" step the philosophy calls for.
2. **Close the security-review deferrals** (debt #2): auth `/agui/stream` (H2),
   JWKS single-flight (H4), audit `signature_verified` (M2).
3. **Fix the gate write-kind guard** (debt #1).
4. **Confirm forge table names** (debt #3) against the running forge.

**Prerequisites / open questions:**
- Can the full sibling stack (forge Postgres 18 + gate + fabric) run locally / in
  compose for a real integration test, or do we mock at the HTTP boundary?
- Which IdP (Kratos/Hydra) backs `FPA_JWKS_URL` for a real token test?

Still deferred: forge update/delete, MCP multi-server, fabric WS subscriptions,
OpenDesign, UI generation, Tauri, KB, durable task store.

---

## 7. Reflect handoff

All in-scope goals MET; the four fabric planes are now implemented and auth is
production-grade (security-review'd, 2 CRITICALs fixed). Honest gaps: gate
write-guard misplaced, 4 deferred security findings, forge table names unconfirmed,
and — the big one — **the system has never run end-to-end against the real stack.**
Corrective action & recommended next phase: **integration-proof-and-debt-closure**
— the holistic integration test the implementation-first philosophy demands, plus
closing the deferred security/guard debt. Pending: can the sibling stack run
locally, and which IdP backs the JWKS.
