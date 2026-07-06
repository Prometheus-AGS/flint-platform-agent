## ADDED Requirements

### Requirement: CI runs the self-contained stub smoke on every PR

`.github/workflows/ci.yml` SHALL include a job that runs the stub smoke
(`smoke/compose.smoke.yml`: agent + postgres + wiremock) on `ubuntu-latest`, with no
secrets and no sibling repositories. The job MUST build the agent from this repository,
bring the stack up, exercise the agent's HTTP surfaces, and tear the stack down.

#### Scenario: PR triggers the stub smoke

- **WHEN** a pull request opens or updates against `main`
- **THEN** the `smoke` job builds + runs the stub smoke and reports pass/fail

#### Scenario: The smoke needs no secrets or siblings

- **WHEN** the `smoke` job runs
- **THEN** it uses only public images + this repo's sources (no cross-org clone, no secret)

### Requirement: The stub smoke job cleans up and fails the build on smoke failure

The job SHALL tear the stack down (`down -v`) regardless of outcome, and a failing smoke
MUST fail the CI job (non-zero exit).

#### Scenario: Smoke failure fails CI

- **WHEN** a stub-smoke assertion fails
- **THEN** the `smoke` job exits non-zero and the stack is torn down
