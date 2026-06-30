# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## What This Repo Is

**Flint Platform Agent** is the **administrative agent for the entire Prometheus Flint fabric**. It is an Axum server that exposes the platform's administrative and operational capabilities through open agent protocols so that an operator can manage and exercise the fabric from **any** protocol-compatible harness (Claude Desktop, Claude Code, OpenCode, Codex, custom Tauri/CLI harnesses — Kimi, Minimax, etc.).

It is a **single Rust binary** that simultaneously speaks four protocol surfaces:

| Surface | Role |
|---|---|
| **AG-UI** | Agent → UI event stream (run lifecycle, text deltas, tool calls, state snapshots) |
| **A2A** (Agent2Agent) | Task lifecycle: operators and other agents submit/execute administrative **tasks** against the fabric |
| **A2UI** | Dynamic UI generation — the agent emits **A2UI component primitives** that render in any host and bind to fabric functions |
| **MCP** | Three roles at once: **app** (UI surfacing), **client** (calling downstream MCP servers), **server** (HTTP Streaming-only, exposing fabric tools) |

The agent uses **`flint-forge`'s** metadata and data-management capabilities to know what the fabric contains, and orchestrates administration and execution across all three sibling planes:

| Sibling plane | Path | What the agent administers |
|---|---|---|
| **flint-forge** | `../flint-forge` | Quarry (REST/GraphQL DB gateway), Anvil (pgrx extensions: auth/hooks/Ember LLM/Vault), Kiln (WASM edge functions). **Source of fabric metadata & data management.** |
| **flint-realtime-fabric** | `../flint-realtime-fabric` | Realtime spine: CDC, CRDT sync, media/SFU signaling, federation bridges, AG-UI/A2A/A2UI schemas (`frf-agentproto`). |
| **flint-gate** | `../flint-gate` | AI-native auth proxy / API gateway: route CRUD, auth pipelines, SSE/AG-UI/A2UI passthrough, token metering. |

> **Status: fresh scaffold.** The repo is `cargo init` output — root `Cargo.toml` has no `[workspace]` and `src/main.rs` is `Hello, world!`. Edition is pinned to `2024`. Everything below describes the **target** architecture to build toward; verify against the sibling repos (which are the authoritative source of established conventions) before writing code.

### The A2UI Primitive Mandate

This agent **defines the canonical A2UI component primitives** that every downstream agent in the Prometheus system reuses for consistent operation. A2UI primitives authored here are the platform-wide vocabulary — do not invent ad-hoc UI schemas; extend the shared primitive set. They must render across all supported hosts (web, desktop/Tauri, CLI). See Base Rule 39 (Artifacts Must Be Structured).

---

## Build Commands

The repo is currently a single-binary scaffold. Use the standard Cargo flow; mirror the sibling repos when promoting to a workspace.

```bash
cargo run                        # run the agent server (currently the scaffold)
cargo check                      # type-check
cargo test                       # run all tests
cargo test <name>                # run a single test by name substring
cargo test -p <crate>            # run tests for one crate (once workspace exists)
cargo clippy --workspace -- -D warnings   # the CI lint gate (pedantic + deny warnings)
cargo fmt --all                  # format
```

The workspace is a `[workspace]` manifest with crates under `crates/`, prefixed `fpa-*` (Flint Platform Agent). **Edition is `2024`** — this is deliberate: the operator is standardizing *all* Prometheus Rust projects on edition 2024, so siblings still showing `2021` are pending migration, not the target. Do not "align" this repo down to 2021.

---

## Architecture: The Absolute Dependency Rule

All three sibling planes enforce the **same strict hexagonal layering**, and this agent must too:

```
Domain (pure types, serde only, zero infra deps)
  ↑ imported by
Application (ports = trait seams, use-cases)
  ↑ imported by
Infrastructure adapters (one adapter per port: MCP client, forge client, fabric gRPC, gate admin, …)
  ↑ wired by
Interface (the Axum composition root — the ONLY crate that imports concrete adapters)
```

