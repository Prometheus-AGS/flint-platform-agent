## ADDED Requirements

### Requirement: project.inspect reads from the store

`project.inspect` SHALL be catalogued `TargetPort::Store` and return the project
aggregate from `ProjectStore.get` by `project_id`. It MUST NOT call forge. An unknown
`project_id` MUST return a clean error, not a forge lookup.

#### Scenario: Inspect an existing project

- **WHEN** `project.inspect` runs with a `project_id` for a stored project
- **THEN** it returns the whole aggregate from the store and calls no forge port

#### Scenario: Inspect an unknown project

- **WHEN** `project.inspect` runs with a `project_id` that has no stored project
- **THEN** it returns a downstream/not-found error and calls no forge port

### Requirement: ProjectStore exposes a list operation

The `ProjectStore` port SHALL provide `list() -> Result<Vec<Project>, PortError>`,
implemented by every adapter. It MUST return every stored project (whole aggregates).

#### Scenario: List returns all stored projects

- **WHEN** `list()` is called with N projects stored
- **THEN** it returns all N project aggregates

#### Scenario: Empty store

- **WHEN** `list()` is called with no projects stored
- **THEN** it returns an empty list (`Ok([])`), not an error

### Requirement: project.list reads from the store

`project.list` SHALL be catalogued `TargetPort::Store` and return the projects from
`ProjectStore.list()`. It MUST NOT call forge. The result SHALL be ordered
deterministically (by id).

#### Scenario: List projects via the store

- **WHEN** `project.list` runs with several projects stored
- **THEN** it returns them from the store (deterministically ordered), calling no forge port
