# Reflection — live-smoke-and-durable-proof

**Phase:** live-smoke-and-durable-proof
**Date:** 2026-07-04
**Changes:** 3/3 executed (p9-c001..c003), all CI-green and pushed (`dedb924`).

> Sycophancy gate applied. This is the **landmark phase**: for the first time the real
> agent runs as a deployed service against real infrastructure. The honest framing is
> that the *code* was ready — the effort went into the *infrastructure* (Docker) and
> two container-build bugs that only a real build could surface. Scope stayed honest:
> dependencies are HTTP stubs, not real siblings (a future phase).

---

## 1. Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| Durable-store proof vs real Postgres | **MET** | c001: `fpa-store-pg` durability test PASSED against a real Postgres (testcontainers) — put→get→list→restart. Captured as `smoke/run-durable-proof.sh`. |
| Agent container + `:8088` | **MET** | c002: `smoke/Dockerfile` builds `fpa-gateway`; `compose.smoke.yml` runs it + postgres + wiremock stub. Agent logs `project store: durable Postgres` + `listening 0.0.0.0:8088`; healthz 200. |
| Automated smoke driving the live agent | **MET** | c003: **6/6 Playwright tests PASSED** against the live container (healthz, auth gate, project CRUD via the real store, fabric.health, MCP, agui). `smoke/run.sh` = one command (up→smoke→down). |

**Overall: 100% of goals MET — and proven by real runs, not mocks.** The two biggest
carried debts (durable-store runtime proof + live smoke, open since phase 6) are
**discharged**. Critically: **no wire drift** — the p5 mock-boundary integration proof
accurately predicted the live behavior (the mocks were faithful).

---

## 2. Delivered changes

| Change | Delivered | Commits |
|---|---|---|
| p9-c001 durable-proof-runner | `smoke/run-durable-proof.sh` (DOCKER_HOST wiring) | `560da4f` |
| p9-c002 agent-container-compose | `Dockerfile`, `compose.smoke.yml`, stub, `.dockerignore` (+ 2 build fixes) | `560da4f`, `19c9da6` |
| p9-c003 playwright-smoke | `smoke.spec.ts`, `run.sh`, config, package | `dedb924` |
| (infra) | `scripts/reset-colima.sh` recovery tool | earlier commits |

All on `origin/main`.

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | 0/3 (subsystem absent — all phases) |
| Dedicated code review | n/a — **no Rust source changes**; `smoke/` artifacts only |
| Verification | **3 real runs, all green**: durable proof, `compose up`+healthz, 6/6 live smoke |
| Container-build bugs found + fixed | **2** (both real, both only surfaceable by a real build) |

**The two build bugs are the phase's most valuable findings — a real container build
caught what in-process tests never could:**
1. **`.cargo/config.toml` leaked into the build context** — the machine-local (gitignored)
   config sets `rustc-wrapper = sccache` + a host `ld64.lld` path; inside the image
   neither exists → `No such file or directory`, exit 101. Fixed: excluded `.cargo/`
   in `.dockerignore` + neutralized `RUSTC_WRAPPER`/`RUSTFLAGS` in the Dockerfile.
2. **transitive `openssl-sys`** — despite reqwest using rustls, something in the tree
   links native OpenSSL; the slim builder lacked `libssl-dev` → build failed. Fixed:
   `libssl-dev` in the builder, `libssl3` in the runtime.

Neither is reproducible without actually building the container — which is exactly why
this phase (not just more unit tests) mattered.

---

## 4. Technical debt

**New (small, honest):**
1. **Dependencies are HTTP stubs, not real siblings.** The smoke proves the **real
   agent binary + wiring + `:8088`** end-to-end, but forge/gate/fabric are wiremock
   (3 static GETs), not the real services. Real-sibling wire-compat is a **separate
   future phase** (operator decision) — the forge pgrx build risk is deliberately not
   in scope here.
2. **`--locked` build + no cargo-chef layer caching** — the container rebuilds the
   whole workspace each time (slow). A `cargo-chef` dependency-caching layer would
   speed reruns; deferred (the smoke is run occasionally, not in a tight loop).
