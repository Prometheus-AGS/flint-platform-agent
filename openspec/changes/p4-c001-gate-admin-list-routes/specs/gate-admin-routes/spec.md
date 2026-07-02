## ADDED Requirements

### Requirement: Real gate route listing

`fpa-gate::list_routes` SHALL retrieve the configured routes from gate's admin API
over HTTP, forwarding the admin bearer, replacing the not-implemented stub.

#### Scenario: Routes listed

- **WHEN** gate's admin API returns the route list
- **THEN** `list_routes` returns that list

#### Scenario: Unauthorized

- **WHEN** gate responds 401 or 403
- **THEN** `list_routes` returns `PortError::Unauthorized`

#### Scenario: Gate unreachable

- **WHEN** the gate admin API cannot be reached
- **THEN** `list_routes` returns `PortError::Transport`, not a panic

### Requirement: Admin path is configuration, not a hardcoded guess

The gate admin base URL/path SHALL come from configuration so the admin path can be
corrected without a code change.

#### Scenario: Configured base URL

- **WHEN** the gate admin base URL is provided via config
- **THEN** the adapter builds the routes request from it (no hardcoded host/prefix)

### Requirement: No route writes this phase

`fpa-gate` SHALL NOT create, update, or delete gate routes in this phase.

#### Scenario: Deploy is refused

- **WHEN** a route-write (e.g. `application.deploy`) is dispatched to the gate plane
- **THEN** it returns a handled "not implemented this phase" error and performs no write
