# Assessment — real-sibling-smoke

**Phase:** real-sibling-smoke
**Date:** 2026-07-04
**Stage:** assess (re-synced with the live flint-forge codebase per operator directive)
**Docker:** healthy (colima vz, 6 CPU / 12 GiB)

> Operator directive: re-sync with `../flint-forge` (mid-development) and ensure its
> **Postgres extension code is built into the PG-18 image** we use, so the agent gets
> the fabric features it needs. Done — and it materially reshapes the forge plan.

---

## 1. Goals recap (from goals.md)

1. Real gate + fabric in the smoke (reachable now).
2. Real forge (STRETCH — pgrx image + a gateway Dockerfile).
3. `compose.real.yml` + `run-real.sh`; stub `run.sh` stays.

---

## 2. flint-forge re-sync (the directive) — findings

**forge is actively mid-development** (branch `main`, latest `8c7955c` today —
`p35-c004`: "DB tests live-verified + acquire RLS fix"). It ships **THREE** PG-18
image tiers, which changes the forge decision:

| Image | Extensions bundled | Build cost | Notes |
|---|---|---|---|
| `docker/postgres/Dockerfile` (**CI**, from p35-c003) | pgvector 0.8.0 + **pg_graphql 1.5.11** | Moderate (2 source builds on `postgres:18-bookworm`) | Purpose-built for `scripts/ci-test.sh` + Dagger. Has pg_graphql. |
| `images/postgres18/Dockerfile.baseline` | pgvector + **SQL-only** `flint_auth`/`flint_hooks`/`flint_llm` schemas | **Light** (no Rust) | Fast; the "verified-working subset". |
| `images/postgres18/Dockerfile` (**full**) | + `flint_llm` (pgrx 0.18.1), `pg_net`, `pg_cron` | **Heavy** — 4 compile stages incl. `rust:1.96` + `cargo install cargo-pgrx` + `cargo pgrx init --pg18` (builds Postgres!) | Full Anvil (Ember LLM BGW). The convergence risk. |

**What the agent actually needs from forge (grounds the tier choice):** `fpa-forge`
calls `GET /openapi.json`, `POST /{schema}/{table}` (writes), and `POST /graphql`
(`graphql_exec`) — all served by the **Quarry gateway (`fdb-gateway`)** over a PG that
has the migrations applied. **pg_graphql matters** (agent has a GraphQL path).
`flint_llm`/`pg_net`/`pg_cron` (the heavy pgrx bits) are **NOT exercised by the agent
today** — Ember/hooks aren't on the agent's forge surface.

→ **The required tier is the CI image (`docker/postgres/Dockerfile`) — pg_graphql +
pgvector, moderate build — NOT the full pgrx image.** The full image's extensions are
real features to build *eventually*, but the agent doesn't call them yet, so making the
smoke depend on the heavy pgrx build would be **YAGNI + the biggest convergence risk for
no agent-visible gain**. (Operator directive honored: the extensions the agent needs —
pg_graphql for reads/writes — ARE built into the image we'll use; the ones it doesn't
yet use are noted as future.)

**Two hard forge gaps remain regardless of tier:**
1. **`fdb-gateway` (Quarry) has NO Dockerfile** — confirmed. The agent talks to the
   gateway, not raw PG. We must **author an `fdb-gateway` Dockerfile** (multi-stage
   rust build, like ours) — and it needs `libssl-dev` (same `openssl-sys` lesson) and
   the `.cargo` exclusion.
2. **Migrations must be applied** to the PG before the gateway serves a useful
   `/openapi.json` (the reflection compiler reads the live schema). forge's
   `ci-test.sh` does `migrate + test`; we need the same migrate step in the smoke.

