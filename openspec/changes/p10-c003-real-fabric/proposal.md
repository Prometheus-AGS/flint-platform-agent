## Why

Operator directive: **nothing but REAL** — this overrides the analyze lean to stub
fabric. The real fabric gateway hard-depends on iggy + keto + surrealdb (`depends_on:
service_healthy`), so "real fabric" means fabric's **full compose**. The agent probes
fabric via `GET /healthz`, but per the directive it must hit the **real** fabric.

## What Changes

- Bring fabric's real services into `compose.real.yml` (from
  `../flint-realtime-fabric/compose.yml`): the fabric **gateway** (`build`), plus its
  hard deps `iggy-server` (`iggyrs/iggy`), `keto` (`oryd/keto:v0.12`), `surrealdb`,
  and fabric's `postgres:17`. Wire the gateway's `IGGY_CONNECTION_STRING`,
  `KETO_BASE_URL`, and its `depends_on` healthchecks.
- Point the agent's `FPA_FABRIC_ENDPOINT` → the real fabric gateway (its `/healthz`).
- No agent code change.

## Capabilities

### New Capabilities
- `real-fabric`: The real flint-realtime-fabric gateway + its required services (iggy, keto, surrealdb, postgres) run in the smoke stack; the agent's `fabric.health` hits the real gateway.

## Impact

- Contributes fabric's ~6 services to `compose.real.yml` (p10-c004); build context
  `../flint-realtime-fabric`. Heavy (this is the weightiest plane). No agent Rust change.

## Open Questions
- **Resource weight:** fabric adds iggy + keto + surrealdb + postgres + the gateway
  build — on the 12 GiB VM with gate + forge also running, this is the highest-pressure
  point. Bring the stack up **incrementally / with generous healthcheck start_periods**;
  if a fabric dep OOMs, that's a real finding (record it; the directive is real, so we
  fix rather than stub).
