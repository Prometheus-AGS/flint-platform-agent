## ADDED Requirements

### Requirement: The real fabric gateway runs with its required services

The smoke stack SHALL run the **real** flint-realtime-fabric gateway (from
`../flint-realtime-fabric`) together with its hard dependencies (iggy, keto, surrealdb,
postgres), and the agent's `FPA_FABRIC_ENDPOINT` MUST point at the real gateway. Fabric
is NOT stubbed.

#### Scenario: Real fabric health

- **WHEN** the fabric gateway and its deps are healthy
- **THEN** the agent's `fabric.health` (`GET /healthz`) hits the real fabric gateway and returns ok

#### Scenario: Fabric deps satisfied at boot

- **WHEN** the fabric gateway starts
- **THEN** iggy + keto + surrealdb + postgres are up (per fabric's `depends_on` healthchecks) so the gateway boots successfully
