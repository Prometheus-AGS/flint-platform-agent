# Analysis — project-and-application-management

**Phase:** project-and-application-management
**Date:** 2026-06-30
**Mode:** stack-specified (Rust/Axum agent + React 19/Vite generated apps + flint fabric)
**Inputs:** `assessment.md` (+ user corrections: forge-owned A2UI, gate-only auth, forge-SDK UI target with local/HTMX interim).

> Engineering counterpart to Assess: turn the assessed gaps into evidence-backed
> **build-vs-adopt** calls before any spec. Machine contract in
> `library-candidates.json`; decisions appended to `decision-log.md`.

---

## 1. Landscape summary

The Rust agent-protocol ecosystem matured enough that the three protocol surfaces
each have a credible crate, but at **very different maturity tiers**:

- **MCP — solved.** `rmcp` is the *official* SDK (`modelcontextprotocol/rust-sdk`)
  and supports **stdio + streamable-HTTP for both server and client**. One
  dependency covers our HTTP-Streaming server constraint *and* the MCP client
  role (reqs #3, #4). High confidence; this is the clearest adopt in the phase.
- **A2A — adopt types, build transport.** `a2a-protocol-types` 0.6.0 is "pure
  data types, serde only, no I/O" — the exact Layer-0 shape our hexagonal
  architecture wants. Its companion server/client are **hyper-backed**, but we are
  **Axum**, so we take the types and put A2A on our own Axum routes (we already
  have stub routes). Medium confidence (early crate) → wrap behind our types.
- **AG-UI — keep in-house, borrow sparingly.** `agui-protocol` 0.1.0 exists
  (types + SSE parser) but is a 0.1 crate with little adoption. We already have a
  working `AgUiEvent` + Axum SSE implementation aligned to the fabric's
  `ContentBlock`. **Default: keep ours**; optionally lift `agui-protocol`'s SSE
  parser if it removes real work — but do not hang the core vocabulary on a 0.1.

The strongest *consistency* signal came from the sibling repos, not the registry:
**flint-gate verifies/mints JWT with `jsonwebtoken` 9 and talks over `reqwest`
0.12 (rustls)**, and the fabric reaches Ory only through its *own* adapters. With
the user's correction that **gate is the sole auth boundary**, the agent's auth
gap collapses to "consume gate's JWT" — `reqwest` + `jsonwebtoken`, zero Ory
crates. High confidence, and it matches gate's own stack exactly.

## 2. Build-vs-adopt calls (summary; full detail in library-candidates.json)

| Gap | Verdict | Confidence |
|---|---|---|
| MCP server (HTTP-stream) + client | **Adopt `rmcp`** | high |
| A2A protocol | **Adopt `a2a-protocol-types`**, build Axum transport | medium |
| AG-UI events + SSE | **Keep in-house** (Axum SSE + `frf-agentproto` parity); `agui-protocol` optional | high |
| Auth (consume gate) | **Adopt `reqwest` + `jsonwebtoken`**, NO Ory crates | high |
| Fabric type parity | **Adopt `frf-agentproto` as git dep @ `proto-v1`** | high |
| Frontend A2UI | **Target forge React SDK** (unbuilt); interim local/HTMX + `@prometheus-ags/a2ui-react` alpha | medium |
| Tauri shell | Adopt Tauri 2 (later phase) | high |
| OpenDesign plugin | Integrate later phase (authored in OD's stack) | medium |
| Knowledge base | Target forge `ext-flint-llm`/Ember; abstraction now | medium |

## 3. Cross-repo dependency strategy (verified)

- **`frf-agentproto` via git dep pinned to tag `proto-v1`** (tag confirmed to
  exist). Reproducible, machine-portable, no vendoring.
- **No Rust reuse from gate** (ships Go/TS/Flutter SDKs only) → `fpa-gate` is a
  thin `reqwest` client to gate's admin HTTP API.
- **forge crates are workspace-internal and its gateway is a stub** → code
  `fpa-forge` to forge's documented REST/GraphQL/MCP contract; do not link forge
  crates.
- **Two GitHub orgs** (`Know-Me-Tools`: forge/gate; `Prometheus-AGS`: fabric/this
  repo). Git deps need cross-org SSH on every build/CI machine — a real
  portability cost to plan for.

## 4. Evidence (tiered)

- **Tier 1 (gh search):** returned empty in this environment (auth/quoting) —
  compensated below. *Logged as a partial-tier per the budget rule, not retried.*
- **Tier 3 (cargo search / docs.rs):** `rmcp` 2.0.0 (docs.rs shows a 1.7.0 line —
  reconcile at pin time); `a2a-protocol-types/-server/-client` 0.6.0, `a2a-rs`
  0.4.1; `agui-protocol` 0.1.0 + `agui-rs-{server,client}` 0.1.2;
  `ory-kratos-client` 26.x / `ory-client` 1.x (NOT adopted — gate-only).
- **Tier 3 (sibling inspection):** gate uses `jsonwebtoken` 9 + `reqwest` 0.12
  rustls; fabric `frf-agentproto` tagged `proto-v1`; forge `fdb-gateway` = stub.
- **Tier 4 (firecrawl):** `rmcp` = official SDK with streamable-HTTP (server+
  client); shadcn+Base UI viable; assistant-ui active; Tauri 2 mobile/PWA;
  OpenDesign plugin model (`open-design.json`, skills→plugins, serialized export).

## 5. Open questions (for Spec/Plan)

1. **`rmcp` version line** — crates.io 2.0.0 vs docs.rs 1.7.0. Confirm the
   canonical/maintained line and the streamable-http server feature flag before pinning (Base Rule 22).
2. **A2A: adopt `a2a-protocol-types` vs hand-roll on `frf-agentproto`.** Types
   crate is cleaner but early; fabric parity argues for hand-roll. *Recommend
   adopt-types-behind-our-own-wrapper so a later swap is cheap.* **Decision needed at spec.**
3. **forge readiness (carried from assess Q1):** mock against forge contracts
   indefinitely, or is forge gateway/A2UI registry near-term? Gates how much
   integration is in-scope this phase.
4. **Hydra ownership (carried from assess Q4):** confirmed gate-side; still need
   to confirm who owns the Postgres permission DDL (forge `flint_auth` vs this agent).

## 6. Handoff to Spec

Adopt set: **`rmcp`** (MCP server+client), **`a2a-protocol-types`** (A2A types,
Axum transport in-house), **in-house AG-UI** on Axum SSE + **`frf-agentproto` git
dep @ proto-v1**, **`reqwest`+`jsonwebtoken`** to consume flint-gate (no Ory
crates). Frontend defers to the forge React SDK behind a swappable local/HTMX
seam. Tauri, OpenDesign, and the KB wiring are later-phase. **Spec must decide:**
the A2A types-vs-hand-roll call (Q2) and the `rmcp` version line (Q1).