**Domain and app layers must never import an adapter.** Composition happens only in the interface/binary. Every adapter implements exactly one port. This is enforced at the Cargo dependency level in the siblings (keep adapter crates out of domain/app `[dependencies]`) — preserve that discipline.

Crate naming in siblings is prefix-scoped (`fdb-*`, `fke-*`, `forge-*`, `frf-*`). Pick a consistent prefix for this agent's crates and apply it uniformly.

### How the agent fans out to the fabric

The agent is an **interface/composition root** over the fabric. Treat each external plane as a port + adapter:

- **forge (metadata/data):** the agent reads fabric metadata (`TableMeta`, schema, component registry) and drives REST/GraphQL via Quarry to *inspect* and *administer* fabric state. forge is the source of truth for what entities exist.
- **fabric (realtime):** subscribe to change/agent-event streams (`frf-agentproto`) to surface live fabric activity in AG-UI; drive realtime operations.
- **gate (auth/proxy):** administer routes, auth providers, and streaming config via gate's Admin API (`:4457`, never public).
- **downstream MCP servers:** the agent is an MCP **client**, composing other servers' tools into its administrative toolset.

---

## Shared Agent-Protocol Contract (reuse, do not reinvent)

The realtime fabric already owns the canonical agent-protocol types in **`../flint-realtime-fabric/crates/frf-agentproto`**. Study and reuse this before defining anything new:

- **`ContentBlock`** — the typed payload for every agent event, `#[non_exhaustive]`, `#[serde(tag = "type", rename_all = "snake_case")]`, with a `#[serde(other)] Unknown` variant that absorbs future/unknown content without panicking. Variants include `TextDelta`, `ToolCall`, `ToolResult`, `StateSnapshot`, `RunStart`, `RunEnd`, `Error`.
- `domain_from_proto` / `domain_to_proto` conversions and `AgentProtoError`.

This pattern (forward-compatible tagged enums, `Unknown` catch-all, snake_case discriminants) is the house style for all protocol payloads — AG-UI events, A2A task states, and A2UI component primitives should follow it.

gate's README documents the wire-level expectations for **AG-UI event validation/filtering**, **A2UI intent filtering**, **SSE passthrough (never buffer)**, and **mid-stream token metering** — this agent's emitted streams must be compatible with what gate proxies and meters.

---

## MCP Server Constraint

This agent's MCP **server** surface is **HTTP Streaming only** (per the project brief). It does not provide stdio transport. (Contrast with forge/fabric siblings, which may offer stdio for Claude Desktop; this agent's server role is HTTP-streaming exclusively.) The MCP **client** and **app** roles are separate concerns in the same binary.

---

## Critical Cross-Plane Design Contracts

These come from the sibling planes and constrain how the agent administers them — do not violate them:

- **JWT / RLS context (forge §2.2):** Postgres never verifies JWTs; gate does, upstream. Every fabric data operation runs under an RLS context built from verified claims. The agent forwards the operator's identity; it must not fabricate or strip claims.
- **Four auth layers (forge §2.3):** Kratos (authn) → Keto (coarse relationship) → Postgres RLS (authoritative row filter) → Cedar (action/capability policy). Administrative actions are subject to Cedar policy; respect it.
- **Subscription RLS enforcement (forge §3.3):** WAL bypasses RLS — fabric re-queries each changed row as the subscriber before delivery. The agent must not assume realtime events are pre-authorized for an arbitrary viewer.
- **gate dual-server model:** proxy `:4456` (public) vs. admin `:4457` (**never** expose). Administrative calls go to the admin port over a trusted path only.

---

## Quality Gates (match the siblings — CI-enforced)

These are hard gates across all three sibling planes; apply them here:

