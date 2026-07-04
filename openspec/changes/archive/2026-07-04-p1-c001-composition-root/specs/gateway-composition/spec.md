## ADDED Requirements

### Requirement: Typed gateway configuration

The gateway SHALL load a typed `GatewayConfig` (forge URL, fabric endpoint, gate admin URL, bind address) from environment variables and MUST validate required values at startup, failing fast with descriptive context rather than starting with silent defaults for security-relevant URLs.

#### Scenario: Missing required configuration

- **WHEN** the gateway starts without a required configuration variable set
- **THEN** startup aborts with an error naming the missing variable and the process does not bind a port

#### Scenario: Valid configuration

- **WHEN** all required variables are present and well-formed
- **THEN** the gateway constructs `GatewayConfig` and proceeds to bind the configured address

### Requirement: Adapter construction and TaskRunner state

The gateway SHALL construct the four plane adapters from configuration, assemble `fpa_app::TaskRunner` from them, and inject it as shared Axum state so every protocol surface accesses the fabric through the same runner.

#### Scenario: State available to handlers

- **WHEN** a request reaches any protocol route (AG-UI, A2A, MCP)
- **THEN** the handler can access the shared `TaskRunner` via Axum `State`

### Requirement: Gate identity extraction without Ory

The gateway SHALL derive the operator identity solely from gate-injected credentials (gate-minted JWT or gate headers) and MUST NOT call Ory services or verify Ory JWKS directly.

#### Scenario: Gate-issued JWT present

- **WHEN** a request carries a valid gate-minted JWT
- **THEN** the gateway decodes it, maps claims to roles/permissions, and attaches an `OperatorContext` to the request

#### Scenario: No credentials

- **WHEN** a request carries no gate identity
- **THEN** the request is treated as unauthenticated and is not granted any authority
