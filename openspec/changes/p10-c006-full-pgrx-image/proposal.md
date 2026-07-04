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

## Open Questions
- **Build feasibility on the dev VM** — the pgrx image is the heaviest build in the
  phase; if it OOMs/times out, record the ceiling and leave it as a documented
  best-effort (the CI image remains the converging path). Base Rule 40: don't expand
  past the goal chasing it.
- **Extension usage by the smoke** — this change only proves the image *builds/loads*;
  exercising `flint_llm` etc. end-to-end is a later phase, not here.
