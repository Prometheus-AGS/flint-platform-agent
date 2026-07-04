## 1. Opt-in pgrx compose

- [ ] 1.1 Add a `compose.pgrx.yml` override (or a `pgrx` profile in `compose.real.yml`) that points the forge Postgres service `build.context`/`dockerfile` at `../flint-forge/images/postgres18/Dockerfile` (reference forge's file, do not fork). Confirm the build context path resolves from `smoke/`.
- [ ] 1.2 Ensure `shared_preload_libraries=pg_net,pg_cron,ext_flint_llm` is set for that service (from forge's image config).

## 2. Keep it best-effort / opt-in

- [ ] 2.1 `run-real.sh` default path uses the CI image (c002); pgrx is a separate invocation (e.g. `run-real.sh --pgrx` or a documented `docker compose -f ... --profile pgrx up`).
- [ ] 2.2 Document build cost + resource ceiling in `smoke/README` (or a comment block): this is the heaviest build in the phase.

## 3. Verification (best-effort)

- [ ] 3.1 Attempt the pgrx image build on the dev VM. If it builds + Postgres starts with the extensions preloaded → record success.
- [ ] 3.2 If it OOMs/times out → record the ceiling; the CI image remains the converging path (Base Rule 40 — do not chase it past the goal). No agent code depends on this.
