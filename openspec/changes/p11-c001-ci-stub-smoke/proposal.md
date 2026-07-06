## Why

The agent's Rust CI (`fmt`/`clippy`/`test`/`msrv`) guards the code, but **nothing guards
the agent's runtime wiring** — that its protocol surfaces boot and its adapters make the
right HTTP hops. Phase 9 built a **self-contained stub smoke** (agent + postgres +
wiremock, no sibling repos, no secrets) that proves exactly this. It is CI-runnable today
(the agent builds from this repo alone — `frf-domain` was vendored in p10-c005), but it
is **not yet wired into CI**. Operator decision (analyze): stub smoke = the per-PR guard.

## What Changes

- Add a **`smoke` job** to `.github/workflows/ci.yml` (`ubuntu-latest`) that runs the
  stub smoke: `docker compose -f smoke/compose.smoke.yml up --build` → wait for the agent
  `/healthz` → run the Playwright stub spec → `down -v`. Reuse `smoke/run.sh` (it already
  encapsulates up→wait→smoke→teardown) rather than re-scripting in YAML.
- The job needs **Docker + Node** (both on `ubuntu-latest`) and **no secrets, no
  siblings** — the stub `compose.smoke.yml` builds the agent from this repo and stands up
  `postgres` + `wiremock` from public images.

## Capabilities

### New Capabilities
- `ci-stub-smoke`: Every PR runs the self-contained stub smoke in CI, guarding the agent's runtime wiring (boot + protocol-surface hops) — not just the Rust build.

## Impact

- `.github/workflows/ci.yml` (+ one job). No agent code change. No secrets. Reuses the
  existing `smoke/` stub assets unchanged.

## Open Questions
- **Playwright browser install:** the stub smoke is HTTP-only (`request` API) — confirm
  `run.sh` does not `npx playwright install chromium` (p9 removed it). If the CI job needs
  `@playwright/test` deps, `npm install` in `smoke/` first (no browsers).
- **Job timeout / flakiness:** set a sensible `timeout-minutes`; the stub is fast
  (seconds) but the agent image build adds a few minutes on a cold runner (cache later).
