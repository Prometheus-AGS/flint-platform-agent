## ADDED Requirements

### Requirement: project.create accepts and validates the nested aggregate

`project.create` SHALL accept a friendly-map input carrying the optional nested
collections (`applications`, `sub_agents`, `schemas`, `realtime`, `entity_meta`) and
build the full `Project` aggregate from them, persisting it whole via `ProjectStore`.
Nested values MUST be validated by typed deserialization; a malformed nested payload
MUST be rejected before any store write.

#### Scenario: Full nested payload is stored

- **WHEN** an authorized operator submits `project.create` with `name` plus one or more valid nested `applications`/`sub_agents`/`schemas`
- **THEN** the runner stores a `Project` containing those nested items and returns it

#### Scenario: Malformed nested payload is rejected

- **WHEN** `project.create` input has a nested item with a wrong-typed field
- **THEN** the runner returns an input/validation error and performs no store write

### Requirement: The server owns id and schema_version

`project.create` SHALL set `schema_version` to the current `SCHEMA_VERSION` regardless
of any client-supplied value, and MUST derive `id` from `project_id` when present or a
fresh UUID otherwise. A client MUST NOT be able to set `schema_version`.

#### Scenario: schema_version is server-controlled

- **WHEN** the input attempts to set `schema_version` to an arbitrary value
- **THEN** the stored project's `schema_version` is the current `SCHEMA_VERSION`, not the client value

### Requirement: Agent-owned aggregate writes route through TargetPort::Store

`project.create` SHALL be catalogued as `TargetPort::Store` and dispatched through the
store arm — not `TargetPort::Forge`. `TargetPort` MUST gain a `Store` variant for
agent-owned aggregate writes.

#### Scenario: project.create routes to the store arm

- **WHEN** `project.create` is dispatched
- **THEN** it is handled by the store dispatch arm and calls no forge/gate/fabric/mcp port
