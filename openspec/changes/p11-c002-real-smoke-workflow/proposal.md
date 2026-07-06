## Why

The **real-sibling** smoke (p10-c005, 5/5 green locally) is higher-fidelity than the stub
but can't be a per-PR gate: **no self-hosted runner** exists, and `gate` + `forge` live in
a **different GitHub org** (`Know-Me-Tools`) than this repo (`Prometheus-AGS`), so cloning
them in CI needs a cross-org token. Operator decisions (analyze): the real smoke is an
**opt-in `workflow_dispatch`/nightly**, and the **token is deferred** — author the workflow
now but leave it **inert** (dispatch-only, documented secret) so it can be switched on
later by adding the secret.

## What Changes

- Add `.github/workflows/real-smoke.yml`:
  - `on: workflow_dispatch` **only** (NO `schedule:` yet — inert until the token exists).
  - Steps: checkout this repo; **clone the 3 siblings at pinned refs** into `../` using
    `${{ secrets.SIBLING_CLONE_TOKEN }}` (a cross-org PAT/App token — NOT set yet);
    ensure Docker; run `smoke/run-real.sh` (or `--no-build` if images are pre-warmed).
  - The three sibling repos + pinned refs are declared as workflow env / inputs:
    `Prometheus-AGS/flint-realtime-fabric`, `Know-Me-Tools/flint-gate`,
    `Know-Me-Tools/flint-forge`.
- **Document the enablement** in `smoke/README`: set `SIBLING_CLONE_TOKEN` (repo secret,
  cross-org read) → uncomment the `schedule:` (nightly) to activate. Until then the
  workflow is manually dispatchable but will fail fast without the secret (a clear error).

## Capabilities

### New Capabilities
- `real-smoke-workflow`: An opt-in GitHub Actions workflow that clones the real siblings and runs the full real-sibling smoke — authored + inert (dispatch-only, token-deferred), ready to enable by adding one secret.

## Impact

- New `.github/workflows/real-smoke.yml` + a `smoke/README` enablement note. No agent code
  change. No secret is committed — the workflow *references* a not-yet-set secret.

## Open Questions
- **Sibling pinned refs:** all three siblings are actively dirty / on feature branches.
  Pin to specific commits or track a branch? (Lean: pin to a known-good commit per sibling,
  bumpable — a moving target defeats reproducibility.)
- **forge in the real workflow:** with `--forge-full` gated on flint-forge#7, the workflow
  should default to the **non-forge** default profile (agent + gate + fabric — the 5/5
  green path). `--forge-full` becomes a workflow input, off by default.
- **Build budget on a stock runner:** cloning + building gate + fabric + agent from
  source is heavy; the runner may need a large timeout or a cached-image step. Acceptable
  for a nightly; documented.
