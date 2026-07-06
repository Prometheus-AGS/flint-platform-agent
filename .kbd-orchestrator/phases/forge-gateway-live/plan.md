# Plan — forge-gateway-live

**Phase:** forge-gateway-live
**Stage:** plan
**Date:** 2026-07-06
**Backend:** OpenSpec (`openspec/changes/p11-c00*`), 3 changes, all `--strict` valid.

## Scope note (carried from assess/analyze)

Goal 1 (boot the real forge gateway) is **BLOCKED on flint-forge#7** (all 3 forge bugs
still open on the live tree) — **out of scope this phase**, gated behind `--forge-full`,
ready to flip when forge fixes it. This phase = **CI wiring + discoverability** (goal 2),
per the operator's 3 decisions (stub per-PR + opt-in nightly real; defer the token; defer
realtime-receipt).

## Change ordering (value-first; all independent, low-risk)

The three changes have **no dependencies** on each other and touch disjoint files
(`.github/workflows/ci.yml`, a new workflow, `Makefile`/`smoke/README`). Ordered by value
delivered soonest + logical foundation:

| Order | Change | Why here | Touches | Risk | Agent |
|---|---|---|---|---|---|
| 1 | **p11-c001-ci-stub-smoke** | Highest value — a REAL per-PR guard live in CI immediately; the honest self-contained path (no siblings/secrets). | `.github/workflows/ci.yml` (+1 job) | Low (stub proven in p9; needs a green CI run to confirm on the runner) | devops-engineer |
| 2 | **p11-c003-make-readme** | Documents the WHOLE smoke surface (stub + real + `--no-build` + profiles); land before c002 so the README can reference both the CI job and the workflow. | new `Makefile`, `smoke/README` | Trivial (thin wrappers + docs) | doc-updater |
| 3 | **p11-c002-real-smoke-workflow** | Inert/token-deferred — no live effect until a secret is added, so lowest urgency. Authored + validated + documented enablement. | new `.github/workflows/real-smoke.yml`, `smoke/README` note | Low-med (YAML correctness; can't run live without the token — verify by inspection + local dry-run of the clone+run logic) | devops-engineer |

## Verification strategy (per change)

- **c001:** actionlint/syntax + a **local dry-run of `smoke/run.sh`** (the exact path the
  job runs) confirms green with the current tree; the first PR/push exercises it on the
  runner. This is the one change with a live-CI proof.
- **c003:** `make help` + `make -n <target>` (dry-run) show correct commands; README renders.
- **c002:** actionlint/syntax; **dispatch-only** (must not trigger on push/PR); fail-fast
  path (no token → exit 1) verified by inspection; a **local dry-run** of the clone +
  `run-real.sh` logic against the on-disk siblings; no secret value committed.

## Risks / notes

- **c001 on the runner:** the agent image build adds a few minutes on a cold runner (no
  cache yet). Acceptable; a build-cache step is a possible follow-up, not this phase.
- **c002 is honestly inert** — it ships correct-but-unrun until the operator adds
  `SIBLING_CLONE_TOKEN`. `tasks.md` says so; the README documents enablement. Do not
  fake a green run for it.
- **No agent Rust change in the whole phase** — pure CI/docs. The Rust CI stays green
  (frf-domain vendored; no cross-repo deps).

## Success criteria (phase-level)

1. Every PR runs the stub smoke in CI (c001) and it passes.
2. `make` targets + `smoke/README` make the stub, real, `--no-build`, and `forge-full`
   flows discoverable and one-command (c003).
3. `real-smoke.yml` exists, validates, is dispatch-only, defaults non-forge, and is
   documented as token-gated (c002) — ready to enable by adding one secret.
4. Rust CI stays green; no agent code change.

## First change to apply

**p11-c001-ci-stub-smoke** — `/kbd-execute forge-gateway-live` then `/kbd-apply` walks its
tasks (add the `smoke` job to ci.yml, local dry-run `smoke/run.sh`, actionlint).