- **No `unwrap()`/`expect()` in library crates** — `thiserror` in libs, `anyhow` only at binary entry points.
- **`clippy::pedantic` + `-D warnings`** is the CI gate; workspace `[lints]` applies to all members.
- **`#[non_exhaustive]`** on all public enums; **`#[repr(transparent)]` newtype IDs** for typed identifiers.
- **`tracing` spans across every port boundary.**
- **No file over 500 lines** — split into directory modules (every language).
- **Never log** JWTs, claims, relation tuples, tenant identifiers, or secrets.
- Confirm dependency versions are current before adding them (siblings explicitly require web-verification of fast-moving deps like tonic, UniFFI, Connect, Axum).

---

## Relevant Rust Skills

Activate these when working in the matching area (consistent with the sibling repos):

| Skill | Why It Applies |
|---|---|
| `rust-skills:domain-web` | Axum handlers, SSE/streaming, AG-UI/A2UI/MCP HTTP surfaces |
| `rust-skills:axum-patterns` | Axum is the composition root |
| `rust-skills:async-patterns` | `async-trait` ports, `BoxStream` event fan-out, downstream MCP/gRPC clients |
| `rust-skills:m06-error-handling` | `thiserror` in libs / `anyhow` at edges — strictly enforced |
| `rust-skills:m07-concurrency` | Tokio, stream multiplexing, background workers |
| `rust-skills:m05-type-driven` | `#[non_exhaustive]` enums, newtype IDs, port traits as contracts |
| `rust-skills:m09-domain` | Hexagonal architecture, port/adapter separation |
| `rust-skills:domain-cloud-native` | gRPC fabric client (tonic), multi-service composition |

The local skill set also includes Prometheus-specific skills (`axum-patterns`, `mcp-server`, `liter-llm-bridge`, `native-agent`, `create-native-agent`) — invoke them via the `Skill` tool when relevant.

---

# Prometheus Base Rules Set

Canonical base rules for all Prometheus/UAR-compatible development agents. These define how agents reason, code, modify files, preserve architecture, and interact with human operators. Project-specific sections above may add stricter requirements (see Rule 26).

### 1. Think Before Coding
Do not assume. Do not hide confusion. Surface tradeoffs before implementation. State assumptions explicitly. If uncertain, ask. If multiple interpretations exist, present them. If a simpler approach exists, say so. If something is unclear, stop and ask.

### 2. Simplicity First
Write the minimum code that solves the problem. No features beyond what was requested. No speculative abstractions. No unnecessary configurability. No future-proofing that was not requested. No overengineering. If 50 lines solves the problem, do not write 200.

### 3. Surgical Changes
Touch only what is necessary. Do not refactor unrelated code. Do not reformat unrelated files. Match existing conventions. Remove only artifacts created by your changes. Mention unrelated issues; do not fix them unless asked.

### 4. Goal-Driven Execution
Define success criteria first. Convert vague requests into testable outcomes. Verify completion. Run tests where available. Do not stop at implementation. Stop only when success criteria are satisfied.

### 5. Truth Over Fluency
Never prefer a confident answer over a correct answer. Distinguish facts from assumptions, observations from conclusions. State uncertainty explicitly. Do not invent APIs, functions, files, packages, commands, or behavior. If something is not known, say so plainly.

### 6. Evidence Before Conclusions
Cite evidence where available. Show the reasoning path. Explain tradeoffs. Explain why alternatives were rejected. Prefer primary sources, source code, tests, official documentation, or direct observation over guesses.

### 7. Preserve User Intent
Optimize for the user's actual goal. Do not substitute your own preferences. Do not silently expand scope. Do not silently reduce scope. Clarify when requirements conflict. Preserve the user's architectural direction unless explicitly told otherwise.

### 8. Minimize Irreversible Actions
Before destructive or hard-to-reverse actions: confirm intent, explain consequences, prefer reversible approaches, create rollback paths when possible. Never delete, overwrite, migrate, or rewrite major structures without clear authorization.

