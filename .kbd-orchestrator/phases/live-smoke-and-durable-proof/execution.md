# Execution — live-smoke-and-durable-proof

**Backend:** OpenSpec (KBD source of truth). **Dispatch:** in-session.
**Nature:** infra/ops — verification = real runs (durable proof / compose up / smoke),
not batched compiles.

## Order & contract

1. p9-c001 durable-proof-runner — `smoke/run-durable-proof.sh`; **run it → real-PG PASS**.
2. p9-c002 agent-container-compose — `smoke/Dockerfile` + `compose.smoke.yml` + wiremock
   stub; **`compose up --build` → `:8088/healthz` = 200** (the one slow rust image build).
3. p9-c003 playwright-smoke — `smoke/smoke.spec.ts` + `run.sh`; **`./smoke/run.sh` →
   live end-to-end smoke PASSES** (expect + fix real wire drift). Hard-depends on c002.

## Heavy-run budget

Not `cargo test`-bound. The heavy runs: 1× `--ignored` durable proof (c001, ~8s), 1×
container build + `compose up` (c002, slow — rust:1.93 image), 1× `smoke/run.sh` (c003).
Reruns reserved for wire-drift fixes the smoke surfaces.

## QA gate

artifact-refiner absent. No Rust source changes → no rust-reviewer needed (smoke/
artifacts only). A smoke-revealed agent bug → a separate follow-up change, not a
silent patch.
