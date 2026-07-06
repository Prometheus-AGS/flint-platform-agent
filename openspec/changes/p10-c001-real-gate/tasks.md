## 1. Real gate services (proven standalone in smoke/compose.gate.yml; folds into compose.real.yml at c005)

- [x] 1.1 `flint-gate` service: `build: { context: ../../flint-gate }` (gate's own Dockerfile — node:22 web + rust:1.90 → debian-slim); ports `4456`/`4457` (host 14456/14457 standalone); env `DATABASE_URL`, `FLINT_GATE_CONFIG=/app/config/config.yaml`, `RUST_LOG`; curl-based healthcheck on admin `/health`. PROVEN: builds + healthy.
- [x] 1.2 `gate-postgres` (`postgres:16-alpine`); gate `depends_on: gate-postgres (service_healthy)`. gate runs its OWN DB migrations at startup (`d.migrate()`) — no bootstrap needed. **Kratos omitted** (the smoke uses the `anonymous` admin-auth provider, so no Kratos session path is exercised; gate itself is fully real).
- [x] 1.3 Smoke-owned config `smoke/gate-config/config.smoke.yaml` (authored HERE, mounted read-only; NOTHING in ../flint-gate): `admin_listen: 0.0.0.0:4457` (agent is a separate container) + `admin_auth.provider.type: anonymous`. Required because gate REFUSES to start on a non-loopback admin bind without `admin_auth` (fail-safe); gate's inbound JWT is JWKS-only, so `anonymous` is the right smoke posture (no IdP). `FPA_GATE_ADMIN_URL` → `http://flint-gate:4457`.

## 2. Verification

- [x] 2.1 `docker compose -f smoke/compose.gate.yml build flint-gate` succeeds against the re-sync'd live gate (active branch `feat/agent-authz-budget-rate-limiting`). PROVEN.
- [x] 2.2 Gate healthy; the agent's exact hop `GET /routes` on the real admin returns **HTTP 200** `{"routes":[],"source":"database"}` (real DB, gate self-migrated); boot logs confirm `admin_listen=0.0.0.0:4457` + "admin API authentication enabled" (Enforce posture accepted the anonymous provider). PROVEN.
