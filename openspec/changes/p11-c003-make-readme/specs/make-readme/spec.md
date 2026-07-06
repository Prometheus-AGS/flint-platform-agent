## ADDED Requirements

### Requirement: A Makefile exposes the smoke workflows as one-command targets

A top-level `Makefile` SHALL provide targets for the stub smoke, the real smoke, the
`--no-build` fast path, and the `--forge-full` variant, each delegating to the existing
`smoke/` scripts (no new behavior).

#### Scenario: Discoverable smoke targets

- **WHEN** a developer runs `make smoke` / `make smoke-real` / `make smoke-real-nobuild`
- **THEN** the corresponding `smoke/` script runs with the right flags

### Requirement: smoke/README documents the real smoke + the reliable workflow

`smoke/README` SHALL document `run-real.sh`, the `--no-build` workflow, the default vs
`forge-full` profile, and the CI stub job + opt-in real-smoke workflow.

#### Scenario: A developer can run the real smoke from the README alone

- **WHEN** a developer follows `smoke/README`
- **THEN** they can run the stub and real smokes and understand `--no-build` + `forge-full`
