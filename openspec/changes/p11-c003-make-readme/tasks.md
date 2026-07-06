## 1. Makefile

- [ ] 1.1 Top-level `Makefile` with `.PHONY` targets: `smoke` → `smoke/run.sh`; `smoke-real` → `smoke/run-real.sh`; `smoke-real-nobuild` → `smoke/run-real.sh --no-build`; `smoke-real-forge` → `smoke/run-real.sh --forge-full`. A `help` target listing them. No new behavior — thin wrappers.

## 2. README

- [ ] 2.1 Update `smoke/README`: document `run-real.sh` (up→wait→smoke→`down -v`), the `--no-build` workflow (build images once per-service, then boot — the VM runs the stack fine; concurrent builds OOM), the default (agent+gate+fabric) vs `forge-full` profile (gated on flint-forge#7), and the `make` targets. Add a pointer to the CI stub job (c001) + the opt-in real-smoke workflow (c002).

## 3. Verification

- [ ] 3.1 `make help` lists the targets; `make -n smoke-real-nobuild` (dry-run) shows the right command. No need to run the heavy smoke here (c001/local runs cover it).
- [ ] 3.2 README renders correctly + links are valid.
