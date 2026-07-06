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

## Execute outcome (2026-07-06) — FULLY PROVEN

Standalone `smoke/compose.gate.yml` brings up the REAL gate (built from `../flint-gate`)
+ `postgres:16-alpine`, both healthy; the agent's exact hop `GET /routes` on the real
admin returns **HTTP 200** `{"routes":[],"source":"database"}`. No agent code change.

Key grounding correction (recorded in memory `gate-admin-auth-smoke-posture`): the stock
`config.example.yaml` binds admin to **loopback** (`127.0.0.1:4457`), unreachable from the
agent's separate container. Widening to `0.0.0.0` triggers gate's **fail-safe**
(`AdminAuthPosture::RefuseStart` — gate bails on a non-loopback admin bind without
`admin_auth`). gate's inbound JWT verifier is **JWKS-only** (no shared-secret HS256), so a
real IdP would be disproportionate. Resolution: a **smoke-owned** `config.smoke.yaml`
(`0.0.0.0` binds + `admin_auth.provider.type: anonymous`) under `smoke/gate-config/` —
gate boots under Enforce posture, admin is reachable, the read hop needs no credential.
Nothing written into `../flint-gate` (read-only build context + config mount); gate was
also actively dirty (another session), so read-only consumption was the right posture.

Kratos was dropped from the standalone file (the anonymous path exercises no session
auth). c005 folds these services (namespaced) into the unified `compose.real.yml`.

## Open Questions (resolved)
- ~~gate build heavy~~ — built fine (node web + rust:1.90), boots healthy.
- ~~admin reachability/auth~~ — resolved via smoke `anonymous` admin_auth on a 0.0.0.0
  bind (see above).
