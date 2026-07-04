# Plan — live-smoke-and-durable-proof

**Phase:** live-smoke-and-durable-proof
**Backend:** OpenSpec (`openspec/changes/p9-c00N-*`)
**Changes:** 3 (all validate `--strict`)
**Nature:** infra/ops (compose, Dockerfile, stub, Playwright) — not pure Rust. The
"batch the compiles" rule doesn't apply the same way; each change's verification IS
running a real thing (the durable proof, `compose up`, the smoke).

---

## Ordered change list

### 1. p9-c001-durable-proof-runner  (G1) — **first (quick win, independent)**
`smoke/run-durable-proof.sh` — the `DOCKER_HOST`-from-colima runner over the already-
passing `fpa-store-pg --ignored` test.
- **Why first:** independent, and it locks in the biggest carried debt's proof as a
  one-command artifact. Fast.
- **Depends on:** nothing (test already exists + passes).
- **Verify:** run it → PASS (already validated live).
- **Agent:** none — shell + a real test run.

### 2. p9-c002-agent-container-compose  (G2 + G3) — **second (the substance)**
`smoke/Dockerfile` + `compose.smoke.yml` + wiremock stub → agent boots on `:8088`.
- **Why second:** the smoke (c003) needs this stack to exist. The build is the main
  risk (reqwest/rustls build deps in the slim image) — surface it here, before the
  smoke.
- **Depends on:** nothing hard; independent of c001.
- **Verify:** `compose config` valid → `up --build` → `curl :8088/healthz` = 200 →
  `down -v`. **First heavy build** (rust:1.93 image compile of fpa-gateway).
- **Agent:** none — but if the container build fails on missing build deps, use
  `rust-build-resolver` / iterate the Dockerfile.

### 3. p9-c003-playwright-smoke  (G4) — **third (the live proof)**
`smoke/smoke.spec.ts` + `run.sh` → drive `:8088` end-to-end.
- **Why last:** hard-depends on c002's running stack.
- **Depends on:** **c002 (hard).**
- **Verify:** `./smoke/run.sh` → Playwright smoke **passes against the live agent**;
  fix any wire drift the mock-boundary hid; clean teardown. **The phase's headline
  live milestone.**
- **Agent:** none — Node/Playwright + a real run.

---

## Dependency graph

```
c001 (durable-proof runner)          [independent, quick]
c002 (Dockerfile + compose + stub)   [the substance]
  └─► c003 (Playwright smoke)         [hard: needs the running stack]
```

## Execution contract

- **c001 first**: write the runner, run it → capture the real-Postgres PASS. (Debt #2
  discharged + documented.)
- **c002 next**: write Dockerfile + compose + stub; `compose up --build`; the rust:1.93
  image build is the one slow step — budget for it; iterate the Dockerfile if the
  build needs `pkg-config`/`libssl-dev`/`ca-certificates`. Gate: `:8088/healthz` = 200.
- **c003 last**: write the smoke + runner; `./smoke/run.sh` end-to-end. **This is the
  phase's one heavy end-to-end run** — expect to fix real wire drift the mocks hid
  (that's the *point* of a live smoke). Reserve reruns for that.
- **`cargo test` budget:** the durable proof (c001) is one `--ignored` run; the Rust
  workspace is otherwise untouched (no lib changes). The heavy runs are `compose up`
  (c002) and `smoke/run.sh` (c003), not `cargo test`.

## Compliance / cautions

- **No Rust source changes** — this phase adds `smoke/` artifacts only; the agent
  binary is built as-is. If the smoke reveals an agent bug (wire drift), that's a
  finding → a small follow-up change, not silently patched into this phase.
- **Secrets:** the smoke's HS256 `FPA_GATE_JWT_KEY` is a throwaway test secret in
  `compose.smoke.yml` — fine for a local smoke; note it's not a real credential.
- **Stock images only**; `.dockerignore` keeps the build context small (exclude
  `target/`, `.git/`, `.kbd-orchestrator/`, `node_modules/`).
- **Clean teardown** (`down -v`) on success AND failure (trap in run.sh).

## Handoff to execute

3 validated changes. c001 (durable-proof runner — locks the already-passing real-PG
proof) first; c002 (Dockerfile + compose.smoke.yml + wiremock stub → agent on :8088)
the substance; c003 (Playwright smoke driving the live agent) last, hard-depending on
c002. First change to apply: **p9-c001-durable-proof-runner**. Each change's
verification is a real run (durable proof / compose up / smoke), not a batched compile.
