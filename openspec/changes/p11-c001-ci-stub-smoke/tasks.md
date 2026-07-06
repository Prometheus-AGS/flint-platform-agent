## 1. CI job

- [ ] 1.1 Add a `smoke` job to `.github/workflows/ci.yml` (`runs-on: ubuntu-latest`): checkout; set up Node (for `@playwright/test`); ensure Docker/Compose (present on the runner); run the stub smoke.
- [ ] 1.2 Run via `smoke/run.sh` (reuses up→wait→smoke→`down -v`) OR inline the equivalent (`docker compose -f smoke/compose.smoke.yml up --build`, poll `/healthz`, `npm --prefix smoke install`, `npx playwright test smoke.spec.ts`, `down -v` in an `if: always()` step). Confirm `run.sh` does NOT install browsers (HTTP-only smoke).
- [ ] 1.3 `timeout-minutes` set; no secrets; no sibling checkout.

## 2. Verification

- [ ] 2.1 Validate the workflow YAML (actionlint or `gh workflow view` / a syntax check). Confirm the job appears on PRs to `main`.
- [ ] 2.2 Locally dry-run the stub smoke (`smoke/run.sh`) to confirm it still passes with the current tree (frf-domain vendored) — the CI job runs the same path.
- [ ] 2.3 A deliberately-broken assertion fails the job (spot-check the exit-code propagation) — or reason about it from `run.sh`'s `set -e` + `npx playwright test` exit.
