# Goals — real-sibling-smoke

> Seeded from `live-smoke-and-durable-proof/reflection.md` → "Recommended Next Phase"
> (Option A, operator-chosen). Phase 9 proved the real agent end-to-end against **stub**
> dependencies (6/6 live smoke). This phase's capstone: swap the wiremock stub for the
> **real forge / gate / fabric** and prove **actual wire compatibility** — the last
> unknown. Docker is now stable (fresh vz VM, 12 GiB).
>
> **Risk-ordered (land fidelity incrementally):** gate + fabric ship Dockerfiles +
> compose; **forge is the hard one** — only its Postgres-18 pgrx image has a Dockerfile;
> the Quarry gateway (`fdb-gateway`) has **no** Dockerfile and the pgrx build is heavy
> (may not converge). So prove the reachable planes first; forge is the stretch.

## Primary goals (risk-ordered)

1. **Real gate + fabric in the smoke (reachable now).** Add the real `flint-gate`
   (`../flint-gate/Dockerfile` + its `docker-compose.yml` services: gate + postgres +
   kratos) and real `flint-realtime-fabric` (`../flint-realtime-fabric/compose.yml`)
   to a `compose.real.yml`; point the agent's `FPA_GATE_ADMIN_URL` + `FPA_FABRIC_ENDPOINT`
   at them; re-run the smoke's gate/fabric hops against the **real** services. Fix any
   wire drift (paths, ports, auth handshake) the stub hid.

2. **Real forge (STRETCH — the pgrx build).** Build forge's Postgres-18 pgrx image
   (`../flint-forge/images/postgres18/Dockerfile`) AND author a Dockerfile for the
   Quarry gateway (`fdb-gateway` — none exists); stand both up; point `FPA_FORGE_URL`
   at the gateway; exercise the forge-read hops. **If the pgrx build does not converge
   on this machine, document it honestly and keep forge stubbed** — do not let it block
   the phase.

3. **A real-sibling smoke variant + runner.** `smoke/compose.real.yml` + a
   `smoke/run-real.sh` (mirrors `run.sh` but against real siblings). The stub smoke
   (`run.sh`) stays as the fast/reliable path; this is the high-fidelity path.

## Success criteria

- The live smoke passes against **real gate + fabric** (at minimum); any wire drift
  found is fixed and recorded.
- forge: either the real forge is in the smoke (full fidelity), OR its non-convergence
  is documented with the exact blocker and forge stays stubbed (partial fidelity, honestly
  scoped).
- `smoke/run-real.sh` is one command, cleans up; the stub `run.sh` still works.

## Open questions (for /kbd-assess → operator)

- **How far to push forge?** Time-box the pgrx build (e.g. one build attempt); if it
  exceeds the box or OOMs, stub it. (Operator may want a hard cap.)
- **Sibling repos as build contexts:** the siblings live at `../flint-gate` etc. — the
  compose `build.context` points outside this repo. Confirm that's acceptable (it is for
  a local smoke; not for a portable CI image).
- **Auth against real gate:** phase 9 used HS256 (agent-verified). Real gate may expect
  a real Ory/Kratos-minted token — does the smoke keep HS256 (agent trusts its own key)
  or go through gate's real auth? (Lean: keep HS256 for the agent's own verify; gate is
  a *downstream* the agent calls, not the agent's authenticator, per the cross-plane
  contract — gate authenticates inbound, the agent calls gate's admin API.)

## Explicitly out of scope (still deferred)

Postgres TLS; per-operator RLS; MCP multi-server; fabric WS subscriptions; OpenDesign;
A2UI/React UI; Tauri; knowledge-base; CI wiring of the smoke (the no-infra Option B —
a separate future phase).