3. **Smoke Postgres durability is proven by the `#[ignore]`d testcontainers test**, not
   by an agent-container restart against the compose Postgres. The agent *does* use the
   real compose Postgres live (logs confirm), but a "kill the agent container, restart,
   data survives" assertion at the compose level is a nice-to-have follow-on.

**Carried (still deferred, unchanged):** real 3-plane smoke; Postgres TLS (rustls
connector); per-operator RLS; MCP multi-server; fabric WS subscriptions; OpenDesign;
A2UI/React UI; Tauri; knowledge-base.

**Infra note (not code debt):** the vz VM's containerd store corrupted mid-build once
(`input/output error` on every write); a fresh `scripts/reset-colima.sh` VM cleared it
and built fine. vz under heavy build write-load is a known risk; `reset-colima.sh` is
the proven recovery, and `COLIMA_VMTYPE=qemu` is the fallback if it recurs.

---

## 5. Lessons captured

- **A real container build surfaces bugs no in-process test can.** Both build failures
  (`.cargo` sccache leak, `openssl-sys`) were invisible to `cargo test` on the host —
  they only exist in a clean, self-contained image build. "It compiles + tests pass on
  my machine" ≠ "it builds in a container." Worth doing early for anything that ships
  as an image.
- **Exclude host-local build config from the Docker context — always.** A gitignored
  `.cargo/config.toml` with host paths is a classic silent container-build breaker.
  `.dockerignore` it AND neutralize the wrapper/linker env in the Dockerfile (belt +
  suspenders).
- **`-sys` crates need their system lib even when you think you're pure-Rust.** reqwest
  was rustls, so I assumed no OpenSSL — wrong; a transitive dep pulled `openssl-sys`.
  When a slim image build fails, check for `-sys` crates before assuming a code bug.
- **Mock-boundary proofs earn their keep when the live smoke agrees with them.** The p5
  in-process integration test predicted the live behavior exactly — 6/6, no drift. That
  validates the whole "prove the shape with mocks, then confirm live" progression.
- **`node_modules` needs a `.gitignore`, not just a `.dockerignore`.** `git add -A`
  swept 206 install files in; caught + amended pre-merge. Different ignore files serve
  different tools — cover both.
- **When infra fights you, invest in the recovery tool, not repeated manual pokes.**
  `scripts/reset-colima.sh` turned "the daemon is wedged again" from a 20-minute manual
  slog into one command — and it paid off three times this phase.

---

## 6. Recommended Next Phase

The agent is now **proven real** — durable persistence + a live containerized service.
The two most valuable directions:

**Option A — `real-sibling-smoke` (infra-gated, higher fidelity):** replace the wiremock
stubs with the real forge/gate/fabric containers and re-run the smoke, proving actual
wire compatibility (not just faithful mocks). Needs the forge pgrx PG-18 build to
converge — the one risk. Discharges debt #1.

**Option B — a no-infra product increment** (e.g. `project.delete` to round out CRUD;
richer `project.create` input validation surfaced live; or a `cargo-chef` build-cache +
CI wiring so the smoke runs in CI).

**Recommendation: operator's call, and it hinges on the forge build.** If the pgrx
build is feasible on this machine (12 GiB VM, now that Docker is stable), **Option A**
is the natural capstone — the last "prove it real" step. If the forge build is too
heavy/risky, **Option B** (I'd suggest **`smoke-in-ci-and-crud-completion`**: wire the
smoke + durable proof into CI with a build cache, and add `project.delete`) keeps
momentum with zero new infra risk. I will ask.

---

## 7. Reflect handoff

All 3 goals MET and **proven by real runs** — durable store vs real Postgres,
containerized agent live on `:8088`, 6/6 Playwright smoke, no wire drift. The two
biggest carried debts (durable-proof + live-smoke) are discharged. Two real
container-build bugs (`.cargo`/sccache leak, transitive `openssl-sys`) were found and
fixed — findings only a real build surfaces. Honest scope: deps are HTTP stubs, not
real siblings. Corrective action & recommended next phase: **operator A/B** —
`real-sibling-smoke` (Option A, if the forge pgrx build converges — the fidelity
capstone) vs `smoke-in-ci-and-crud-completion` (Option B, no-infra: CI-wire the smoke +
`project.delete`). Infra is stable (fresh vz VM); `scripts/reset-colima.sh` recovers it.
