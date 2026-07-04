## 1. Unified real compose

- [ ] 1.1 Author `smoke/compose.real.yml` composing: **agent** (build `smoke/Dockerfile`, `:8088`) + **gate** stack (c001) + **forge** stack (c002: PG + migrate one-shot + `fdb-gateway`) + **fabric** stack (c003). Namespace services `gate-*` / `forge-*` / `fabric-*`; one project network.
- [ ] 1.2 Wire the agent to the REAL planes: `FPA_GATE_ADMIN_URL` → `http://gate:4457`, `FPA_FORGE_URL` → `http://forge-gateway:<port>`, `FPA_FABRIC_ENDPOINT` → `http://fabric-gateway:8080`, `FPA_PROJECT_DB_URL` → agent's own Postgres, `FPA_GATE_JWT_KEY` (throwaway test secret — not a real credential).
- [ ] 1.3 `depends_on: { condition: service_healthy }` ordering across the whole graph; generous `start_period`s (fabric heaviest). Agent starts last.

## 2. One-command runner

- [ ] 2.1 `smoke/run-real.sh`: export `DOCKER_HOST` (colima socket), `up --build`, wait for every plane's health, run the real smoke, `down -v` on a trap (success AND failure).
- [ ] 2.2 Keep the p9 stub path (`run.sh` + `compose.smoke.yml`) intact as the fast path.
- [ ] 2.3 If a single `up` OOMs on the 12 GiB VM, bring up in waves (gate+forge, then fabric, then agent) and record it.

## 3. Real smoke incl. realtime event

- [ ] 3.1 `smoke/smoke.real.spec.ts`: the p9 HTTP hops (auth, project CRUD, `fabric.health`, gate/forge reads) against the REAL stack; fix any wire drift found.
- [ ] 3.2 Realtime test: agent subscribes (c004 client) → drive a change (forge DB write that CDC picks up, or fabric `dev` fan-out trigger — decide at execute) → assert the agent receives the corresponding `EventEnvelope` change event within a timeout.

## 4. Verification (integration milestone — counts toward the ≤3 test budget)

- [ ] 4.1 `smoke/run-real.sh` green end-to-end: all planes healthy, HTTP hops pass, realtime event received. This IS the phase's proof.
- [ ] 4.2 No leftover containers/volumes after teardown. Record VM resource headroom.
