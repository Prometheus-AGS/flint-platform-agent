## ADDED Requirements

### Requirement: The spec baseline reflects shipped phase-5 capabilities

After archival, `openspec/specs/` SHALL contain the capability specs shipped by the
four phase-5 changes (project-store, forge-rest-path, security-debt-closure,
integration-proof), and each archived change MUST reside under
`openspec/changes/archive/`. No source code MUST change as part of this archival.

#### Scenario: p5 changes are archived into the baseline

- **WHEN** `p5-c001`..`p5-c004` are archived
- **THEN** their `ADDED Requirements` deltas appear in `openspec/specs/` and the change directories move under `openspec/changes/archive/`

#### Scenario: No code impact

- **WHEN** the archival is performed
- **THEN** no Rust source file is modified and the build/tests are unaffected
