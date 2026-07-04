## ADDED Requirements

### Requirement: A one-command runner proves durability against real Postgres

`smoke/run-durable-proof.sh` SHALL run the `fpa-store-pg` `#[ignore]`d durability
test against a real Postgres container, wiring `DOCKER_HOST` (and the socket override)
from the active colima context so the testcontainers crate can reach the daemon. The
runner MUST fail with a clear message when Docker is unreachable.

#### Scenario: Durable proof runs green

- **WHEN** Docker is healthy and `smoke/run-durable-proof.sh` runs
- **THEN** the `fpa-store-pg` durability test executes against a real Postgres (put → fresh pool → get → list → restart-survival) and passes — not `ignored`

#### Scenario: Docker unreachable

- **WHEN** the runner runs with no reachable Docker daemon
- **THEN** it exits non-zero with a message pointing at `scripts/reset-colima.sh`
