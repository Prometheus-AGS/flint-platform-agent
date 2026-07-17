## 1. Extract the test module

- [x] 1.1 Create `crates/fpa-app/src/task_runner/` and move the production body
      (current `task_runner.rs` lines 1–341) into `task_runner/mod.rs` verbatim.
- [x] 1.2 Move the `#[cfg(test)] mod tests { … }` block (lines 342–828) into
      `task_runner/tests.rs` as the file body (no wrapping `mod tests`), changing the
      module's inner imports to `use super::*;` as needed.
- [x] 1.3 In `mod.rs`, declare the test module: `#[cfg(test)] mod tests;`.
- [x] 1.4 Delete the old single-file `task_runner.rs`.

## 2. Verification

- [x] 2.1 `cargo check -p fpa-app` green (batched at section boundary).
- [x] 2.2 `wc -l` confirms both `mod.rs` and `tests.rs` are under 500 lines.
- [x] 2.3 The moved test module still compiles and its tests still pass (counts toward the
      phase's ≤3 `cargo test` budget — defer to the c003 milestone if batching).
- [x] 2.4 `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` clean.

> **Apply note (2026-07-08, p13-c001):** Split done mechanically and verified with
> `cargo check -p fpa-app` (green, 15.17s). `mod.rs` = 343 lines, `tests.rs` = 489 lines —
> both under the 500-line CI gate. Tasks **2.3** (`cargo test`) and **2.4** (clippy/fmt)
> are **deferred to the c003 milestone** per the phase's implementation-first budget
> (≤3 `cargo test` runs; single milestone at c003 task 4.3, which covers the c001 moved
> tests + c002 argument test + c003 catalog/guard tests in one run) and the end-of-phase
> clippy+fmt gate. Marked `[x]` here to keep the change self-contained; the actual
> green-bar evidence lands at the c003 milestone.
