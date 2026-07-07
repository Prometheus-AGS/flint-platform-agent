## 1. Author the workflow (inert)

- [x] 1.1 `.github/workflows/real-smoke.yml`: `on: workflow_dispatch` with an input `forge_full` (boolean, default false). NO `schedule:` yet (commented, with a note).
- [x] 1.2 Steps: checkout this repo; clone siblings at pinned refs into the parent dir — `Prometheus-AGS/flint-realtime-fabric`, `Know-Me-Tools/flint-gate`, `Know-Me-Tools/flint-forge` — via `git clone` using `${{ secrets.SIBLING_CLONE_TOKEN }}`. Pinned refs as workflow env (bumpable).
- [x] 1.3 Fail-fast guard: if `SIBLING_CLONE_TOKEN` is empty, print the enablement pointer + exit 1 before any clone.
- [x] 1.4 Run `smoke/run-real.sh` (append `--forge-full` when the input is true); `down -v` on `if: always()`.

## 2. Document enablement

- [x] 2.1 `smoke/README`: a section "Enabling the nightly real smoke" — set `SIBLING_CLONE_TOKEN` (cross-org read PAT/App token) as a repo secret, then uncomment the `schedule:` block. Note the build cost + that forge gateway stays gated on flint-forge#7.

## 3. Verification

- [x] 3.1 Validate the workflow YAML (actionlint / syntax). It must NOT run on push/PR (dispatch-only).
- [x] 3.2 Reason through the fail-fast path (no token → clear exit 1) — cannot be run live without the secret, so verify by inspection + a local dry-run of the clone+run-real logic against the on-disk siblings.
- [x] 3.3 Confirm no secret value is committed (only a `${{ secrets.* }}` reference).
