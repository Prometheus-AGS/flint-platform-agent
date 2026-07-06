## Why

Phase 10 produced `smoke/run-real.sh` (+ the `--no-build` fast path, `--forge-full`
profile, `compose.real.yml`) but there is **no task runner** and `smoke/README` only
documents the stub `run.sh`. A developer can't discover the real smoke or the reliable
`--no-build` workflow without reading the scripts. Goal 2 explicitly names a "make target"
as an option; a tiny Makefile + a README update makes the whole smoke surface discoverable
and one-command.

## What Changes

- Add a top-level **`Makefile`** with focused targets:
  - `smoke` — the stub smoke (`smoke/run.sh`).
  - `smoke-real` — the real smoke (`smoke/run-real.sh`), builds images.
  - `smoke-real-nobuild` — `smoke/run-real.sh --no-build` (the reliable path once images
    exist — the phase-10 lesson: the VM runs the stack fine, only concurrent builds OOM).
  - `smoke-real-forge` — `smoke/run-real.sh --forge-full` (gated on flint-forge#7).
- Update **`smoke/README`** to document: `run-real.sh`, the `--no-build` workflow (build
  images once per-service, then boot), the default vs `forge-full` profile, and a pointer
  to the CI stub job + the opt-in real-smoke workflow.

## Capabilities

### New Capabilities
- `make-readme`: One-command `make` targets + updated `smoke/README` make the full smoke surface (stub, real, `--no-build`, `forge-full`) discoverable and reproducible.

## Impact

- New `Makefile`; `smoke/README` updated. No agent code change. Documentation + a thin
  convenience wrapper over the existing scripts (no new behavior).

## Open Questions
- **Make vs just:** the analyze noted "GH Actions vs a make target." Makefile is the
  lowest-friction, no-install choice (present on macOS/Linux). Confirm Make (not `just`).
