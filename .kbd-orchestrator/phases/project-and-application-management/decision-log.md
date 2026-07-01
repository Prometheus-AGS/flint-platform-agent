### 2026-06-30T16:49:05Z — Analyze: build-vs-adopt verdicts
Mode: stack-specified | Tiers: gh(empty), cargo-search, sibling-inspection, firecrawl
Decisions:
- MCP: ADOPT rmcp (official SDK, stdio+streamable-HTTP, server+client). Provenance: research (high)
- A2A: ADOPT a2a-protocol-types (serde-only) + in-house Axum transport; hand-roll on frf-agentproto is fallback. Provenance: research (medium) — final call deferred to Spec
- AG-UI: KEEP in-house types on Axum SSE + frf-agentproto parity; agui-protocol 0.1.0 optional SSE-parser only. Provenance: research (high)
- Auth: ADOPT reqwest + jsonwebtoken to consume flint-gate ONLY; NO Ory crates. Provenance: user + sibling-consistency (high)
- Fabric parity: ADOPT frf-agentproto as git dep @ tag proto-v1. Provenance: research (high)
- Frontend: TARGET forge React SDK (p5-c010, unbuilt); interim local/HTMX + @prometheus-ags/a2ui-react alpha behind a swappable seam. Provenance: user (high)
Open for Spec: rmcp version line (2.0.0 vs 1.7.0); A2A adopt-types vs hand-roll.

### 2026-06-30T17:07:27Z — Spec: 4 OpenSpec changes created (scope-cut applied)
Backend: OpenSpec (resolved via openspec_available=true). ZeeSpec gate: n/a (no .zeespec).
Scope cut (assess Q5): foundational unblocked work this phase; UI/Tauri/OpenDesign/KB deferred.
Changes:
- p1-c001-composition-root  — wire adapters → TaskRunner as Axum state; gate identity extractor (no Ory)
- p1-c002-project-domain-model — Project artifact aggregate + JSON Schema; A2UI refs conform to forge RFC-FORGE-A2UI-001
- p1-c003-a2a-task-catalog  — task catalog + TaskRunner dispatch (adopt a2a-protocol-types behind wrapper; OPEN Q at impl)
- p1-c004-mcp-transport     — rmcp server (HTTP-stream only) + client; skill format (OPEN Q: rmcp version line)
All four pass . Two decisions deferred into change Open Questions, to resolve at execute.

### 2026-06-30T17:16:20Z — Plan: ordered 4 changes
Order: c001 ∥ c002 (roots) → c003 (needs both) → c004 (needs c003). Linear for single-dev.
First to apply: p1-c001-composition-root. Library annotations carried from analyze.
Waypoint refreshed; plan_complete=true; active_change=p1-c001-composition-root.

### 2026-06-30T17:58:02Z — Execute: backend=openspec; c001 unblocked; fabric tag blocker found
Backend: openspec, driven via /kbd-apply (not bare /opsx:apply).
BLOCKER found: frf-agentproto proto-v1 tag is NOT pushed to the fabric remote
(remote has only main). A tag git-dep would fail everywhere. Resolution: c001
needs no fabric dep; defer to c003 — then push the tag, OR pin to main SHA
696f68e, OR hand-roll A2A locally. c001 deps (reqwest, jsonwebtoken) are clean.

### 2026-06-30T21:49:01Z — c003 executed: A2A adopt + MSRV bump
DECISION (user): adopt a2a-protocol-types 0.6 behind fpa-protocol wrapper.
CONSEQUENCE: 0.6 requires rustc 1.93 > our MSRV 1.85. DECISION (user): bump
workspace MSRV 1.85→1.93. Updated: Cargo.toml rust-version, rust-toolchain.toml
channel, CI msrv job, README, constraints.md, project.json. Installed rust 1.93.1.
Built: fpa-protocol a2a_std wrapper, fpa-app catalog (8 kinds) + AuthContext +
TaskRunner dispatch (permission-before-port + audit), ApiError mapping, A2A route
wiring. Verified live: viewer→list allowed, viewer→deploy 403, unknown→404,
audit logs decisions w/o secrets. c003 → done (3/4).

### c004 executed: MCP via canonical mcp-server skill (NOT rmcp) — verdict reversed
User steer + mcp-server skill: the canonical Prometheus MCP-server pattern is
hand-rolled JSON-RPC 2.0 over Axum, NOT rmcp's ServerHandler. REVERSED the
analyze/spec "adopt rmcp" verdict: reverted rmcp deps; extended routes/mcp.rs to
generate tools/list from the catalog and route tools/call through TaskRunner
(shared permission+audit). MCP CLIENT (fpa-mcp) = hand-rolled JSON-RPC over
reqwest (no rmcp). Skill format documented (skills/README.md + list-projects
example). Also hardened: replaced adapters' todo!() with PortError::Downstream so
unimplemented planes don't panic the request path. c004 → done (4/4).
