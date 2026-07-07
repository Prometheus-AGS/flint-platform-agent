# Execution — forge-gateway-live

**Stage:** execute
**Backend:** `openspec` (changes under `openspec/changes/p11-c00*`).
**Date:** 2026-07-06

## Backend selection

OpenSpec, all 3 changes `--strict` valid. Pure CI/docs work — no agent Rust change.
The 3 changes are independent + low-risk, so this executes them together (implement all,
then verify), per the repo's implementation-first philosophy.

## Dispatch order (from plan.md)

1. **c001** ci-stub-smoke — add a `smoke` job to `.github/workflows/ci.yml` invoking
   `smoke/run.sh` (the stub runner is already CI-ready: docker check, up→wait→npm→smoke→
   `down -v` trap, browser-free, dies non-zero on fail).
2. **c003** make-readme — top-level `Makefile` + `smoke/README` update.
3. **c002** real-smoke-workflow — `.github/workflows/real-smoke.yml`, INERT
   (workflow_dispatch only, token-deferred, defaults non-forge) + README enablement note.

## Verification

- YAML: `actionlint` if available, else a syntax/render check + reasoning.
- c001: local dry-run of `smoke/run.sh` (the exact CI path) — the one live-provable change.
- c003: `make help` + `make -n` dry-runs.
- c002: dispatch-only (not push/PR), fail-fast without token, no secret value committed.

No artifact-refiner (no constraint file). QA = YAML lint + local dry-run + inspection.
