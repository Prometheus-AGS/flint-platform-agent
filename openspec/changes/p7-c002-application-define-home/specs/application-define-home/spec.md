## ADDED Requirements

### Requirement: application.define mutates the project aggregate in the store

`application.define` SHALL load the target project from `ProjectStore` by
`project_id`, upsert the supplied `ApplicationDef` into `project.applications`, and
persist the mutated aggregate. It MUST be catalogued `TargetPort::Store` and perform
no forge/gate write.

#### Scenario: Define an application in an existing project

- **WHEN** an authorized operator submits `application.define` with a valid `project_id` and an `ApplicationDef`
- **THEN** the project's `applications` list contains that application after the call, and a subsequent `get` returns it

#### Scenario: Upsert by application id

- **WHEN** `application.define` is called twice for the same project with the same application id but different contents
- **THEN** the project holds a single application with that id, reflecting the second definition (replace, not duplicate)

### Requirement: Unknown project is rejected

`application.define` SHALL return a clean error when no project exists for the given
`project_id`, and MUST NOT create a project implicitly.

#### Scenario: Unknown project_id

- **WHEN** `application.define` targets a `project_id` with no stored project
- **THEN** the runner returns a downstream/not-found error and stores nothing
