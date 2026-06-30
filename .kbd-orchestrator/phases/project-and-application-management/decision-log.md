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
