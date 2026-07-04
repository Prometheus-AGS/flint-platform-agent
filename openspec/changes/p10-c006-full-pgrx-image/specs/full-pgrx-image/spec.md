## ADDED Requirements

### Requirement: The smoke stack can optionally run forge's full pgrx PG-18 image

An opt-in compose profile/override SHALL swap the forge Postgres service to forge's full
pgrx PG-18 image (`../flint-forge/images/postgres18/Dockerfile`) with its
`shared_preload_libraries` and pgrx extensions (`flint_llm`, `pg_net`, `pg_cron`). It
MUST reference forge's own Dockerfile (not fork it) and MUST NOT be the default
`run-real.sh` path — the phase converges on the CI image (c002).

#### Scenario: Opt-in pgrx image builds and loads

- **WHEN** the pgrx profile is explicitly invoked
- **THEN** the full pgrx PG-18 image builds and Postgres starts with the pgrx extensions preloaded

#### Scenario: Default path is unaffected

- **WHEN** `run-real.sh` runs without the pgrx profile
- **THEN** it uses the forge CI image (c002) and does not build the heavy pgrx image

### Requirement: The pgrx build cost and ceiling are documented

The change SHALL document the build cost and resource ceiling. If the pgrx image cannot
build on the dev VM, that ceiling is recorded and the CI image remains the converging
path (best-effort, per Base Rule 40 — do not expand past the goal chasing it).

#### Scenario: Build infeasible is recorded, not fatal

- **WHEN** the pgrx image build OOMs or times out on the dev VM
- **THEN** the ceiling is documented and the smoke still converges on the CI image
