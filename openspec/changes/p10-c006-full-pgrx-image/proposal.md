## Why

The forge CI PG-18 image (c002) gives the smoke a real forge with pgvector + pg_graphql
— enough to converge this phase. But the operator's assess directive is explicit: forge's
**Postgres extension code (pgrx) really needs to be built into the PG-18 images so we get
those features.** Forge's `images/postgres18/Dockerfile` builds the heavy pgrx stack
(`flint_llm` / `pg_net` / `pg_cron`, `shared_preload_libraries=...`). That build is slow
(rust:1.96 + `cargo pgrx init --pg18`) and higher-risk, so it is scoped as its own
**best-effort** change rather than gating the smoke.

## What Changes

- Add a **compose profile / override** that swaps the forge Postgres service from the CI
  image (c002) to forge's **full pgrx PG-18 image** (`../flint-forge/images/postgres18/
  Dockerfile`), with `shared_preload_libraries` and the pgrx extensions available.
- Document the build cost + the resource ceiling; keep it **opt-in** (not the default
  `run-real.sh` path) so the phase can converge on the CI image and still prove the pgrx
  image builds when explicitly invoked.
- No agent code change. This validates the fabric/LLM PG features are buildable into the
  images the agent's stack will eventually run against.

## Capabilities

### New Capabilities
- `full-pgrx-image`: The smoke stack can optionally run forge's full pgrx PG-18 image (flint_llm / pg_net / pg_cron), proving the fabric's Postgres extension features build into the images — the operator's "extensions built into PG-18" directive, as a best-effort opt-in.

## Impact

- New compose profile/override + docs under `smoke/`. Reuses forge's own
  `images/postgres18/Dockerfile` (do not fork it — Base Rule 3, reference it). Heavy
  build; opt-in. No agent Rust change.

## Execute outcome (2026-07-06) — best-effort, blocked upstream (documented)

`smoke/compose.pgrx.yml` authored + validated (references forge's own
`images/postgres18/Dockerfile`, no fork). The time-boxed build **did not even reach the
OOM ceiling** — it fails on a **forge bug**: forge's `.dockerignore` excludes `images/`,
yet `images/postgres18/Dockerfile` COPYs from `images/postgres18/{extensions,init,
init-baseline}/`, so those files are never in the build context (`COPY … 99-assert.sql:
not found`). The image is unbuildable from the forge repo as configured (the CI image
built fine because it has zero `COPY images/` lines).

Per Base Rule 40 (don't chase past the goal) + the established "read-only consumption of
siblings" discipline: did NOT fork forge's Dockerfile or edit forge's in-flight
`.dockerignore`. Appended the finding to **Know-Me-Tools/flint-forge#7** (with the fix:
narrow the ignore / add negations for the paths the pgrx Dockerfile needs). The CI image
(c002) remains the converging path; nothing on the green path depends on this.

## Resolved
- Build feasibility: blocked on a forge config bug (not an OOM); reported upstream. The
  smoke-side artifact is done + validated. Exercising `flint_llm` end-to-end remains a
  later phase.
