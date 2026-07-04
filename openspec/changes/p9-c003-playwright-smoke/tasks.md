## 1. Playwright project (in smoke/)

- [x] 1.1 `smoke/package.json` — `@playwright/test` (pinned) + `jsonwebtoken` (HS256 mint), dev-only; a `test` script.
- [x] 1.2 `smoke/playwright.config.ts` — `use.baseURL = http://localhost:8088`; no browser projects needed (HTTP via `request`), reporters minimal.

## 2. The smoke test

- [x] 2.1 `smoke/smoke.spec.ts` — helper: mint HS256 bearer (`jsonwebtoken.sign({sub,roles,exp}, SMOKE_SECRET)`), matching the agent's HS256 verify.
- [x] 2.2 `GET /healthz` → 200.
- [x] 2.3 unauthenticated `GET /agui/stream` → 401; unauthenticated `POST /a2a/tasks` → 401.
- [x] 2.4 authed A2A: `project.create` (name+project_id) → ok; `project.inspect` → returns it; `project.list` → contains it.
- [x] 2.5 authed A2A `fabric.health` → ok (hits stub `/healthz`).
- [x] 2.6 `POST /mcp` `tools/list` → 200; `tools/call` `fabric.health` → ok.
- [x] 2.7 authed `GET /agui/stream` → 200 / SSE.

## 3. Runner

- [x] 3.1 `smoke/run.sh`: `docker compose -f smoke/compose.smoke.yml up --build -d` → poll `:8088/healthz` (timeout) → `npm --prefix smoke test` → `trap` `docker compose … down -v` on exit (success OR failure). Export the same SMOKE_SECRET the compose uses.
- [x] 3.2 `smoke/README.md`: the one-command flow (`./smoke/run.sh`) + `smoke/run-durable-proof.sh` (from p9-c001).

## 4. Verification (the phase's live milestone)

- [x] 4.1 `bash -n smoke/run.sh`; `docker compose -f smoke/compose.smoke.yml config` valid.
- [x] 4.2 **Run `./smoke/run.sh` — the Playwright smoke passes against the live containerized agent** (the phase's one heavy end-to-end run). Fix any wire drift the mocks hid; record the pass + any drift in the reflection.
- [x] 4.3 Clean teardown verified (no leftover containers/volumes).
