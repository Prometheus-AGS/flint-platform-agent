## ADDED Requirements

### Requirement: An opt-in real-smoke workflow exists, authored but inert

`.github/workflows/real-smoke.yml` SHALL run the full real-sibling smoke on
`workflow_dispatch`. It MUST clone the three sibling repositories at pinned refs using a
cross-org token secret and run `smoke/run-real.sh`. It MUST NOT be enabled on a schedule
until the token secret is provisioned (documented in `smoke/README`).

#### Scenario: Manual dispatch with the token set

- **WHEN** `SIBLING_CLONE_TOKEN` is set and the workflow is dispatched
- **THEN** it clones gate/fabric/forge at their pinned refs and runs the real smoke

#### Scenario: No token → fails fast with a clear message

- **WHEN** the workflow is dispatched without `SIBLING_CLONE_TOKEN`
- **THEN** it fails early with a message pointing to the `smoke/README` enablement note (no partial run)

### Requirement: The workflow defaults to the non-forge real stack

The workflow SHALL default to the converging default profile (agent + real gate + real
fabric — the p10-c005 5/5 path); `--forge-full` is an opt-in input, off by default (the
forge gateway is blocked on flint-forge#7).

#### Scenario: Default dispatch excludes the forge gateway

- **WHEN** the workflow runs with default inputs
- **THEN** it runs the agent + gate + fabric stack, not the forge gateway