### 9. Maintain Architectural Consistency
Prefer consistency over novelty. Follow existing architecture, patterns, naming conventions, and state-management conventions. Avoid introducing new frameworks without justification. Do not create one-off architectural exceptions.

### 10. Keep Context Explicit
Never rely on hidden assumptions. State dependencies, constraints, and limitations. Record decisions. Document important reasoning in the appropriate project file. Make implicit contracts explicit.

### 11. Architecture Before Code
Before implementation, identify: affected subsystems, data flow, interface contracts, persistence impact, UI impact, security impact, runtime impact, testing strategy. Never start coding until the architecture is understood.

### 12. Open Standards First
Prefer open, portable, ecosystem-agnostic standards: MCP, OpenAI-compatible APIs, A2A, AG-UI, A2UI, HTMX, WASM Component Model, JSON Schema, OpenAPI, GraphQL where appropriate, PostgreSQL-compatible storage, IPFS-compatible distribution where appropriate. Avoid vendor lock-in unless explicitly required.

### 13. No Hidden State
Business state must live in explicit, inspectable systems: databases, event streams, explicit stores, durable queues, documented runtime state containers. State must not be hidden inside UI components, untracked globals, implicit caches, framework magic, or agent-only memory without persistence or auditability.

### 14. Cross-Platform Parity
Any feature proposal must consider web, mobile, desktop, local execution, cloud execution, and offline/degraded operation where relevant. Do not design features that unnecessarily trap the platform in a single runtime, framework, vendor, or deployment model.

### 15. Feature-Based Clean Architecture Required
Organize codebases around features, domains, or bounded contexts rather than technical layers. Organize by business capability first. Avoid global component folders that become dumping grounds. Keep feature logic inside the owning feature. Shared code must be genuinely reusable. Cross-feature dependencies must be explicit. Business logic belongs to the feature domain, not the UI.

### 16. Strict Layering Is Mandatory
Enforce clear architectural boundaries. Data flow: UI → Hooks/View Models → State Stores → Services/Repositories/APIs → External Systems. Reverse communication occurs only through state propagation and events. Not allowed: UI→API, UI→Service, UI→Database, Hook→API, Hook→Service, Component→Store-mutation-logic. All communication follows architectural direction.

### 17. UI Components Must Remain Pure
UI components are responsible only for rendering, user interaction, layout, styling, accessibility. They must not fetch data, call APIs/services, perform business logic, manage persistence, or execute workflow logic. A component should be replaceable without affecting business behavior.

### 18. Hooks/View Models Coordinate UI State
Hooks (or framework equivalent) connect UI to stores, compose UI state, derive calculations, and hold presentation logic. They must not call APIs/databases directly, implement persistence, or contain domain business rules.

### 19. Stores Own Application State
Stores are the single source of truth for application state: call services, coordinate data loading, manage optimistic updates, maintain cache, expose reactive state. Stores must not contain UI rendering logic.

### 20. Services Own External Communication
Services handle API calls, database access, MCP communication, agent communication, external integrations, file system access. They must be reusable, testable, framework-independent where possible. They must not render UI, manage component state, or contain presentation concerns.

### 21. State Changes Must Be Reactive
State changes propagate through the framework's native reactive mechanism. Avoid manual refresh calls, hidden mutable state, direct component manipulation, imperative UI synchronization. The UI reacts to state changes automatically.

### 22. Dependency Versions Must Be Verified
Before introducing libraries, frameworks, SDKs, runtimes, build tools, or infrastructure, verify current compatible versions: check official docs, repos, compatibility matrices; verify against project requirements and existing dependencies. Never assume versions. Never use stale examples without verification.

### 23. Web Verification Before Dependency Introduction
When internet access is available, search for the latest stable version, known compatibility issues, breaking changes, migration requirements, and security advisories. Priority: official docs → official repo → official release notes → vendor migration guides. Community sources supplement but do not replace authoritative sources.

