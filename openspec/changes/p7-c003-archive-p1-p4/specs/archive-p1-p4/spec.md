## ADDED Requirements

### Requirement: The spec baseline reflects shipped p1–p4 capabilities

After archival, `openspec/specs/` SHALL contain the capability specs of the completed
p1–p4 changes, and each archived change MUST reside under
`openspec/changes/archive/`. An incomplete change MUST NOT be force-archived — any
partial is reconciled first or explicitly skipped and noted.

#### Scenario: Complete p1–p4 changes are archived

- **WHEN** the `✓ Complete` p1–p4 changes are archived
- **THEN** their deltas appear in `openspec/specs/` and the change directories move under `openspec/changes/archive/`

#### Scenario: A partial change is not force-archived

- **WHEN** a p1–p4 change still shows incomplete tasks and is not reconciled
- **THEN** it is left in the active list (skipped + noted), not archived

#### Scenario: No code impact

- **WHEN** the archival runs
- **THEN** no Rust source file is modified
