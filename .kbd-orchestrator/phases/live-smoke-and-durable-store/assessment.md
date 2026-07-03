# Assessment — live-smoke-and-durable-store

**Phase:** live-smoke-and-durable-store
**Date:** 2026-07-03
**Stage:** assess (grounded in the actual runnability of the sibling stack + this env)

> Headline: **goal #1 (live cross-plane smoke) is blocked in this environment** — the
> Docker daemon is unreachable and forge has no ready compose (only a heavy custom
> pgrx PG-18 image). Goal #2 (durable store) is **unblocked** via a testcontainer /
> dev Postgres and can proceed independently. This needs an operator scoping call.

---

## 1. Goals recap (from goals.md)

1. Live smoke against real forge + gate + fabric (close the mock-boundary gap).
2. Durable Postgres-backed `ProjectStore` (projects survive restart).
3. Archive the p5 OpenSpec changes.

---

## 2. Runnability of the sibling stack (the crux)

| Plane | How it runs | Cost / blocker |
|---|---|---|
| **forge** | **No compose.** CI via **Dagger (Go)**; data plane is a custom **pgrx Postgres-18 image** (`images/postgres18/Dockerfile`: `rust:1.96-bookworm` → compiles `flint_llm` (pgrx 0.18.1) + pgvector + pg_net from source). The Quarry **gateway** binary has no containerized run recipe. | **Heavy.** Multi-stage source build (minutes–tens of minutes) *and* no orchestration to run gateway+DB together. Highest-effort plane by far. |
| **gate** | `docker-compose.yml` — `build: .` + `postgres:16-alpine` + `oryd/kratos:v1.2`. | Moderate — a Docker build of gate + 2 services. |
| **fabric** | `compose.yml` — gateway build + `iggy` + `keto` + its own `flint-gate` + `surrealdb` + `postgres:17`. | **Heavy** — 6 services, two of them built. |

**Environment blocker:** the **Docker daemon is unreachable** here. `docker` and
`colima` CLIs are present, but `docker info` times out — colima isn't started. So
**nothing containerized can run right now** without the operator starting colima /
Docker first. Even then, forge is the tall pole (custom PG-18 build, no gateway
compose).

**Conclusion:** a full three-plane live smoke is **not achievable in this session**
as-is. Options are an operator decision (see §5).

---

## 3. Durable ProjectStore (goal #2 — unblocked, but placement matters)

- The `ProjectStore` port already exists (`fpa-ports/src/project_store.rs`:
  `put`/`get`, async). The in-memory adapter lives in **`fpa-app`**.
- **Hexagonal placement (Base Rule / Rule 16):** a Postgres adapter **must not** live
  in `fpa-app` (the app layer must not import infra). It belongs in a **new adapter
  crate** — proposed `fpa-store-pg` — alongside `fpa-forge`/`fpa-gate`/etc. The
  composition root (`fpa-gateway::state`) wires it; `fpa-app` keeps only the in-mem
  adapter (for tests).
- **Driver:** fabric uses **`tokio-postgres` 0.7** (`with-serde_json-1`). Adopting the
  same keeps the stack consistent (Base Rule 9) and lets the `Project` aggregate
  round-trip as a JSONB `body` column. `sqlx` is the alternative (compile-time-checked
  queries) — analyze should pick, verifying current versions (Base Rule 22).
- **Test strategy:** a `testcontainers`-driven Postgres (ephemeral, per-test) proves
  round-trip + restart-survival deterministically **without** the sibling stack — this
  is the durable-store proof, independent of goal #1. Needs Docker up too, but only
  ONE lightweight `postgres:*-alpine` container, not the whole fabric.
  - **Note:** testcontainers ALSO needs the Docker daemon. If Docker stays down, even
    the durable-store *integration test* can't run — but the adapter can still be
    *implemented* and unit-shaped, with the container test `#[ignore]`d until Docker.

---

## 4. Archive p5 changes (goal #3 — trivial, unblocked)

`openspec/changes/p5-c00N-*` validate `--strict` with tasks complete; `/opsx:archive`
moves them into `specs/`. No code impact. Do it regardless of the goal-1 scoping.

---

## 5. Gap summary

| # | Gap | Blocker? |
|---|---|---|
| G1 | Live 3-plane smoke | **BLOCKED** — Docker down + forge has no compose (heavy pgrx build). Operator decision. |
| G2 | Durable `ProjectStore` (new `fpa-store-pg` crate, tokio-postgres, JSONB) | Implementable now; its **container test** needs Docker up. |
| G3 | Where the durable store lives (agent PG vs forge `flint_meta.projects`) | **Operator decision** (revisits p5 now the write path is fixed). |
| G4 | Archive p5 OpenSpec changes | None — do it. |

---

## 6. Open questions (for the operator / analyze)

1. **Live-smoke scope** — the full three-plane live smoke can't run here (Docker
   down, forge unbuilt). How to proceed? (See the question I'm asking now.)
2. **Durable store location** — agent-owned Postgres (new `fpa-store-pg`) vs a forge
   `flint_meta.projects` table via the now-correct `POST /{schema}/{table}`.
3. **Driver + test infra** — `tokio-postgres` (sibling-consistent) vs `sqlx`;
   `testcontainers` (needs Docker) vs an assumed dev-compose Postgres.

---

## 7. Operator decisions (resolved at assess, 2026-07-03)

- **Live smoke → DEFERRED** to its own future phase (operator, AskUserQuestion).
  Reason: can't run here (Docker down; forge unbuilt/no compose). The p5
  mock-boundary proof stands until then.
- **Durable store location → agent-owned Postgres** (new `fpa-store-pg` crate). The
  forge-`flint_meta.projects` option is rejected for this phase — it inherits the
  same forge/Docker blocker we just deferred. Consistent with the p5 decision.
- **Driver → `tokio-postgres`** (sibling-consistent with fabric) is the recommended
  default; analyze confirms vs `sqlx` and pins the version.

## 8. Handoff to analyze/plan (re-scoped)

Live smoke is **deferred** (operator) — it can't run in this environment. This phase
is now **durable `ProjectStore` + p5 archival**. Build a new **`fpa-store-pg`**
adapter crate (hexagonal — NOT in `fpa-app`) implementing the existing `ProjectStore`
port with `tokio-postgres` + a JSONB `body` column, wired at the composition root;
keep the in-mem adapter for tests. The container-backed round-trip/restart test needs
Docker up, so it lands `#[ignore]`d-by-default (runs when colima is started). Archive
p5-c001..c004 into `specs/`. Analyze: confirm `tokio-postgres` vs `sqlx` + versions,
and the testcontainers approach. No remaining operator gates.
