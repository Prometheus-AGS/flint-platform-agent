# Assessment — project-and-application-management

**Phase:** project-and-application-management
**Date:** 2026-06-30
**Method:** Codebase inspection (this repo + 4 sibling repos) + firecrawl validation of external tech claims, applying sycophancy-correction (claims verified, not accepted).

---

## 0. Bottom line up front

The vision is coherent and the sibling ecosystem is real, but the spec **assumes
infrastructure that is mostly still specified rather than built**. The single
biggest risk is not in this agent — it is that **`flint-forge`'s gateway is a
stub** (`fdb-gateway/src/main.rs` only) and the **A2UI registry is an RFC
(`RFC-FORGE-A2UI-001`), not a running service.** This agent cannot "administer
the fabric" until those exist. The realistic near-term scope is: build the
agent's *protocol + orchestration shell* against **contracts**, with adapters
that degrade gracefully while forge/gate fill in.

Three framing corrections (verified) are folded into the gaps below:
1. **A2UI primitives are forge-owned, not agent-owned.** `RFC-FORGE-A2UI-001`
   names `flint-platform-agent` as a *consumer* of a global registry served from
   forge's Postgres. This contradicts the current `CLAUDE.md` claim that this
   repo "defines the canonical A2UI primitives." → Reframe: this agent *queries
   and composes* the registry; `fpa-protocol` types must conform to it.
2. **`prometheus-entity-management` is a TypeScript pnpm/Turborepo workspace**,
   not a Rust crate — it **cannot** be a Cargo path dependency. It already
   publishes `a2ui-react`, `entity-graph-{a2a,mcp,react,sdl}`. These are consumed
   by the *generated* React apps, not by this Rust binary.
3. **"OpenDesign" = `open-design.ai` / `nexu-io/open-design`** (confirmed with
   user): a local-first Claude-Design alternative with a **Skills→Plugins** model
   (`open-design.json` manifest, `plugins/`+`skills/` dirs, `od plugin
   list|install`). Requirement #6 = ship an OpenDesign *plugin* that exports to
   our fabric format — NOT consume a generic "ODSF design API."

---

## 1. Current state of THIS repo (baseline)

| Area | State |
|---|---|
| Workspace | ✅ 10-crate hexagonal `fpa-*` (edition 2024); builds, clippy-clean, CI green |
| Protocol surfaces | 🟡 Stubbed: AG-UI SSE, A2A task endpoints, MCP server (HTTP-streaming JSON-RPC). Return typed payloads; no real logic |
| Ports | 🟡 4 trait seams defined (`ForgeMetadata`, `FabricClient`, `GateAdmin`, `McpClient`) — all `todo!()` |
| Composition root | ❌ Adapters not wired into `TaskRunner`; no Axum `State` |
| A2A task catalog | ❌ None |
| MCP client | 🟡 Trait only (`fpa-mcp`), no transport |
| KB / OpenDesign / UI / Tauri / Auth | ❌ Not started |

---

## 2. Sibling-repo ground truth (what we can actually depend on today)

| Repo | Org | Relevant state | Consequence for this agent |
|---|---|---|---|
| **flint-forge** | Know-Me-Tools | `fdb-gateway` = stub `main.rs`; A2UI is `RFC-FORGE-A2UI-001` (design, not built); KB/embeddings live in `ext-flint-llm` (Ember, pgrx); `forge-domain` workspace-versioned | **Hard dependency not ready.** Agent must code to forge's *documented contracts* and tolerate their absence |
| **flint-realtime-fabric** | Prometheus-AGS | `frf-agentproto` implemented (`ContentBlock` + proto); has git remote | ✅ Reusable as **git dependency** for protocol type parity |
| **flint-gate** | Know-Me-Tools | Ships Go/TS/Flutter SDKs — **no Rust SDK**; admin server `:4457` | `fpa-gate` calls admin HTTP API directly (reqwest); no crate reuse |
| **prometheus-entity-management** | Prometheus-AGS | TS workspace; publishes `a2ui-react`, `entity-graph-{a2a,mcp,react,sdl,core,cli,htmx}` | **Consumed by generated React apps**, not by Rust. Informs A2UI/entity metadata shape |

