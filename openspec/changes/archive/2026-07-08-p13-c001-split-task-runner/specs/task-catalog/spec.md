## MODIFIED Requirements

### Requirement: The task runner stays under the file-size limit

The A2A task runner SHALL be organized so that no single source file exceeds the
project's 500-line hard limit. When the runner grows past that limit it MUST be split
into a directory module (`task_runner/mod.rs` + submodules) rather than remaining a
single oversized file. The split MUST preserve behavior and the public path
`crate::task_runner::TaskRunner`.

#### Scenario: Runner is split without behavior change

- **WHEN** the runner's single file exceeds 500 lines
- **THEN** it is converted to a directory module whose production file and test file are
  each under 500 lines
- **AND** every existing task-runner test still compiles and passes unchanged
- **AND** `crate::task_runner::TaskRunner` still resolves for all callers (gateway,
  in-crate use-cases)

#### Scenario: No public API drift from the split

- **WHEN** the runner is reorganized into a directory module
- **THEN** no public item is added, removed, or renamed by the split alone
- **AND** the composition root (`fpa-gateway`) requires no edit to keep compiling