**forge is itself BLOCKED on Docker** (p35-c003 task: "Run CheckDb / build the image
end-to-end — BLOCKED in this env (no dagger CLI, no Docker)"). We now HAVE Docker — so
we can build/run what forge couldn't. Worth noting to the forge team.

---

## 3. Sibling containerization inventory (gate/fabric)

| Sibling | Ships | Reachable? |
|---|---|---|
| **gate** | `Dockerfile` + `docker-compose.yml` (gate + postgres:16 + kratos) | ✅ build + run |
| **fabric** | `Dockerfile` + `compose.yml` (gateway + iggy + keto + its own gate + surrealdb + postgres:17) + Makefile | ✅ but **heavy** (6 services) |
| **forge gateway** | crate only; **no Dockerfile** | ❌ must author (see §2) |
| **forge PG** | 3 image tiers (see §2) | ✅ CI tier is the pick |

---

## 4. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | Real gate in the smoke (build + wire `FPA_GATE_ADMIN_URL`) | Small-med | No |
| G2 | Real fabric in the smoke (heavy 6-svc compose; agent only needs `/healthz`) | Med | No — maybe run a **subset** (just fabric gateway) |
| G3 | forge CI PG image (`docker/postgres/Dockerfile`) build | Med (2 source builds) | No |
| G4 | **Author `fdb-gateway` Dockerfile** (none exists) + migrate step | **Med-high** | forge fidelity gate |
| G5 | `compose.real.yml` + `run-real.sh`; keep stub `run.sh` | Med | No |
| G6 | Full pgrx image (flint_llm/pg_net/pg_cron) | Heavy | **Out of scope** — agent doesn't use it yet |

---

## 5. Open questions (operator decision at analyze/spec)

1. **forge image tier — confirm CI image, not full pgrx?** My strong rec: the **CI
   image** (pg_graphql + pgvector) — it's what the agent's forge surface needs; the full
   pgrx build is heavy and its extensions (Ember LLM/pg_net/pg_cron) aren't on the
   agent's path yet. **This is the key decision your directive raised** — do you want
   the agent's real-forge smoke to (a) use the CI image now (agent-sufficient, moderate
   build), or (b) invest in the full pgrx image for future features + convergence risk?
2. **fabric scope:** run fabric's **full 6-service compose**, or just the fabric
   **gateway** (the only thing the agent probes — `/healthz`)? Lean: gateway-only /
   minimal for the smoke.
3. **`fdb-gateway` Dockerfile authored HERE (in flint-forge) or in our repo's smoke?**
   Lean: author it under `../flint-forge` (it's forge's artifact; forge is blocked on
   Docker and would benefit) — but that edits a sibling repo. Confirm.
4. **Time-box the builds** — a hard cap so a runaway build doesn't stall the phase;
   fall back to keeping that plane stubbed.

---

## 5b. Operator decision (resolved at assess, 2026-07-04)

**forge image = BOTH (CI tier for the smoke + full pgrx image as a separate best-effort
change).** Provenance: **user** (AskUserQuestion). Two decoupled changes:
- **Core:** the real-forge smoke uses forge's **CI image** (`docker/postgres/Dockerfile`
  — pg_graphql + pgvector) + applied migrations + an authored `fdb-gateway` Dockerfile.
  Agent-sufficient, converges → guaranteed forge fidelity in the smoke.
- **Best-effort:** attempt building the **full pgrx image**
  (`images/postgres18/Dockerfile` — flint_llm/pg_net/pg_cron). Converges → proves the
  full extension stack builds (valuable for forge, which is Docker-blocked). Stalls →
  document the blocker; **does NOT touch the green CI-tier smoke.** Time-boxed.

fabric: run **gateway-only / minimal** (the agent only probes `/healthz`), not the full
6-service compose, unless assess-of-fabric shows the gateway needs its deps to boot.

## 6. Handoff to analyze/plan

Re-synced with live forge (directive): forge ships **3 PG-18 image tiers**; the agent's
forge surface (`/openapi.json`, `/{schema}/{table}`, `/graphql` via `fdb-gateway`) needs
the **CI image (pg_graphql + pgvector)**, NOT the heavy full pgrx image (its Ember
LLM/pg_net/pg_cron extensions aren't on the agent's path yet — building them in would be
convergence risk for no current gain; noted as future). Two real forge gaps: **no
`fdb-gateway` Dockerfile exists** (must author) + a migrate step. gate + fabric are
build+run-ready (fabric heavy → consider gateway-only). The key operator decision is the
**forge image tier** (CI vs full pgrx) — recommend CI. Recommend a build time-box and
incremental landing (gate → fabric-minimal → forge-CI+gateway), keeping any
non-converging plane stubbed. CLAUDE.md's `../flint-forge` link is current; the
extension-image detail is captured here.
