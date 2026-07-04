## ADDED Requirements

### Requirement: The real flint-gate runs in the smoke stack

The smoke stack SHALL build and run the **real** flint-gate from `../flint-gate` (its
Dockerfile) alongside its required `postgres` and `kratos` services, exposing the gate
admin port. The agent's `FPA_GATE_ADMIN_URL` MUST point at the real gate admin.

#### Scenario: Real gate serves the admin API

- **WHEN** the smoke stack is up
- **THEN** the real flint-gate container is healthy and the agent's `application`/route
  read hits the real gate admin (`GET /routes`), not a stub

#### Scenario: Gate boots with its real dependencies

- **WHEN** flint-gate starts
- **THEN** it connects to its real postgres and kratos and passes its healthcheck before the agent depends on it
