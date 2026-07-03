# Analysis — integration-proof-and-debt-closure

**Phase:** integration-proof-and-debt-closure
**Date:** 2026-07-03
**Mode:** stack-specified (in-house Rust workspace; no new stack to discover)
**Research verdict:** **build, in-house** for every gap — no external library adoption
warranted. External-research tiers were **not run** (see "Research scope" below).

---

## 1. Research scope & why the tiers were skipped

Every gap in `assessment.md` is either (a) a small internal correction to existing
agent code, or (b) a persistence choice satisfiable with a port + in-memory impl the
workspace already demonstrates (`fpa-app::TaskStore`). None calls for a new crate,
framework, or skeleton:

- G1 (forge REST path) — a URL-construction fix inside `fpa-forge`; the contract is
  already source-verified from forge (`POST /{schema}/{table}`, no `/rest` prefix).
- G2 (Project persistence) — **operator-decided** (below); satisfied by a new port +
  in-memory adapter following the existing `TaskStore` pattern. No library.
- G3–G6 (write-guard, agui auth, JWKS single-flight, audit flag) — edits to existing
  agent code using deps already in the tree (`tokio::sync::Mutex` for G5).
- G7 (integration harness) — Rust's built-in `tests/` + `wiremock` (already a
  dev-dep) + `tower`/`axum` test client. All present.

Per the research-pipeline budget rules, running Tier 1–4 to "confirm" we need
nothing new would be wasted queries. **Tiers 1–4: not run (justified skip).**
Confidence: high — the relevant contracts were read directly from sibling source
this phase and last.

---

## 2. Operator decision — Project persistence (G2)

**Question put to the operator** (AskUserQuestion, this session): where does the
nested `Project` aggregate persist, given forge has no `projects` table and its REST
insert is flat-row `POST /{schema}/{table}`?

**Decision: Agent-owned `ProjectStore` port** (provenance: **user**).

Rationale accepted:
- `Project` is a rich nested aggregate (`fpa-domain::project::Project` — applications,
  sub-agents, schemas, realtime, entity-meta); a flat forge row is a poor fit.
- The agent already owns `TaskStore` (in-memory now, durable later) — precedent for
  an agent-owned store port; consistent architecture (Base Rule 9).
- **Unblocks the integration proof with zero cross-repo dependency** — no forge
  migration gate. `project.create` proves against the agent's own store.
- The G1 forge REST-path fix **still lands** (correctness), proven against forge's
  real reads / an existing table — just decoupled from `project.create`.

Design shape (for spec):
```
fpa-ports::ProjectStore  (trait, Send + Sync)
    put(&Project) -> Result<(), PortError>
    get(&ProjectId) -> Result<Option<Project>, PortError>
fpa-app  (or fpa-infra): InMemoryProjectStore  — RwLock<HashMap<ProjectId, Project>>
task_runner: "project.create" -> ProjectStore.put(project_from_input)
```
Durable (Postgres/forge-backed) impl is explicitly a **later phase** (mirrors the
`TaskStore` "durable later" note). Keep the port so the swap is a composition-root
change only.

---

## 3. Build-vs-adopt calls (all BUILD)

| Gap | Decision | Basis |
|---|---|---|
| G1 forge REST path | build (fix) | Contract source-verified; ~1-line prefix/arity change + tests |
| G2 Project store | **build** (port + in-mem) | Operator decision; `TaskStore` precedent; no library fits a nested artifact better |
| G3 gate write-guard | build (fix) | Branch on kind before port call |
| G4 agui auth | build (fix) | Add `OperatorContext` extractor (mirrors A2A H1 fix) |
| G5 JWKS single-flight | build (fix) | `tokio::sync::Mutex` refresh guard — dep already present |
| G6 audit flag | build (fix) | Thread `signature_verified` through `AuthContext` |
| G7 integration harness | build | Rust `tests/` + wiremock + axum test client — all present |

**No `adopt` / `adapt` candidates.** `library-candidates.json` records the gaps as
`build_required` with empty candidate lists.

---

## 4. Integration-proof approach (resolves assess open-Q2)

**Mock each plane at the HTTP boundary for the first green proof.** forge has no
ready compose (custom pgrx Postgres-18 image); fabric/gate do, but a first proof
should not gate on standing up three real services. Use the established in-repo
pattern: `wiremock` for forge/gate/fabric HTTP, drive the agent's real Axum router
via a `tower`/axum test client, and assert the full
`authenticate → project.create (real ProjectStore) → list_routes → fabric.health`
flow across AG-UI / A2A / MCP. `project.create` hits the **real** in-memory
`ProjectStore` (not mocked) so the aggregate round-trips for real. A **live smoke**
against real siblings is a documented follow-on, not this phase.

## 5. IdP / test-key (resolves assess open-Q3)

For the JWKS verify path, **mint an ephemeral RSA keypair in-test** and serve its
public JWK via wiremock as the "IdP". This proves H4 single-flight + signature
verification deterministically without depending on a live Kratos/Hydra or a shared
secret. Which real IdP backs `FPA_JWKS_URL` in production stays a deployment-config
concern (already env-driven), out of scope for the proof.

---

## 6. Open questions remaining

None blocking. All three assess open-questions are resolved:
- Project persistence → agent-owned store (operator).
- Integration approach → mock-at-boundary first, live smoke later.
- IdP/test-key → ephemeral in-test RSA keypair + wiremock JWKS.

Residual (non-blocking, for a later phase): durable `ProjectStore` backend, and a
real forge-live smoke once forge's compose story exists.

---

## 7. Handoff to spec/plan

All gaps are BUILD/in-house; no external adoption. Operator chose an **agent-owned
`ProjectStore` port** for the nested Project aggregate (in-memory now, durable
later) — this unblocks G7 with no cross-repo dependency while the G1 forge REST-path
fix still lands for correctness. Integration proof = mock-at-boundary + real
in-memory ProjectStore + in-test RSA JWKS. Spec should order: the small debt fixes
(G3–G6) + G1 + the ProjectStore (G2) as implementation changes, then G7 as the
single integration-test change that exercises the whole flow (the first justified
`cargo test` of this phase).
