# Execution — integration-proof-and-debt-closure

**Backend:** OpenSpec (KBD is source of truth; tasks tracked in
`openspec/changes/p5-c00N-*/tasks.md`, progress in `progress.json`).
**Dispatch:** in-session, agent-implemented (Rust workspace).
**Philosophy:** implementation-first — build c001→c003 fully, ONE batched
`cargo check` at the section boundary, then c004 + ONE `cargo test`.

## Order & contract

1. p5-c001-project-store — ProjectStore port + in-mem adapter + project.create rewire.
2. p5-c002-forge-rest-path-fix — `POST /{schema}/{table}`.
3. p5-c003-security-debt-closure — G3 gate guard, G4 agui auth, G5 JWKS single-flight, G6 audit flag.
   → **section boundary: one `cargo check`.**
   → **security-reviewer** on c003 before commit (mandatory, auth surface).
4. p5-c004-integration-proof — end-to-end test → **one `cargo test`** (milestone #1 of ≤3).

## Test-wait budget (this goal)

3 full `cargo test` runs max. Planned: 1 for c004's integration proof; 2 held for
re-runs if the proof surfaces integration gaps. `cargo check` is unlimited but
batched at section boundaries only.

## QA gate

artifact-refiner subsystem absent (all prior phases). c003 gets a mandatory
`security-reviewer` pass instead. Changes >3 files: c001, c003, c004.
