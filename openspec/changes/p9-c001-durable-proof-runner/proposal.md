## Why

The `fpa-store-pg` durable/restart-survival test is `#[ignore]`d and, until this
phase, had never run — the durable store was "compiled + unit-tested" but not proven
against a real database (carried debt since phase 6). With Docker now up (source-built
colima+lima, vz), the test **passes against a real Postgres** (validated live: 7.56s,
put→get→list→restart) — but only when `DOCKER_HOST` points at colima's socket, which
the testcontainers crate needs and does not get from the docker CLI *context*. This
change captures that as a repeatable, documented runner so the proof is a one-command
operation, not tribal knowledge.

## What Changes

- Add `smoke/run-durable-proof.sh`: exports
  `DOCKER_HOST=$(docker context inspect colima --format '{{.Endpoints.docker.Host}}')`
  and `TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE=${DOCKER_HOST#unix://}`, then runs
  `cargo test -p fpa-store-pg -- --ignored --nocapture`. Fails loudly if Docker isn't
  reachable (points at `scripts/reset-colima.sh`).
- No Rust change — the `#[ignore]`d test already exists and passes; this makes running
  it reproducible.

## Capabilities

### New Capabilities
- `durable-proof-runner`: A one-command runner that runs the `fpa-store-pg` durability test against a real Postgres via testcontainers, with the colima `DOCKER_HOST` wiring baked in.

### Modified Capabilities

## Impact

- New `smoke/run-durable-proof.sh` (+ its mention in the smoke README). No Rust/Cargo
  change. No new deps (testcontainers is already a dev-dep from p6).

## Open Questions
- None. The invocation is already validated live.
