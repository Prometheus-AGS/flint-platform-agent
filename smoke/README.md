# smoke/

Live smoke tests for the Flint Platform Agent — run the **real containerized agent** and
drive it end-to-end over HTTP. Two fidelities:

- **Stub smoke** (`run.sh` / `compose.smoke.yml`) — self-contained: agent + `postgres` +
  `wiremock`. No sibling repos, no secrets. The fast/reliable per-PR path (runs in CI).
- **Real-sibling smoke** (`run-real.sh` / `compose.real.yml`) — the agent against the
  **real** flint-gate + flint-realtime-fabric (+ optionally real forge). High-fidelity;
  needs the sibling repos as build contexts.

**Prereq:** a healthy Docker (this repo uses source-built colima+lima, vz driver).
If `docker` hangs or `docker info` errors, recover with `../scripts/reset-colima.sh`.

## One-command flows (via `make`, from the repo root)

```bash
make smoke                # stub smoke (self-contained; what CI runs per-PR)
make smoke-real           # real-sibling smoke — builds images (heavy)
make smoke-real-nobuild   # real smoke on PRE-BUILT images — the reliable path
make smoke-real-forge     # real smoke incl. the forge gateway (needs flint-forge#7)
```

Or call the scripts directly: `./smoke/run.sh`, `./smoke/run-real.sh [--no-build] [--forge-full]`.

## Real smoke: build once, then `--no-build` (important)

The 12 GiB dev VM runs the **8-service real stack fine** (~60s to all-healthy), but
`docker compose up --build` compiles the agent + fabric + gate **concurrently** and OOMs
the VM. The reliable workflow:

1. Build images **one service at a time** (they cache; source rarely changes):
   ```bash
   docker compose -f smoke/compose.real.yml build flint-gate
   docker compose -f smoke/compose.real.yml build fabric-gateway
   docker compose -f smoke/compose.real.yml build agent
   ```
2. Run the smoke on the pre-built images:
   ```bash
   make smoke-real-nobuild        # = ./smoke/run-real.sh --no-build
   ```

`run-real.sh` (default) builds serially then boots; `--no-build` skips the build step.

## Profiles: default vs `forge-full`

`compose.real.yml`'s **default** stack is **agent + real gate + real fabric + agent
Postgres** (the planes that serve the agent's HTTP hops — the p10-c005 5/5-green path).
The **forge** services (PG + bootstrap + gateway) are in the **`forge-full` profile**,
**off by default** — the forge gateway is blocked on **Know-Me-Tools/flint-forge#7**
(duplicate migration versions + `.dockerignore` excludes `images/`). Enable with
`make smoke-real-forge` / `run-real.sh --forge-full` once #7 lands.

## CI

- **Per-PR:** `.github/workflows/ci.yml` runs the **stub smoke** (self-contained, no
  secrets) — see the `smoke` job.
- **Opt-in real smoke:** `.github/workflows/real-smoke.yml` (`workflow_dispatch`) clones
  the siblings and runs the real smoke. It is **inert until you provision a secret** —
  see "Enabling the nightly real smoke" below.

### Enabling the nightly real smoke

The real smoke clones `Prometheus-AGS/flint-realtime-fabric`,
`Know-Me-Tools/flint-gate`, and `Know-Me-Tools/flint-forge` — the latter two are
**cross-org private** repos, so CI needs a token to read them.

1. Create a repo secret **`SIBLING_CLONE_TOKEN`** — a PAT or GitHub App token with read
   access to all three sibling repos (crosses the `Prometheus-AGS` ↔ `Know-Me-Tools`
   boundary).
2. Uncomment the `schedule:` block in `.github/workflows/real-smoke.yml` to run it
   nightly (until then it is `workflow_dispatch`-only and fails fast without the secret).

## Durable-store proof (real Postgres, standalone)

```bash
./smoke/run-durable-proof.sh   # fpa-store-pg durability test vs a real PG (testcontainers)
```
Exports `DOCKER_HOST` from the colima context so `testcontainers` can reach the daemon,
then runs `cargo test -p fpa-store-pg -- --ignored`.

## What's here

| File | Purpose |
|---|---|
| `Dockerfile` | Multi-stage build of the `fpa-gateway` binary (rust:1.93 → debian-slim). |
| `compose.smoke.yml` | **Stub:** agent (`:8088`) + `postgres:17` + `wiremock`. |
| `smoke.spec.ts` / `run.sh` | The stub Playwright smoke + its runner. |
| `compose.real.yml` | **Real:** agent + real gate + real fabric (+ `forge-full` profile). |
| `smoke.real.spec.ts` / `run-real.sh` | The real Playwright smoke + its runner (`--no-build`, `--forge-full`). |
| `compose.gate.yml` / `compose.fabric.yml` / `compose.forge.yml` | Standalone per-plane composes (each proven in isolation). |
| `gate-config/`, `fabric-config/`, `forge-bootstrap/` | Smoke-owned config/seed for the real planes (nothing written into sibling repos). |
| `fdb-gateway.Dockerfile` (+ `.dockerignore`) | Authored Dockerfile for forge's Quarry gateway (forge ships none). |
| `compose.pgrx.yml` | Best-effort opt-in build of forge's full pgrx PG-18 image (blocked on flint-forge#7). |
| `run-durable-proof.sh` | Standalone real-Postgres durability proof. |

## Notes

- `FPA_GATE_JWT_KEY` in the composes is a **throwaway HS256 test secret** — not a real
  credential.
- The real smoke reaches fabric's **real auth boundary** (Ory-JWKS + per-event Keto); it
  does not fake event receipt (proving that needs a real IdP — a future phase).
- Everything the real smoke needs from the siblings is **read-only** (build contexts +
  smoke-owned config under `smoke/`). **Nothing is written into the sibling repos.**
