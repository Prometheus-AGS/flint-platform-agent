# Analysis — real-sibling-smoke

**Phase:** real-sibling-smoke
**Date:** 2026-07-04
**Mode:** stack-specified (existing siblings; verify build recipes from source)
**Research:** internal sibling-source verification (no external library hunt — Tiers
1–4 not run; nothing to adopt).

---

## 1. What this analyze produced

Concrete build recipes for each real sibling, verified from source. The material
findings (some simplify the plan, one complicates it):

### gate — moderate, ready
- `flint-gate/docker-compose.yml`: `gate` (`build .`) + `postgres:16-alpine` + `kratos
  v1.2`. Admin on **`:4457`** (what the agent's `FPA_GATE_ADMIN_URL` targets), proxy
  `:4456`. Env: `FLINT_GATE_JWT_SECRET`, `FLINT_GATE_CONFIG=/app/config/config.yaml`.
- Dockerfile: `node:22` web-builder + `rust:1.90-bookworm` builder → `debian-slim`.
  Buildable as-is.
- **The agent calls `GET {admin}/routes`** — a real gate serves it. Verdict: **include
  real gate** (build + its 2 deps).

### fabric — HEAVY (finding: gateway-only NOT viable)
- `flint-realtime-fabric/compose.yml` `gateway` **hard-depends** (`depends_on:
  service_healthy`) on **iggy-server + keto + surrealdb** (+ postgres). It reads
  `IGGY_CONNECTION_STRING`, `KETO_BASE_URL` at startup — it will not boot standalone.
- **So "real fabric" = the full 6-service compose** for what the agent only uses as a
  single `GET /healthz`. That's a lot of weight for one health check.
- **Options for the plan:** (a) run the full fabric compose (heavy, high fidelity); or
  (b) keep fabric **stubbed** in the real-sibling smoke and note it — the agent's fabric
  surface is *only* `/healthz`, so a real fabric adds near-zero agent-visible fidelity
  vs. its cost. **Lean: stub fabric's `/healthz`, run real gate + real forge** — that's
  where the agent's real surface actually lives (routes + openapi/graphql/rest).

### forge — LIGHTER than feared + gateway Dockerfile is SIMPLE
Two findings that de-risk forge substantially:
- **CI PG image (`docker/postgres/Dockerfile`) installs pg_graphql from a prebuilt
  `.deb`** (`curl` the `v1.5.11` release + `dpkg -i`), **not** a pgrx source build.
  pgvector is a source build (`v0.8.0`), but that's quick. So the CI image is a
  moderate build, **not** the heavy pgrx compile — good.
- **`fdb-gateway` uses `sqlx::query_as` (runtime-checked), has NO `.sqlx` offline
  cache** → **it builds WITHOUT a live database.** Authoring its Dockerfile is a plain
  multi-stage rust build (like ours: exclude `.cargo`, add `libssl-dev`). It binds via
  `axum::serve(listener, ...)` and reads `DATABASE_URL` at **runtime** (warns, doesn't
  fail, if unset — but needs it for a real `/openapi.json`).
- **Migrations:** `sqlx migrate run --source migrations` (from `ci-test.sh`) → the smoke
  needs a migrate step (a one-shot `sqlx-cli` container, or `sqlx migrate run` against
  the CI PG) before the gateway is useful.

### full pgrx image (best-effort change)
`images/postgres18/Dockerfile` — `rust:1.96` + `cargo install cargo-pgrx` + `cargo pgrx
init --pg18` (builds Postgres) + `flint_llm`/pg_net/pg_cron. The genuinely heavy one;
time-boxed best-effort per the operator decision.

---

## 2. Build-vs-adopt (all reuse-sibling / build-thin)

| Gap | Decision | Note |
|---|---|---|
| gate in smoke | **reuse** gate's Dockerfile+compose | admin :4457 |
| fabric | **stub `/healthz`** (recommend) or reuse full compose | gateway-only not viable |
| forge CI PG | **reuse** `docker/postgres/Dockerfile` | pg_graphql via .deb — moderate |
| `fdb-gateway` image | **build** a thin Dockerfile (author) | runtime sqlx → no DB-at-build |
| migrate step | **reuse** `sqlx migrate run --source migrations` | one-shot |
| full pgrx image | **reuse** `images/postgres18/Dockerfile` (best-effort) | heavy, time-boxed |

No external library candidate — everything is a sibling artifact or a thin Dockerfile.
`library-candidates.json` records `build_required` gaps.

---

## 3. Key recommendation to spec

**Real gate + real forge (CI image + authored gateway + migrate); stub fabric.**
Rationale: the agent's *real* forge/gate surface (routes, openapi, graphql, rest) gets
genuine fidelity; fabric's surface is a single `/healthz` whose real 6-service stack
adds cost far out of proportion to fidelity. This is the highest fidelity-per-effort.
(If the operator wants *full* fidelity including real fabric, it's the full compose —
flag at spec.)

---

## 4. Open questions (for spec / operator)

1. **Real fabric or stub it?** Lean: **stub** (agent only probes `/healthz`; real fabric
   = 6 services for one health check). Operator may want full fidelity anyway. → confirm.
2. **Author `fdb-gateway` Dockerfile in `../flint-forge` or in our `smoke/`?** Lean:
   in our `smoke/` (self-contained; doesn't edit a sibling repo mid-development). It
   builds from the `../flint-forge` context.
3. **Build time-box** for the full-pgrx best-effort change — a hard cap (e.g. abort +
   document if it exceeds N minutes / OOMs).
4. **gate config** — gate wants `/app/config/config.yaml` + a JWT secret; use gate's
   default/example config for the smoke.

---

## 5. Handoff to spec

Sibling recipes verified from source. **De-risking findings:** the forge CI image uses
a prebuilt pg_graphql `.deb` (not pgrx — moderate build), and `fdb-gateway` builds
without a live DB (runtime sqlx) → its Dockerfile is thin. **Complicating finding:**
fabric's gateway hard-depends on iggy+keto+surreal, so real fabric = the full 6-service
compose for a single `/healthz` → **recommend stubbing fabric**, running **real gate +
real forge (CI PG + authored fdb-gateway + `sqlx migrate run`)**. Full pgrx image stays
a time-boxed best-effort change. Spec the changes; surface the fabric stub-vs-real
decision to the operator.
