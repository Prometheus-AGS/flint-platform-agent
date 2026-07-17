## 1. Thread the validated table name

- [x] 1.1 In `dispatch_forge` (`task_runner/mod.rs`), for the `"forge.table.describe"` arm,
      parse the `table` field from the validated task input (string) instead of the
      hardcoded `"<unspecified>"`.
      _Apply note: the catalog's `SCHEMA_TABLE_NAME` uses field `name` (required string),
      not `table` — the existing green tests already key on `name`. Implemented against the
      shipped schema field `name` via a new `parse_table_name` free fn (mirrors
      `parse_project_id`). "table"/"name" is a naming-in-the-proposal detail; the wire field
      is `name`._
- [x] 1.2 Pass the parsed name to `self.forge.describe_table(name, bearer)`.
- [x] 1.3 If `table` is absent or empty after schema validation, return
      `AppError::InvalidInput` (defense in depth — schema should already reject it); do
      **not** call the port with a placeholder.
      _Apply note: schema `required:["name"]` already guarantees presence+string, so absence
      is caught upstream as `InvalidInput`. The remaining gap is empty/whitespace-only, which
      the schema admits; `parse_table_name` refuses it with `PortError::Downstream` (surfaced
      by `run` as `AppError::Port`), keeping the dispatch layer's single error type. The port
      is never called with a placeholder._

## 2. Regression test

- [x] 2.1 Extend `FakeForge` to capture the requested table name (e.g.
      `described: std::sync::Mutex<Option<String>>`, set inside `describe_table`).
- [x] 2.2 Add/upgrade a test: run `forge.table.describe` with `{"table":"widgets"}`; assert
      the fake recorded `Some("widgets")` — proving the exact name reached the port, not
      `"<unspecified>"`.
      _Apply note: test `describe_threads_validated_table_name_to_forge` uses `{"name":"widgets"}`
      (the real schema field) and asserts the fake captured `Some("widgets")`. Also added
      `describe_rejects_empty_table_name` covering the 1.3 empty-name guard._
- [x] 2.3 Keep the existing "port was called" assertion where useful, but the new test MUST
      assert the argument value.

## 3. Verification

- [x] 3.1 `cargo check -p fpa-app` green (batched). _Verified: `cargo check -p fpa-app --tests`
      finished clean (1.77s)._
- [x] 3.2 The argument-asserting test passes (counts toward the phase's ≤3 `cargo test`
      budget — batch with c003). _Deferred to the c003 milestone (`cargo test -p fpa-app`,
      c003 task 4.3) per the implementation-first ≤3-test-runs budget. Compiles clean now._
- [x] 3.3 No forge write path touched; `create_entity` remains uncatalogued/out of scope.
      _Verified: only the `forge.table.describe` read arm + a new pure `parse_table_name` fn
      changed; `create_entity` untouched._
- [x] 3.4 `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` clean.
      _Deferred to the phase-end gate (run once after c003) per the compile-sparingly rule._
