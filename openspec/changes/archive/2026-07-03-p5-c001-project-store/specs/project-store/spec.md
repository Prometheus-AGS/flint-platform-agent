## ADDED Requirements

### Requirement: Project persists behind an agent-owned store port

`fpa-ports` SHALL define a `ProjectStore` port with `put(&Project) -> Result<(),
PortError>` and `get(&ProjectId) -> Result<Option<Project>, PortError>`, `Send +
Sync`. The `Project` aggregate MUST be stored whole (nested applications,
sub-agents, schemas, realtime, entity-meta) — never decomposed into a flat row.

#### Scenario: Store and retrieve a project

- **WHEN** a `Project` is `put` and then `get` by its `ProjectId`
- **THEN** the retrieved value equals the stored aggregate, including `schema_version` and all nested collections

#### Scenario: Missing project

- **WHEN** `get` is called with a `ProjectId` that was never stored
- **THEN** the store returns `Ok(None)` (not an error)

### Requirement: project.create writes to the store, not forge

The `project.create` task kind SHALL build a `Project` from the validated task
input and persist it via `ProjectStore.put`, returning the stored artifact. It MUST
NOT call `forge.create_entity` (the Project has no forge table).

#### Scenario: Authorized project.create

- **WHEN** an authorized operator submits `project.create` with valid input
- **THEN** the runner stores a `Project` (at the current `SCHEMA_VERSION`) via the store and returns it, calling no forge write

### Requirement: In-memory adapter is the interim backend

An `InMemoryProjectStore` SHALL implement `ProjectStore` using an interior-mutable
map, safe for concurrent access. A durable backend is explicitly out of scope this
phase and MUST remain swappable behind the port (composition-root change only).

#### Scenario: Concurrent access

- **WHEN** multiple tasks put/get projects concurrently
- **THEN** the adapter serializes access safely and never panics or corrupts state
