## 1. Unified real compose

- [x] 1.1 `smoke/compose.real.yml` composes: **agent** (`smoke/Dockerfile`, `:8088`) + **gate** (c001) + **forge** PG+bootstrap (c002) + **fabric** (c003). Services `gate-*`/`forge-*`/`fabric-*` namespaced; one network. Validated (`compose config`). forge-gateway is in the `forge-full` profile (off by default — blocked on flint-forge#7).
- [x] 1.2 Agent wired to REAL planes: `FPA_GATE_ADMIN_URL` → `http://flint-gate:4457`, `FPA_FABRIC_ENDPOINT` → `http://fabric-gateway:8080`, `FPA_FORGE_URL` → `http://forge-gateway:8080`, `FPA_PROJECT_DB_URL` → `agent-postgres`, `FPA_GATE_JWT_KEY` (throwaway HS256 test secret).
- [x] 1.3 `depends_on: service_healthy` ordering; generous `start_period`s (fabric 40s). Agent starts last (deps healthy).

## 2. One-command runner

- [x] 2.1 `smoke/run-real.sh`: exports `DOCKER_HOST`, builds **serially** (see 2.3), `up --wait`, polls agent `/healthz`, runs the smoke, `down -v` on a trap.
- [x] 2.2 p9 stub path (`run.sh` + `compose.smoke.yml`) untouched.
- [x] 2.3 **Wave/serial bring-up implemented + REQUIRED:** `up --build` compiles all planes concurrently → OOM-kills the 12 GiB VM (2+ large Rust builds at once). run-real.sh builds one service at a time. **VM finding:** even serial, the unified 4-plane build is at the VM's edge (12 GiB RAM / 120 GB disk); repeated runs filled the build cache (57 GB) and crashed dockerd. Recovered via `colima restart` + `builder prune`. The live unified run is **capacity-bound** on this host.

## 3. Real smoke incl. realtime event

- [x] 3.1 `smoke/smoke.real.spec.ts`: HTTP hops (agent health, auth-reject, `fabric.health` → real fabric, project CRUD → real PG) against the REAL stack. (Gate A2A hop dropped — no read-only gate catalog kind exists; gate proven standalone in c001.)
- [x] 3.2 Realtime test authored: agent subscribes via the NEW **`/fabric/subscribe` SSE bridge** (agent code — the inbound bridge c004 deferred) → smoke `POST /v1/publish` to fabric on the SAME channel → assert the `EventEnvelope` frame. Trigger = fabric `/v1/publish` (deterministic channel, what fabric's own subscribe_mux.rs uses) NOT dev-inject (random channels).

## 4. Verification (integration milestone)

- [x] 4.1 **CODE proven-compilable + planes proven individually:** agent SSE bridge (check/clippy -D warnings/fmt green); `frf-domain` vendored so the containerized agent builds (was a cross-repo path dep — confirmed `Compiling frf-domain` in-context); c001 gate, c002 forge PG, c003 fabric each green in isolation.
- [ ] 4.2 **DEFERRED — unified live green + realtime SSE assertion:** the all-planes `run-real.sh` is VM-capacity-bound (12 GiB can't sustain 4 heavy sibling builds; dockerd crashed 3×). Per operator: fall back to proving realtime at the **port layer** (a `#[ignore]` Rust integration test: fabric-only compose + `FabricClient::subscribe` + `POST /v1/publish` + assert) — far lighter than 10 containers. Tracked as the c005 completion step.
