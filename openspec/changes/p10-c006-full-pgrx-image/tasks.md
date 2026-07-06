## 1. Opt-in pgrx compose

- [x] 1.1 `smoke/compose.pgrx.yml` — standalone service `forge-pgrx-postgres` that builds forge's OWN `../flint-forge/images/postgres18/Dockerfile` (referenced, not forked) from the forge context. Validated (`compose config`). Kept separate from `compose.real.yml` (best-effort, off the green path).
- [x] 1.2 The image's CMD already sets `shared_preload_libraries=pg_net,pg_cron,ext_flint_llm` + `wal_level=logical` + `cron.database_name` — the compose does not override it.

## 2. Keep it best-effort / opt-in

- [x] 2.1 `run-real.sh` default path is unchanged (CI image / c002). pgrx is a SEPARATE invocation: `docker compose -f smoke/compose.pgrx.yml build|up`.
- [x] 2.2 Build cost + resource ceiling documented in `compose.pgrx.yml`'s header (the heaviest build in the phase — `cargo pgrx init --pg18` compiles Postgres; build ALONE).

## 3. Verification (best-effort)

- [x] 3.1 Attempted the pgrx build (time-boxed 25 min, standalone). **RESULT: blocked on a forge bug — NOT an OOM.** The build fails early (before the heavy pgrx compile) on `COPY images/postgres18/init-baseline/99-assert.sql ... : not found`. Root cause: forge's `.dockerignore` excludes `images/`, but `images/postgres18/Dockerfile` COPYs from `images/postgres18/{extensions,init,init-baseline}/` — a self-contradiction; those files are never in the build context. The image is unbuildable from the forge repo as configured. (The CI image builds because it has zero `COPY images/` lines.)
- [x] 3.2 **Documented the ceiling (Base Rule 40 — did not chase it):** the blocker is forge's — appended to **Know-Me-Tools/flint-forge#7** (`.dockerignore` excludes what the pgrx Dockerfile needs; suggested negations/narrowing). Did NOT fork forge's Dockerfile or edit forge's `.dockerignore` (in-flight forge file, another session editing it). The CI image (c002) remains the converging path; no agent code depends on the pgrx image. c006 is DONE as best-effort: the smoke-side compose is authored + validated; the build is blocked upstream and reported.
