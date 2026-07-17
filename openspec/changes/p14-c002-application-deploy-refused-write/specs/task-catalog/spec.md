## MODIFIED Requirements

### Requirement: application.deploy is a declared refused write pending a gate write contract

`application.deploy` SHALL remain catalogued `TargetPort::Gate` with `required_role: "admin"`
and SHALL be classified as a Gate **write** by `is_gate_write_kind`. It SHALL refuse: every
invocation returns a `PortError::Downstream` error and MUST NOT return a success result — the
agent MUST NOT fabricate a deployment it did not perform (never fake green; Base Rule 5). The
refusal message SHALL name the missing dependency: there is no verified flint-gate admin
**write** endpoint (`GateAdmin` exposes `list_routes` only; `GateAdapter` implements
`GET /routes` only). flint-gate remains the only auth boundary — the agent never calls Ory.

A **real** `application.deploy` write is out of scope this phase because the gate write
contract is unverified. Implementing it later SHALL require, in order:

1. Verify a gate admin route-create / deploy endpoint and its payload shape against gate's
   own source (`../flint-gate`, read-only reference — author nothing there).
2. Add a write method to the `GateAdmin` port (a new port surface).
3. Implement it in `GateAdapter` with a wiremock test.
4. Add a real (non-empty) input schema for `application.deploy`.
5. Add a write arm to `dispatch_gate`.
6. Keep the `admin` role floor and the audit record.

#### Scenario: application.deploy refuses, never succeeds

- **WHEN** `application.deploy` runs with an admin role
- **THEN** it returns a `Downstream` error, not a success result
- **AND** the error message names the missing gate admin write contract

#### Scenario: application.deploy stays classified as a write

- **WHEN** the Gate classification guard runs
- **THEN** `application.deploy` remains classified as a write by `is_gate_write_kind`
- **AND** it is NOT reached by the Gate read (`list_routes`) branch
