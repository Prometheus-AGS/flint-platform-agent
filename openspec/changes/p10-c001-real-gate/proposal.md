## Why

Operator directive: **nothing but REAL** — the agent's gate surface (`GET
{admin}/routes`) must be exercised against the **real flint-gate**, not a stub. Gate is
actively developed (`feat/agent-authz-budget-rate-limiting`; recently added its own
`docker-compose smoke stack + Playwright E2E`), so we re-sync and reuse its real build.

## What Changes

- Reuse `../flint-gate`'s real build: `flint-gate/Dockerfile` (node:22 web-builder +
  `rust:1.90-bookworm` builder → debian-slim) + its compose services in our unified
  `compose.real.yml` (added in p10-c004): `flint-gate` (build), `postgres:16-alpine`,
  `kratos:v1.2`.
- Wire the real gate correctly (from gate's `docker-compose.yml`):
  - admin **:4457**, proxy :4456; mount `../flint-gate/config.example.yaml` →
    `/app/config/config.yaml`; `FLINT_GATE_JWT_SECRET`; `DATABASE_URL` → gate's postgres;
    healthcheck via `/dev/tcp/localhost/4456`.
  - The agent's `FPA_GATE_ADMIN_URL` → `http://flint-gate:4457`.
- No agent code change; the agent's `fpa-gate::list_routes` (`GET /routes`) now hits the
  **real** gate admin.

## Capabilities

### New Capabilities
- `real-gate`: The real flint-gate (built from source + its postgres + kratos) runs in the smoke stack; the agent's gate admin calls hit it live.

## Impact

- Contributes gate services to `compose.real.yml` (p10-c004); build context
  `../flint-gate`. No agent Rust change. Stock deps (postgres:16, kratos:v1.2) + gate's
  own build.

## Open Questions
- gate's build is heavy-ish (node web + rust:1.90). Time-boxed like the others; if it
  fails, that's a real finding to fix (this is the point of a real smoke).
