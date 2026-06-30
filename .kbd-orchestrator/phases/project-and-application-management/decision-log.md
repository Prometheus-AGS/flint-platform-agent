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
