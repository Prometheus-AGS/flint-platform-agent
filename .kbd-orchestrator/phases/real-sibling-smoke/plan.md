# Plan — real-sibling-smoke

**Phase:** real-sibling-smoke
**Stage:** plan
**Date:** 2026-07-04
**Backend:** OpenSpec (`openspec/changes/p10-c00*`), 6 changes, all `openspec validate --strict` clean.

## Operator directive reconciliation (IMPORTANT)

The analyze artifacts (`library-candidates.json` G2) recommended **stubbing fabric**.
The operator **overrode** that at `/kbd-spec`: *"we want nothing but REAL stuff"* +
(AskUserQuestion) *"Add the agent subscribe client THIS phase too."* The **spec is
authoritative**: fabric is real (c003) and the agent gets a real WS subscribe client
(c004). This plan orders the six REAL changes accordingly — the G2 "stub" line in the
candidate file is superseded, recorded here so the delta is explicit (Base Rule 10).

## Change ordering (dependency-driven)

Order is chosen so the only new **Rust** lands + compiles early (protects the ≤3
`cargo test` budget), the highest-risk *authored* infra (forge gateway Dockerfile) is
proven before integration, and the heavy all-planes compose is last of the converging
set. c006 is strictly last (best-effort, opt-in, touches nothing on the green path).

| Order | Change | Why here | Depends on | Risk | Agent |
|---|---|---|---|---|---|
| 1 | **p10-c004-fabric-realtime-client** | Only new Rust; must compile before c005's realtime assertion. Independent of compose infra → build first, `cargo check` once. | frf-agentproto (path dep) | Med (new port method + WS dep) | rust-build-resolver / rust-reviewer |
| 2 | **p10-c002-real-forge** | Highest *authoring* risk: `fdb-gateway` has NO Dockerfile (must author) + migrate step. Prove the forge plane builds+serves before wiring it into the unified compose. | Docker (have it); forge CI PG image | Med-high | devops-engineer |
| 3 | **p10-c001-real-gate** | Reuse `../flint-gate` Dockerfile+compose — lower authoring risk. Prove gate `:4457` boots standalone. | Docker | Small-med | devops-engineer |
| 4 | **p10-c003-real-fabric** | Heaviest to boot (gateway + iggy + keto + surreal + PG, CDC repl). Prove the fabric spine + `/healthz` before the realtime smoke rides on it. | Docker; c004 client (to later subscribe) | Med-high (6 svcs, CDC) | devops-engineer |
| 5 | **p10-c005-compose-real-and-smoke** | Integrates ALL of the above into `compose.real.yml` + `run-real.sh` + the smoke incl. the write→CDC→agent realtime event. The phase's proof. **Integration milestone — 1 of the ≤3 `cargo test`/full-run waits.** | c001, c002, c003, c004 | High (all planes on one 12 GiB VM) | e2e-runner |
| 6 | **p10-c006-full-pgrx-image** | Best-effort, opt-in, time-boxed. Converges → proves forge's heavy pgrx stack builds. Stalls → documented; does NOT touch the green c005 path. | forge full pgrx Dockerfile | Heavy (rust:1.96 + cargo pgrx init --pg18) | devops-engineer |

## Build-order notes / risks

- **Parallelizable:** c004 (Rust) is independent of c001–c003 (containers). If iterating,
  c004 can be built while container images build. But *land* in the table order so c005
  has every dependency present.
- **Resource ceiling (the real risk):** all planes at once on 12 GiB may OOM. c005 tasks
  already say: if a single `up` OOMs, bring up in **waves** (gate+forge → fabric →
  agent) and record it. Do NOT chase a single-`up` if waves work (Base Rule 40).
- **Sibling-repo edits:** c002 authors `smoke/fdb-gateway.Dockerfile` **in THIS repo**
  (not `../flint-forge`) — references forge's crate via build context, does not fork
  forge (Base Rule 3). c006 references forge's own `images/postgres18/Dockerfile` in
  place (opt-in override), also no fork.
- **Test budget:** the whole phase's real integration proof is c005's `run-real.sh` —
  that is the milestone worth a full wait. c004's unit (serde round-trip of
  `ContentBlock` + Unknown + error mapping) is a cheap local test, not a full-suite wait.
- **frf-agentproto path dep** is cross-repo (not portable) — acceptable for a local
  smoke; flagged as an Open Question in c004 for a later published-crate follow-up.

## Success criteria (phase-level)

1. `smoke/run-real.sh` green end-to-end: real gate + real forge (CI image + fdb-gateway
   + migrations) + real fabric all healthy; agent boots wired to all three on `:8088`.
2. HTTP smoke passes against the real planes (auth, project CRUD, `fabric.health`,
   gate/forge reads) — any wire drift fixed.
3. **Realtime proof:** agent subscribes (c004) and receives a `ContentBlock` change
   event when an upstream change is driven (forge write → CDC, or fabric `dev` trigger).
4. Teardown leaves no containers/volumes; stub `run.sh` (p9) still works.
5. c006 best-effort: pgrx image builds+loads OR its ceiling is documented — either way
   the green path is unaffected.

## First change to apply

**p10-c004-fabric-realtime-client** — `/kbd-execute real-sibling-smoke` then
`/kbd-apply` walks its tasks (add frf-agentproto path dep + tokio-tungstenite, new
`FabricClient::subscribe` port method, WS adapter in fpa-fabric, one batched `cargo
check`, cheap serde unit test).
