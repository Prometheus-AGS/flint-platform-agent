## ADDED Requirements

### Requirement: The spec baseline reflects shipped phase-6 capabilities

After archival, `openspec/specs/` SHALL contain the phase-6 capability specs
(durable-project-store), and each archived p6 change MUST reside under
`openspec/changes/archive/`. No source code MUST change.

#### Scenario: p6 changes archived

- **WHEN** `p6-c001` and `p6-c002` are archived
- **THEN** their deltas appear in `openspec/specs/` and the change directories move under `openspec/changes/archive/`

#### Scenario: No code impact

- **WHEN** the archival runs
- **THEN** no Rust source file is modified
