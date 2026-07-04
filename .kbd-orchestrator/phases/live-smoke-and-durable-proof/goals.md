# Goals — live-smoke-and-durable-proof

> Seeded from `project-read-and-list/reflection.md` → "Recommended Next Phase"
> (Option A), and shaped by the operator: **Docker is now healthy** (source-built
> colima + lima, vz driver, 6 CPU / 12 GiB — see `scripts/reset-colima.sh`). This is
> the phase that finally runs the agent against **real infrastructure** and discharges
> the two biggest carried debts: durable-store runtime proof + live smoke.
>
> Delivered artifacts (operator-specified shape): a **compose file** that builds
> images and stands up the environment, an **exposed HTTP endpoint**, and an
> **automated smoke test** (Playwright/HTTP tools) driving it.

## Primary goals (risk-ordered — land the sure win first)

1. **Durable-store proof against real Postgres.** Run the `fpa-store-pg` durability
   path (put → new pool → get → **list** → restart-survival) against a **real
   Postgres container**. Two acceptable forms: (a) the existing `#[ignore]`d
   testcontainers test now actually runs (`cargo test -p fpa-store-pg -- --ignored`);
   and/or (b) a compose `postgres:17-alpine` on a fixed port with `FPA_PROJECT_DB_URL`
   pointed at it. This converts "compiled + unit-tested" → "proven against a real DB."
   **Lowest risk, highest confidence — do this first.**

2. **Agent container + exposed endpoint.** A `Dockerfile` that builds `fpa-gateway`
   (multi-stage: cargo build → slim runtime) and a **compose file**
   (`compose.smoke.yml` or similar) that runs it wired to its required deps, exposing
   the gateway on a host port (`:8088`). The agent's config requires `FPA_FORGE_URL`
   / `FPA_FABRIC_ENDPOINT` / `FPA_GATE_ADMIN_URL` — those point at either real siblings
   or lightweight stubs (decided at spec).

3. **Automated smoke test against the running agent.** Drive the live endpoint end to
   end — `healthz`, an authenticated A2A `project.create` → `project.inspect`/`list`,
   `fabric.health`, MCP `tools/list`+`tools/call`, and the `/agui/stream` auth gate —
   asserting real HTTP responses from the **real running binary** (Playwright/HTTP
   tooling). This is the live analogue of the p5 mock-boundary integration proof.

## Success criteria

- Real-Postgres durability proof passes (test output shows it RAN, not `ignored`).
- `docker compose -f compose.smoke.yml up` builds + starts the agent; `curl`/Playwright
  against the exposed port returns healthy responses for the driven flow.
- The smoke is repeatable (a script/`just`/`make` target) and documented; teardown clean.
- Any wire drift the mock-boundary hid (paths, headers, status codes, auth handshake)
  is fixed and noted.

## Open questions (for /kbd-assess → operator decision at spec)

- **Sibling dependencies in the smoke: real forge/gate/fabric, or HTTP stubs?**
  Real = highest fidelity but heavy (forge pgrx PG-18 build) and may not converge;
  stubs (wiremock/nginx/tiny mock containers) = fast, proves the real agent binary +
  wiring without the sibling-build risk. **Recommendation: stubs for the first green
  smoke, real siblings as a follow-on change.** Operator to confirm at spec.
- **Smoke driver:** Playwright (operator named it — good for HTTP + any future UI) vs a
  Rust/`reqwest` or shell/`curl+jq` harness. Playwright fits the "automate via
  Playwright tools" instruction; confirm scope (HTTP-only now).
- **Where the compose lives:** repo root `compose.smoke.yml` + `Dockerfile`, or under
  a `deploy/`/`smoke/` dir. Lean: a dedicated `smoke/` dir to keep it separate from
  any future production compose.
- **Auth for the live smoke:** reuse the HS256 path (mint a bearer with the shared
  secret, as the p5 integration proof does) — no live IdP needed. Confirm.

## Explicitly out of scope (still deferred)

Postgres TLS (rustls connector — local/trusted network only in the smoke); per-operator
RLS; MCP multi-server; fabric WS subscriptions; OpenDesign; A2UI/React UI; Tauri;
knowledge-base. The full real-forge (pgrx) build is a **stretch** — attempt only if
resources/time allow after the stub-based smoke is green.
