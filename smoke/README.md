# smoke/

Live smoke test for the Flint Platform Agent — runs the **real containerized agent**
against real infrastructure (a real Postgres + HTTP-stubbed plane dependencies) and
drives it end-to-end over HTTP.

**Prereq:** a healthy Docker (this repo uses source-built colima+lima, vz driver).
If `docker` hangs or `docker info` errors, recover with `../scripts/reset-colima.sh`.

## One-command flow

```bash
./smoke/run.sh          # build + up -> wait for :8088 -> Playwright smoke -> down -v
```

## Durable-store proof (real Postgres, standalone)

```bash
./smoke/run-durable-proof.sh   # fpa-store-pg durability test vs a real PG (testcontainers)
```
Exports `DOCKER_HOST` from the colima context so the `testcontainers` crate can reach
the daemon, then runs `cargo test -p fpa-store-pg -- --ignored`.

## What's here

| File | Purpose |
|---|---|
| `Dockerfile` | Multi-stage build of the `fpa-gateway` binary (rust:1.93 → debian-slim). |
| `compose.smoke.yml` | agent (`:8088`) + `postgres:17-alpine` + `wiremock` stub. |
| `stubs/mappings/*.json` | The 3 adapter probes the stub answers: forge `/openapi.json`, fabric `/healthz`, gate `/routes`. |
| `smoke.spec.ts` | Playwright HTTP smoke driving `:8088`. |
| `run.sh` | Orchestrates up → wait → smoke → teardown. |
| `run-durable-proof.sh` | The standalone real-Postgres durability proof. |

## Notes

- **Dependencies are stubs**, not real forge/gate/fabric — this proves the real agent
  binary + wiring + `:8088`, not sibling wire-compat (a future phase). The stub surface
  is 3 static GETs.
- `FPA_GATE_JWT_KEY` in the compose is a **throwaway test secret** (HS256) the smoke
  mints its bearer with — not a real credential.
- Manual bring-up/tear-down:
  `docker compose -f smoke/compose.smoke.yml up --build` / `down -v`.
