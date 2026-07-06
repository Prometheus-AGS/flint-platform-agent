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

## Execute outcome (2026-07-06) — health PROVEN

Standalone `smoke/compose.fabric.yml` (modeled on fabric's own `compose.ci.yml`) brings
up the REAL gateway + iggy + keto + postgres, all healthy; the agent's exact hop `GET
/healthz` → **HTTP 200** `{"status":"ok","version":"0.1.0"}`. Lighter than the full
compose.yml — surrealdb + flint-gate dropped (not needed for health), `CDC_ENABLED=false`.

Two fixes vs fabric's untracked (broken) `compose.ci.yml`, both smoke-side (nothing
written into ../flint-realtime-fabric): (1) keto v0.12 needs a config **file** — vendored
`smoke/fabric-config/keto.yml`; (2) `GATEWAY_JWKS_URL` is hard-required at config parse
even in dev — set a placeholder.

**Finding for c005:** an **iggy client/server version mismatch** (gateway's `iggy` client
crate vs `iggyrs/iggy:latest`) logs `invalid_command` / `Failed to get cluster metadata`.
The gateway degrades gracefully (health stays 200), but the realtime event bus rides on
iggy — c005's write→CDC→subscribe path may need a pinned iggy tag. Recorded.

## Open Questions (resolved / carried)
- ~~Resource weight~~ — standalone fabric ran fine on the 12 GiB VM (no OOM). The
  all-planes-at-once pressure is a c005 concern (wave-bringup fallback already specced).
- iggy version pin — carried to c005 (realtime path).
