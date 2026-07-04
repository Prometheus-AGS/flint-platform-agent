## Why

With the agent running as a container on `:8088` (p9-c002), we can finally drive the
**real running binary** end-to-end over HTTP — the live analogue of the p5
mock-boundary integration proof. This is the live-smoke deliverable the operator
asked for (automated via Playwright).

## What Changes

- **`smoke/smoke.spec.ts`** — a Playwright test (using its `request` API for HTTP;
  no browser needed for these JSON endpoints) that, against `http://localhost:8088`:
  1. `GET /healthz` → 200.
  2. **auth gate:** `GET /agui/stream` with no bearer → 401.
  3. mint an HS256 bearer with the shared smoke secret; then via A2A `POST /a2a/tasks`:
     `project.create` (name+project_id) → 200; `project.inspect` (same id) → returns
     it from the store; `project.list` → contains it; `fabric.health` → ok (hits the
     stub `/healthz`).
  4. **MCP:** `POST /mcp` `tools/list` → 200 with the tool set; `tools/call`
     `fabric.health` → ok.
  5. authenticated `GET /agui/stream` → opens (200 / SSE headers).
- **`smoke/package.json`** + Playwright config — minimal, pinned; a `smoke/run.sh`
  that brings the compose up, waits for `:8088/healthz`, runs the Playwright smoke,
  and tears down.
- **`smoke/README.md`** — updated with the full one-command flow.

## Capabilities

### New Capabilities
- `playwright-smoke`: An automated HTTP smoke that drives the live containerized agent end-to-end (authenticate → project CRUD via the store → fabric.health → MCP → agui auth), proving the real binary over the wire.

### Modified Capabilities

## Impact

- New `smoke/smoke.spec.ts`, `smoke/package.json`, Playwright config, `smoke/run.sh`;
  README update. Node/Playwright are dev-only (in `smoke/`, not the Rust workspace).

## Open Questions
- **Playwright vs a lighter curl/reqwest harness:** operator named Playwright — use its
  `request` fixture (HTTP), keeping the door open for real browser UI checks later.
- HS256 bearer minting in TS: a tiny `jsonwebtoken` (npm) sign with the shared secret,
  matching the agent's server-fixed HS256 verify (as the Rust p5 proof does).
