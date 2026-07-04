# Execution — project-read-and-list

**Backend:** OpenSpec (KBD source of truth). **Dispatch:** in-session.
**Philosophy:** implementation-first — c001 whole (port + both adapters + dispatch),
ONE batched `cargo check`, fast unit tests; c002 mechanical CLI.

## Order & contract

1. p8-c001 store-reads: `ProjectStore::list()` (fpa-ports) + in-mem (fpa-app) + Pg
   (fpa-store-pg); catalog retargets + dispatch_store inspect/list arms + STORE_KINDS
   test update.
   → **section boundary: one `cargo check`** + clippy + fmt.
   → fast unit tests (in-mem): inspect existing/unknown/no-forge, list all/empty/order.
   → `rust-reviewer` on the port+adapters+dispatch.
2. p8-c002 archive p6: `openspec archive` p6-c001/c002; verify.

## Test-wait budget

≤3 full `cargo test`. Planned: **1** run (all in-mem; Pg list rides the #[ignore]d
container test — not run, Docker down). 2 held.

## QA gate

artifact-refiner absent. c001 gets a `rust-reviewer` pass. c002 CLI-only (skip QA).
