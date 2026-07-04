# Analysis — live-smoke-and-durable-proof (SKIPPED)

**Phase:** live-smoke-and-durable-proof
**Date:** 2026-07-04
**Status:** **SKIPPED** — `/kbd-analyze --skip "stock images (rust/postgres/wiremock);
durable proof already validated live; no external research"`

---

## Why skipped

Every dependency is a **stock, already-pullable image** and the highest-risk goal is
**already validated live** — there is no library/framework/skeleton to research.

Confirmed pullable this session (colima vz Docker):
- `rust:1.93-slim-bookworm` — builder base at our MSRV (edition 2024). ✅
- `debian:bookworm-slim` — slim runtime for the agent binary. ✅
- `postgres:17-alpine` — the agent's `FPA_PROJECT_DB_URL` DB + the testcontainers DB. ✅
- `wiremock/wiremock:latest` — the dependency stub (3 static GETs). ✅

Confirmed build path: `fpa-gateway` is a **bin+lib** crate → the Dockerfile builds the
binary with `cargo build -p fpa-gateway --release --bin fpa-gateway`.

Confirmed proof: `cargo test -p fpa-store-pg -- --ignored` **PASSED against a real
Postgres** (7.56s) once `DOCKER_HOST` points at colima's socket (see
`memory: testcontainers-colima-docker-host`).

The tiered research pipeline would return nothing here — skip is the honest call.

## Decisions carried to spec (from assess + operator)

1. **Deps = HTTP stubs** (operator): one `wiremock` container answering
   `/openapi.json`, `/healthz`, `/routes` (200). Real forge/gate/fabric = a separate
   future phase.
2. **Smoke driver = Playwright** (operator-named), HTTP scope on `:8088`.
3. **Durable proof runner** — `smoke/run-durable-proof.sh` exporting `DOCKER_HOST` +
   `TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE` from the colima context, then
   `cargo test -p fpa-store-pg -- --ignored`. Already validated.
4. **Dockerfile** — multi-stage `rust:1.93-slim-bookworm` builder → `debian:bookworm-slim`
   runtime, copy the one `fpa-gateway` binary; run on `0.0.0.0:8088`.
5. **Layout** — a dedicated `smoke/` dir: `Dockerfile`, `compose.smoke.yml`, wiremock
   stub mappings, `run-durable-proof.sh`, the Playwright smoke, and a runner
   (`smoke/run.sh` / README). Keeps it separate from any future production compose.
6. **Auth** — HS256: the smoke mints a bearer with `FPA_GATE_JWT_KEY` (as the p5
   integration proof does); no live IdP.

## Candidates

None. `library-candidates.json` records zero candidates; the gaps are `build_required`
(compose + Dockerfile + stub config + Playwright smoke over stock images).

## Handoff to spec

Analyze skipped (justified — stock images, live-validated proof, no research). Spec
from the assessment + the six decisions above: (a) durable-proof runner; (b)
`Dockerfile` + `compose.smoke.yml` + wiremock stub exposing `:8088`; (c) Playwright
HTTP smoke driving authenticate→project.create→inspect/list→fabric.health→MCP→agui-auth
against the real running agent.