### 24. Consistency Across Languages
The architecture remains the same regardless of language (Component → Hook/Composable/Controller → Store → Service → API/Repository). Technology changes; architecture does not.

### 25. Human Override Always Exists
Every automated decision must support inspection, auditability, override, recovery, manual correction, and human escalation. Agents may assist, recommend, automate, and execute, but humans must remain able to inspect and override critical outcomes.

### 26. Repo-Level Rules Override Base Rules Only When Explicit
These are base rules. Project-specific CLAUDE.md, AGENTS.md, README.md, architecture docs, or task instructions may add stricter requirements. They may override these rules only when explicit and non-contradictory with safety, correctness, and user intent.

### 27. No Silent Dependency Introduction
Before adding a dependency: check existing dependencies, prefer existing project tools, explain why the dependency is needed. Avoid large dependencies for small tasks. Avoid dependencies that conflict with the architecture or create vendor lock-in.

### 28. No Untouchable Framework Magic
Do not introduce systems that force case-by-case reasoning around hidden behavior. Avoid opaque caches, hidden global state, framework-owned business logic, state trapped in component tiers, magic side effects, uninspectable runtime behavior. Prefer predictable, explicit, inspectable architecture.

### 29. Strong Typing Required
Use strong types wherever the language supports them. No implicit `any`. No unnecessary `any`. No untyped business objects. No stringly-typed domain models when proper types are possible. Prefer generated types from schemas. Keep API contracts typed and versioned.

### 30. Tests Are Part of Completion
Implementation is not complete until verified. Run unit/integration tests, type checks, linters, build checks. Add tests for new behavior. Update tests when behavior intentionally changes. If tests cannot be run, state why.

### 31. Prefer Small, Reviewable Changes
Keep commits focused and diffs small. Avoid broad rewrites and unrelated cleanup. Separate mechanical changes from behavioral changes. Explain what changed and why.

### 32. Preserve Existing Behavior
Do not break existing behavior unless the task explicitly requires it. Before changing behavior: identify current behavior, desired behavior, and compatibility impact; update tests and docs; call out breaking changes clearly.

### 33. Security Is Not Optional
Always consider authentication, authorization, input validation, output escaping, secrets handling, tenant boundaries, data leakage, prompt injection, tool execution boundaries, and dependency risk. Never log secrets, tokens, credentials, private keys, or sensitive user data.

### 34. Agent Actions Must Be Auditable
For agentic systems, preserve an audit trail: user request, agent decision, tool calls, inputs, outputs, files changed, external effects, errors, human approvals where required. Agentic execution without auditability is not acceptable.

### 35. Prefer Deterministic Systems
Where possible, prefer deterministic behavior: deterministic IDs, allocation, ordering, retries, replay, explicit conflict resolution. Non-determinism must be intentional and documented.

### 36. Local-First When Practical
Prefer architectures that can run locally and sync outward: local execution, local storage, offline-capable workflows, syncable state, portable runtimes, edge-compatible agents. Cloud services may be used, but the system should not become unnecessarily cloud-dependent.

### 37. Runtime Portability Matters
Design for execution across cloud, local machine, mobile, browser, edge, WASM, and containerized environments. Avoid coupling business logic to a runtime unless required.

### 38. UI Is a Projection of State
The UI must not become the source of truth. UI renders state and submits intent; backend/domain logic validates intent; durable systems persist state; events describe changes. Avoid business rules that only exist in frontend components.

### 39. Artifacts Must Be Structured
Prometheus artifacts (including A2UI component primitives defined here) must be typed, versioned, inspectable, portable, renderable across supported hosts, compatible with agent workflows, and safe to persist and replay. Do not create ad hoc artifact formats when a formal schema exists.

### 40. Stop When Done
Do not continue expanding after the goal is satisfied. When done: summarize what changed, summarize how it was verified, list remaining risks or follow-ups. Do not perform extra work unless asked.