### Cross-repo crate referencing (requirement: "reference repos from this workspace")
- **Verified constraint:** Cargo path deps require the crate to be inside *this*
  workspace tree; it is not. Options, in order of preference:
  1. **git dependencies** (e.g. `frf-agentproto = { git = "…flint-realtime-fabric", tag = "proto-v1" }`) — pins to a tag, survives across machines, no vendoring. **Recommended for `frf-agentproto`.**
  2. **Published crates** — none are on crates.io / a private registry yet.
  3. **git submodules / vendoring** — heavier; only if we need to build forge locally.
- **Do NOT** assume forge/gate expose Rust crates we can link — gate has none;
  forge's are workspace-internal and its gateway is unbuilt.

---

## 3. Gap analysis per requirement

Legend: ✅ ready · 🟡 partial/contract-only · ❌ greenfield · ⚠️ blocked by sibling

| # | Requirement | Status | Gap / risk |
|---|---|---|---|
| 1 | Agentic endpoints (admin/test/use fabric) | 🟡 | Surfaces stubbed; need adapter wiring + auth context propagation. ⚠️ real value blocked until forge gateway exists |
| 2 | Project/app mgmt abstractions, A2A tasks, tools, services | ❌ | No task catalog. Need to define the **domain model of a "project"** (A2UI components, sub-agents, app defs, WASM plugins, DB schemas, realtime params, entity metadata). This is the **core of the phase** |
| 3 | Skills for MCP tools | ❌ | Decide skill format (likely mirror the OpenDesign/Anthropic SKILL.md convention used across Prometheus) |
| 4 | MCP client behavior | 🟡 | Trait exists; need an MCP client transport (HTTP-streaming + stdio). Verify a maintained Rust MCP SDK vs hand-roll |
| 5 | Knowledge-base tied to projects | ⚠️❌ | Depends on forge Postgres + `ext-flint-llm` (Ember) embeddings. Build the KB *abstraction* now; wire to forge later |
| 6 | OpenDesign plugin → export to fabric | 🟡 | OpenDesign plugin model confirmed (`open-design.json`, skills→plugins). Need an export target = our project artifact schema (req #2). The plugin itself is authored in OpenDesign's stack, not Rust |
| 7 | React 19/Vite 7 UI via `build.rs` static embed; playground that *generates* like-architected apps | ❌ | Large. Two sub-parts: (a) **this agent's own** embedded chat UI; (b) a **project generator/template** that scaffolds new agent+UI repos. shadcn+Base UI + assistant-ui + A2UI confirmed viable. **UI source-of-components decision (user, 2026-06-30):** target = the **flint-forge React SDK** (forge change `p5-c010-react-sdk`, *not yet built*); interim = locally-defined A2UI React components or **HTMX prototyping** behind a swappable seam, optionally `@prometheus-ags/a2ui-react` v3.0.0-alpha.0. Replace with forge SDK when it ships |
| 8 | Tauri desktop+mobile, PWA fallback, runtime detect | ❌ | Confirmed viable (Tauri 2 stable, mobile shipped, runtime detection standard). Cross-cutting UI constraint, not a standalone module |
| 9 | Ory Kratos+Hydra+Keto auth; Postgres role/permission model from JWT via gate | 🟡 | Aligns with forge's documented 4-layer model (Kratos→Keto→RLS→Cedar). **Hydra (OAuth2) is the new addition** vs forge's spec — verify forge/gate already assume Hydra or if we're extending the model |

---

## 4. External tech validation (firecrawl-verified, not assumed)

| Claim | Verdict | Note |
|---|---|---|
| OpenDesign has plugin/skill model + fabric export | ✅ confirmed | `nexu-io/open-design`; `open-design.json` manifest; `od plugin list/install`; serialized plugin export |
| assistant-ui for generative UI | ✅ confirmed | YC W25, active through Jun 2026, renders tool calls/JSON as React |
| shadcn "latest" + Base UI | ✅ confirmed | shadcn supports both Radix and Base UI since Jun 2025; "shadcn + base-ui" is a real path |
| Tauri 2 desktop+mobile+PWA detect | ✅ confirmed | v2 stable, iOS/Android ship from same core; runtime detection is standard |
| Ory Hydra alongside Kratos/Keto | 🟡 plausible | Standard Ory stack; **but** forge's spec only names Kratos/Keto/Cedar — confirm Hydra is intended platform-wide or agent-specific |

---

## 5. Recommendations / things the spec missed

1. **Resequence around the forge dependency.** Treat forge gateway + A2UI
   registry as **external contracts**, not callable services. Build the agent
   shell + domain model + adapters-with-fakes now; integrate when forge lands.
   Otherwise this phase blocks on another repo's roadmap.
2. **Fix the A2UI ownership contradiction in `CLAUDE.md`** before coding
   `fpa-protocol` further — align to `RFC-FORGE-A2UI-001` (agent = consumer).
3. **Author a "Project" artifact schema first** (req #2). It is the hub every
   other requirement (#5 KB, #6 export, #7 generation) plugs into. It must be
   typed, versioned, JSON-Schema'd (Base Rule 39).
4. **Split requirement #7 into two phases** — the agent's own embedded UI is a
   modest `build.rs` job; the *project generator that emits new repos* is a
   large subsystem (templating, GitHub repo creation, deploy). Do not conflate.
5. **MCP Rust SDK decision is a gate** for #3/#4 — verify a maintained crate
   (e.g. an official `rmcp`/`modelcontextprotocol` Rust SDK) at plan time per
   Base Rule 22; do not hand-roll if a vetted one exists.
6. **Auth: confirm the Hydra addition** with the forge/gate owners; the Postgres
   permission model should be *derived from* gate-issued JWT claims (the agent
   must not mint its own authority — Base Rule 33, and forge §2.2/§2.3).
7. **Two GitHub orgs in play** (`Know-Me-Tools` for forge/gate, `Prometheus-AGS`
   for fabric/this repo). Cross-org git deps need access on every build machine
   — factor into the "portable across machines" goal.
8. **Missing from the spec:** versioning/migration story for the Project
   artifact and generated apps; observability/audit trail for agent-driven
   deploys (Base Rule 34); cost controls for KB embedding/LLM calls via Ember;
   a testing strategy for generated apps (who verifies the output compiles/ships).

---

## 6. Open questions for analyze/plan

- **Q1.** Is forge's gateway/A2UI registry on a near-term roadmap, or do we mock
  against the RFC indefinitely? (Determines whether integration is in-scope this phase.)
- **Q2.** Reference siblings via **git dependency** (recommended) — confirm tags
  exist (`proto-v1` on fabric?) and CI machines have cross-org SSH access.
- **Q3.** ✅ **Resolved (user, 2026-06-30):** agent consumes forge's A2UI;
  target is the flint-forge React SDK (`p5-c010`, unbuilt). Interim: locally-defined
  A2UI React components or HTMX prototyping behind a swappable seam, optionally
  `@prometheus-ags/a2ui-react` alpha. Replace with forge SDK when ready.
- **Q4.** Hydra: platform-wide auth decision or agent-local? Who owns the
  Postgres permission DDL — forge (`flint_auth`) or this agent?
- **Q5.** Scope cut for THIS phase: I recommend **(a) composition root + (b)
  Project domain model + (c) A2A task catalog skeleton + (d) MCP client
  transport** — and defer UI generation (#7), Tauri (#8), OpenDesign plugin (#6)
  to later phases. Confirm or adjust.

---

## 7. Stage handoff

**Key gaps:** agent is a clean protocol shell with zero orchestration logic; the
ambitious surface (KB, UI generation, OpenDesign, Tauri, auth) is largely
greenfield and several parts are **blocked on flint-forge** (stub gateway,
unbuilt A2UI registry). **Open for plan:** scope-cut decision (Q5), forge
dependency strategy (Q1/Q2), and the A2UI ownership reframe (Q3).
