## Why

Operator directive: **one docker-compose that builds everything we need and puts it
together here to do real tests.** This change assembles the real gate (c001), real
forge (c002), real fabric (c003), and the agent — into a single `smoke/compose.real.yml`
— and a smoke that drives the whole thing, including a **realtime event** end-to-end
(via the new subscribe client, c004).

## What Changes

- **`smoke/compose.real.yml`** — the unified all-real stack:
  - **agent** (built from `smoke/Dockerfile`) on `:8088`, wired to the REAL planes:
    `FPA_GATE_ADMIN_URL` → real gate `:4457`, `FPA_FORGE_URL` → real `fdb-gateway`,
    `FPA_FABRIC_ENDPOINT` → real fabric gateway `:8080` (host 28080),
    `FPA_PROJECT_DB_URL` → a Postgres for the agent's own store, `FPA_GATE_JWT_KEY`.
  - **gate** stack (c001), **forge** stack + migrate + `fdb-gateway` (c002),
    **fabric** stack (c003). Service names namespaced (`gate-*`, `forge-*`, `fabric-*`)
    to avoid collisions; `depends_on: service_healthy` ordering; generous
    `start_period`s (fabric is heaviest).
- **`smoke/run-real.sh`** — one command: `up --build` → wait for every plane's health →
  run the real smoke → `down -v` (trap on success/failure). The stub `run.sh` +
  `compose.smoke.yml` (p9) stay as the fast path.
- **`smoke/smoke.real.spec.ts`** — the phase-9 smoke against the real stack, PLUS a
  **realtime event test**: agent subscribes (c004) → a change is driven (forge DB write
  or fabric `dev` trigger) → the agent receives the `ContentBlock` change event.

## Capabilities

### New Capabilities
- `compose-real-and-smoke`: A single compose stack builds + runs the real agent + real gate + real forge + real fabric, and an automated smoke drives them end-to-end including a realtime change event — the full-fidelity "nothing but real" proof.

## Impact

- New `smoke/compose.real.yml`, `smoke/run-real.sh`, `smoke/smoke.real.spec.ts` (+ any
  real-stack env). Reuses p9's Dockerfile/stub-runner. Heavy: builds gate + forge PG +
  fdb-gateway + fabric + agent. No agent Rust change beyond c004.

## Open Questions
- **Resource ceiling:** all planes at once on a 12 GiB VM is the real risk. Bring up in
  waves (gate+forge first, then fabric) if a single `up` OOMs; record it.
- **Realtime trigger:** forge DB write that CDC picks up, vs fabric's `dev` fan-out
  route. Decide at execute (lean: the `dev` route if it's the reliable trigger).
