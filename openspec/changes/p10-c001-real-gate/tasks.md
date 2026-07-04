## 1. Real gate services (into compose.real.yml — authored in p10-c004)

- [ ] 1.1 `flint-gate` service: `build: { context: ../../flint-gate }` (path from `smoke/`); ports `4456`/`4457`; env `DATABASE_URL`, `FLINT_GATE_JWT_SECRET`, `FLINT_GATE_CONFIG=/app/config/config.yaml`, `RUST_LOG`; volume `../../flint-gate/config.example.yaml:/app/config/config.yaml:ro`; healthcheck `/dev/tcp/localhost/4456`.
- [ ] 1.2 `gate-postgres` (`postgres:16-alpine`) + `kratos` (`oryd/kratos:v1.2 serve all --dev`) per gate's compose; gate `depends_on: gate-postgres (service_healthy)`.
- [ ] 1.3 Point the agent's `FPA_GATE_ADMIN_URL=http://flint-gate:4457`.

## 2. Verification

- [ ] 2.1 `docker compose -f smoke/compose.real.yml build flint-gate` succeeds (re-sync'd latest gate).
- [ ] 2.2 Gate container becomes healthy; `curl` the agent, exercise the gate-read hop against the real admin.
