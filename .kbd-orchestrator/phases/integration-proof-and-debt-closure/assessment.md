# Assessment — integration-proof-and-debt-closure

**Phase:** integration-proof-and-debt-closure
**Date:** 2026-07-03
**Stage:** assess (grounded in code, not the reflection's assumptions)

> This assessment **corrects two of the reflection's assumptions** after reading
> the actual sibling source. The forge write path is more broken than "confirm the
> table name" implied, and M2 is smaller than stated. Findings below cite files.

---

## 1. Goals recap (from goals.md)

1. End-to-end integration proof across AG-UI / A2A / MCP.
2. Close deferred security findings: H2 (`/agui/stream` auth), H4 (JWKS single-flight), M2 (audit `signature_verified`).
3. Fix misplaced gate write-kind guard (`application.deploy` → refuse, not list).
4. Confirm forge table names.

---

## 2. Codebase state per goal (evidence)

### Goal 3 — gate write-guard (CONFIRMED gap, small fix)
`crates/fpa-app/src/task_runner.rs:108`:
```rust
TargetPort::Gate => self.gate.list_routes().await,
```
**All** Gate-targeted kinds route to `list_routes()`. `catalog.rs:91` catalogues
exactly one Gate kind — `application.deploy` — so a deploy **lists routes** instead
of refusing. Fix: branch on `entry.kind` (or a write-flag) before the port call and
return `AppError::Port(PortError::Downstream("gate route-write not implemented"))`
for write kinds. **Small, self-contained, testable.** No blocker.

### Goal 2a — H2 `/agui/stream` unauthenticated (CONFIRMED)
`crates/fpa-gateway/src/routes/agui.rs:31` — `async fn stream()` takes **no
`OperatorContext`**. Anyone can open the stream. Fix mirrors the A2A H1 fix
(c002): add `_operator: OperatorContext` to the handler so the extractor enforces
identity. The stub still emits `run_start`/`run_end`; auth just gates entry.
**Small.** No blocker.

### Goal 2b — H4 JWKS single-flight (CONFIRMED gap)
`crates/fpa-gateway/src/jwks.rs:66-95` — `jwks()` drops the read guard (line ~72),
then fetches, then takes the write guard (line ~89). Two concurrent callers on a
cold/stale cache **both** miss and **both** hit the IdP. Empty-set poisoning is
already guarded (H4 partial, done c002); the **double-fetch window** is not.
Fix: collapse to a single-flight — `tokio::sync::Mutex` around the refresh, or an
`OnceCell`-per-epoch, re-checking freshness under the write lock. **Small-medium.**

### Goal 2c — M2 audit `signature_verified` (SMALLER than reflection said)
`crates/fpa-gateway/src/identity.rs:59` already **defines** `signature_verified:
bool`, includes it in the redacted `Debug` (`:84`), and logs an "identity verified"
tracing event (`:122-124`). What's missing: the **task audit** in
`task_runner.rs::run` logs `operator`/`kind`/`decision` but **not** whether the
identity was signature-verified vs gate-trusted. Fix: thread the flag from
`OperatorContext` → `AuthContext` → the runner's audit `tracing::info!`. **Small.**

### Goal 4 → **REFRAMED: forge write path is mis-shaped, and `Project` has no forge home (BLOCKER-class)**
This is the phase's most important finding. Reading forge source:

- **Real REST insert path is `POST /{schema}/{table}`** — merged at the gateway
  **root with no `/rest` prefix**. Evidence:
  - `fdb-reflection/src/compilers/rest/mod.rs:258` →
    `format!("/{schema}/{table}")` is the generated `endpoint.path`.
  - `fdb-gateway/src/main.rs:182-192` → the reflection router is **`.merge`d**
    (not `.nest("/rest", …)`) onto the root app.
  - `mutations.rs:78` → handler signature is `Path((schema, table))` — **two**
    path segments, schema first.
- **The agent builds the wrong URL.** `fpa-forge/src/lib.rs` (`rest_insert`,
  `DEFAULT_REST_PREFIX = "/rest"`) POSTs to `{base}/rest/<table>` — **wrong prefix
  (`/rest` does not exist) and wrong arity (missing `{schema}`).** This is a
  guaranteed runtime 404.
- **There is no `projects` table in forge.** `migrations/` (0002–0005) define only
  `flint_a2ui.*` (applications, components, …) and `flint_meta.cedar_policies`.
  `flint_a2ui.applications` is an **A2UI component-app**, not the agent's
  administrative "application". The agent's `Project` is its **own domain artifact**
  (`fpa-domain/src/project/mod.rs` — "hub aggregate this agent administers"),
  `ProjectId(Uuid)` — with **no persistence target in forge**.

**Implication:** `project.create` cannot be integration-proven against a real forge
until we decide *where the Project artifact lives*. Options (for analyze/spec):
(a) the agent owns Project persistence itself (its own store/table it provisions),
(b) forge grows a `flint_meta.projects` (or similar) table the agent writes via the
corrected `POST /{schema}/{table}`, or (c) the proof uses a forge table that *does*
exist (e.g. an A2UI insert) to prove the REST wiring, deferring Project persistence.
**This is the top open question.**

### Goal 1 — integration proof (NOT STARTED; infra available)
- **Zero integration tests exist** — no `tests/` dir in any crate; all coverage is
  in-crate `#[cfg(test)]` wiremock/unit.
- **Sibling stack is runnable:** `flint-realtime-fabric` has `compose.yml` +
  `compose.ci.yml` + `Makefile`; `flint-gate` has `docker-compose.yml`. **forge has
  no top-level compose** — only a custom `images/postgres18/Dockerfile` + a
  `scripts/ci-check.sh`; standing up real forge is non-trivial (pgrx extensions +
  Postgres 18 image build).
- Therefore a **first proof should mock each plane at the HTTP boundary**
  (wiremock, the pattern already in `fpa-gate`/`fpa-forge` tests) driving the full
  `authenticate → project.create → list_routes → fabric.health` flow through the
  Axum router — then a **later** live smoke once forge's compose story is known.

---

## 3. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | forge REST path wrong (`/rest/<table>` vs real `/{schema}/{table}`) | Small (fix) | Blocks live `project.create` |
| G2 | `Project` artifact has no forge persistence target | **Design** | **Blocks** goal-1 project.create semantics |
| G3 | gate write-guard misrouted (`application.deploy`→list) | Small | No |
| G4 | H2 `/agui/stream` unauthenticated | Small | No |
| G5 | H4 JWKS double-fetch window | Small-med | No |
| G6 | M2 `signature_verified` not in task audit | Small | No |
| G7 | No integration test harness at all | Medium | Is the goal |

---

## 4. Open questions (for analyze / spec)

1. **Where does the `Project` artifact persist?** (agent-owned store vs a new forge
   `flint_meta.projects` table vs prove-with-existing-table). Drives G1/G2 and the
   shape of the integration proof. **This likely needs an operator decision.**
2. **Integration proof: mock-at-boundary first, or invest in live compose now?**
   forge has no ready compose; fabric/gate do. Recommend HTTP-boundary mock for the
   first green proof, live smoke as a follow-on.
3. **Which IdP backs `FPA_JWKS_URL`** (Kratos vs Hydra) and is a test JWKS/keypair
   available for a real verify test? (Or do we mint a test RSA key in-test?)

---

## 5. Handoff to analyze/plan

Four of the six debt items (G3–G6) are small, self-contained, no external research
needed. The integration proof (G7) is the phase's spine but is **gated on a design
decision** (G2: where `Project` lives) and a forge REST-path correction (G1) that is
now precisely known from source (`POST /{schema}/{table}`, no `/rest` prefix). The
biggest risk is treating G2 as "confirm a table name" — it is actually "decide the
Project persistence model," and it should go to the operator before spec. Recommend
Analyze focus: (1) resolve the Project-persistence decision, (2) confirm forge
compose feasibility, (3) pick the IdP/test-key approach.
