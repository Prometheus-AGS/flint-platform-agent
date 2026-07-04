## ADDED Requirements

### Requirement: The agent builds into a container image

`smoke/Dockerfile` SHALL build the `fpa-gateway` binary in a `rust:1.93`-based
builder stage and produce a slim runtime image that runs the binary bound to
`0.0.0.0:8088`. The image MUST NOT require the Rust toolchain at runtime.

#### Scenario: Image builds and runs

- **WHEN** `smoke/Dockerfile` is built
- **THEN** the resulting image starts `fpa-gateway` listening on `:8088`

### Requirement: A compose stack stands the agent up with satisfied dependencies

`smoke/compose.smoke.yml` SHALL run the agent alongside a `postgres` container and a
wiremock stub container, providing every required env var
(`FPA_FORGE_URL`, `FPA_FABRIC_ENDPOINT`, `FPA_GATE_ADMIN_URL`, `FPA_PROJECT_DB_URL`,
`FPA_GATE_JWT_KEY`) so the agent boots, and publishing `8088` to the host.

#### Scenario: Stack comes up healthy

- **WHEN** `docker compose -f smoke/compose.smoke.yml up` runs
- **THEN** postgres and the stub become healthy, the agent boots without a missing-config abort, and `GET http://localhost:8088/healthz` returns 200

### Requirement: The dependency stub answers the three adapter probes

The wiremock stub SHALL answer `GET /openapi.json` (200, an OpenAPI-shaped body),
`GET /healthz` (200), and `GET /routes` (200 JSON) so the agent's forge/fabric/gate
adapters succeed without real siblings.

#### Scenario: Stubbed dependency reads succeed

- **WHEN** the agent calls its forge/fabric/gate adapters against the stub
- **THEN** each probe returns a success the adapter accepts (no Transport/Downstream error from a missing dependency)
