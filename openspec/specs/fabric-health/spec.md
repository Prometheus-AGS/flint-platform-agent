# fabric-health Specification

## Purpose
TBD - created by archiving change p3-c002-fabric-health. Update Purpose after archive.
## Requirements
### Requirement: Fabric liveness via GET /healthz

`fpa-fabric::health` SHALL report realtime-fabric liveness by calling
`GET {endpoint}/healthz`, returning `Ok(())` on a success status.

#### Scenario: Fabric healthy

- **WHEN** the fabric gateway responds 2xx to `/healthz`
- **THEN** `health()` returns `Ok(())`

#### Scenario: Fabric unhealthy

- **WHEN** the fabric gateway responds with a non-success status
- **THEN** `health()` returns `PortError::Downstream`

#### Scenario: Fabric unreachable

- **WHEN** the fabric gateway cannot be reached
- **THEN** `health()` returns `PortError::Transport`, not a panic

