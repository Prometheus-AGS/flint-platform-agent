# project-artifact Specification

## Purpose
TBD - created by archiving change p1-c002-project-domain-model. Update Purpose after archive.
## Requirements
### Requirement: Versioned Project aggregate

The domain SHALL define a `Project` aggregate carrying a `schema_version` and collections of application definitions, sub-agent definitions, schema definitions, realtime parameters, and entity-management references. All public enums MUST be `#[non_exhaustive]` and all identifiers MUST be `#[repr(transparent)]` newtypes.

#### Scenario: Serde round-trip

- **WHEN** a `Project` value is serialized to JSON and deserialized back
- **THEN** the result equals the original value

#### Scenario: Forward compatibility

- **WHEN** a `Project` JSON contains an unrecognized enum variant in a `#[non_exhaustive]` field
- **THEN** deserialization succeeds without panic, preserving the value as unknown

### Requirement: Published JSON Schema

The `Project` artifact SHALL have a checked-in, versioned JSON Schema so external tools and generated applications can validate project documents.

#### Scenario: Sample validates

- **WHEN** a well-formed sample `Project` document is checked against `project.schema.json`
- **THEN** validation passes

### Requirement: A2UI registry conformance

`ComponentRef` within a `Project` SHALL reference a forge A2UI registry entry by id and version per `RFC-FORGE-A2UI-001` and MUST NOT embed component source or define an agent-local component vocabulary.

#### Scenario: Component reference shape

- **WHEN** a `Project` includes a UI component
- **THEN** it is stored as a registry id + version reference, not as inline component definition

