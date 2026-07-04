# Assessment — live-smoke-and-durable-proof

**Phase:** live-smoke-and-durable-proof
**Date:** 2026-07-04
**Stage:** assess (grounded in a LIVE Docker environment — colima vz, 6 CPU / 12 GiB)

> Headline: **Goal #1 is already proven.** With Docker up, the `fpa-store-pg`
> durable test **ran against a real Postgres** (testcontainers, 7.56s, PASS) after
> one env fix. The durable-store runtime-proof debt is effectively discharged;
> this phase bakes it in + adds the agent-container HTTP smoke.

---

## 1. Goals recap (from goals.md)

1. Durable-store proof vs real Postgres.
2. Agent container (`Dockerfile`) + compose stack exposing `:8088`.
3. Automated smoke (Playwright/HTTP) driving the live endpoint.

---

## 2. Live findings (run against real Docker)

### Goal 1 — durable proof: **PROVEN NOW** (needs one config fix baked in)
- `docker pull postgres:17-alpine` ✅; `cargo test -p fpa-store-pg -- --ignored`
  **ran and PASSED** — real container, put → new pool → get → **list** →
  restart-survival, 7.56s.
- **The fix (must be captured):** testcontainers resolves the daemon via
  **`DOCKER_HOST`**, not the docker CLI *context*. colima's socket is at
  `unix:///Users/gqadonis/.colima/default/docker.sock`. Unset → `Connection refused
  (os 61)`; set → PASS. So the phase must provide a **runner** (script / `just` /
  `make` target) that exports:
  ```
  DOCKER_HOST=$(docker context inspect colima --format '{{.Endpoints.docker.Host}}')
  TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE=${DOCKER_HOST#unix://}
  cargo test -p fpa-store-pg -- --ignored
  ```
  This is the durable-proof deliverable — small, and already validated.

### Goal 2 — agent container: what it needs (all known)
- **Bind:** the gateway binds `0.0.0.0:8088` by default (`FPA_GATEWAY_ADDR`
  overridable) — container-friendly, publish `8088:8088`.
- **Required env at startup (or it aborts before binding):** `FPA_FORGE_URL`,
  `FPA_FABRIC_ENDPOINT`, `FPA_GATE_ADMIN_URL`. So a dependency (real or stub)
  **must** exist for the agent to boot. HS256 auth via `FPA_GATE_JWT_KEY` (mint a
  bearer in the smoke, as the p5 integration proof does — no live IdP needed).
- **Dockerfile:** multi-stage — `rust:1.93+`-based builder (MSRV 1.93, edition 2024)
  → `cargo build -p fpa-gateway --release` → slim runtime (debian-slim or distroless)
  copying the one binary. Standard; no workspace obstacles.

### Goal 3 — stub targets are trivial (3 GET endpoints)
The agent's adapters probe exactly:
- **forge:** `GET {FPA_FORGE_URL}/openapi.json` (+ `/graphql`, `/rest/...` for writes —
  but the smoke uses the **store**, not forge, for project data, so `/openapi.json`
  returning a minimal doc suffices for `forge.table.*` reads).
- **fabric:** `GET {FPA_FABRIC_ENDPOINT}/healthz` → 200.
- **gate:** `GET {FPA_GATE_ADMIN_URL}/routes` → 200 JSON.
A single tiny stub (wiremock image, or an nginx/static-json container) answers all
three. This is why **stubs are low-risk**: the surface to fake is 3 static GETs.

---

## 3. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | Durable proof: capture the `DOCKER_HOST` runner + keep the test | **Small — already passing** | No |
| G2 | `Dockerfile` for `fpa-gateway` (multi-stage) | Small-med | No |
| G3 | `compose.smoke.yml`: agent + postgres + dep stubs, expose :8088 | Medium | No |
| G4 | Automated HTTP smoke (Playwright/curl) driving the live flow | Medium | No |
| G5 | Sibling deps: real vs stubs | **Operator decision** | gates G3/G4 fidelity |
| G6 | Real forge pgrx build (stretch) | Heavy | No (out of core scope) |

---

## 4. Open questions (operator decision at spec)

1. **Sibling deps = real forge/gate/fabric, or HTTP stubs?** Evidence now in hand:
   the stub surface is **3 static GETs**, so stubs give a genuine live-agent proof
   fast and reliably; real forge needs the heavy pgrx PG-18 build (may not converge).
   **Strong recommendation: stubs for the first green smoke (G3/G4); real siblings a
   follow-on/stretch (G6).** — asking at spec.
2. **Smoke driver:** Playwright (operator-named; good for HTTP + future UI) vs a
   `curl+jq`/`reqwest` harness. Playwright fits the instruction; confirm HTTP-only scope.
3. **Compose + Dockerfile location:** a dedicated `smoke/` dir (compose, Dockerfile,
   stub configs, runner) vs repo root. Lean: `smoke/`.
4. **Durable-proof home:** a `smoke/run-durable-proof.sh` (the DOCKER_HOST runner) +
   optionally a Postgres in the same compose that the test targets by URL. Lean: keep
   the testcontainers test as the canonical proof (it self-manages the container);
   the compose Postgres is for the *agent's* `FPA_PROJECT_DB_URL` durability across
   an agent-container restart (a second, higher-level proof).

---

## 4b. Operator decision (resolved at assess, 2026-07-04)

**Smoke dependencies → HTTP stubs (first green milestone).** Provenance: **user**
(AskUserQuestion). `compose.smoke.yml` = built `fpa-gateway` (:8088) + `postgres:17-alpine`
(for the agent's `FPA_PROJECT_DB_URL`) + **one stub container** (wiremock) answering the
3 GETs (`/openapi.json`, `/healthz`, `/routes`). Playwright drives `:8088`. **Real
forge/gate/fabric are NOT in scope this phase** — a separate future phase, once/if the
pgrx build story is sorted. This guarantees a green live-smoke without the sibling-build
convergence risk. Smoke driver = **Playwright** (operator-named), HTTP scope.

## 5. Handoff to analyze/plan

Docker is live and **goal #1 already passes** — the durable-store proof ran against a
real Postgres (testcontainers) once `DOCKER_HOST` points at colima's socket; that
runner is the deliverable. Goal #2/#3/#4 are a multi-stage `fpa-gateway` Dockerfile +
a `compose.smoke.yml` (agent + postgres + a 3-GET dependency stub) exposing `:8088` +
an automated HTTP smoke driving authenticate→project.create→inspect/list→fabric.health
→MCP→agui-auth. The stub surface is tiny (3 static GETs), making **stubs the low-risk
first target** — the one operator decision (real vs stubs) is recommended as stubs-first
with real siblings as a stretch (G6). Analyze can likely **skip** external research
(images are stock: `rust`, `postgres`, `wiremock`/`nginx`); confirm the Playwright
harness approach at spec.
