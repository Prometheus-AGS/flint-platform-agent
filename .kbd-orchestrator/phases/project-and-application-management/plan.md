# Plan — project-and-application-management

**Phase:** project-and-application-management
**Date:** 2026-06-30
**Backend:** OpenSpec (changes already scaffolded + validated)
**Changes this phase:** 4 (scope-cut per assessment Q5; UI/Tauri/OpenDesign/KB deferred)

> Ordered change list the Execute stage drives one task per turn. Library
> annotations come from `library-candidates.json`; ordering follows the
> dependency edges declared in each change's proposal.

---

## Dependency graph

```
p1-c001-composition-root ─┐
                          ├─► p1-c003-a2a-task-catalog ─► p1-c004-mcp-transport
p1-c002-project-domain-model ─┘
```

- `c001` and `c002` are **independent roots** (could run in parallel).
- `c003` requires `c001` (TaskRunner state) **and** `c002` (Project model for task inputs).
- `c004` requires `c003` (MCP tools surface the catalog).

Single-developer assumption → linear order. If parallelized, c001 ∥ c002 first.

---

## Ordered change list

### 1. `p1-c001-composition-root`  ·  root, no deps
**Goal:** Wire the four plane adapters into `TaskRunner` and inject as Axum state; extract gate-only operator identity.
- **Library:** `reqwest` 0.12 (rustls), `jsonwebtoken` 9 — adopt; matches flint-gate's own stack. **No Ory crates.**
- **Risk:** low (in-repo wiring; adapters stay `todo!()`).
- **Recommended agent:** `rust-build-resolver` / general Rust; review with `rust-reviewer`.
- **Gate to next:** workspace builds with `AppState`; gate-identity extractor unit-tested.

### 2. `p1-c002-project-domain-model`  ·  root, no deps
**Goal:** Define the `Project` artifact aggregate + versioned JSON Schema; A2UI refs conform to forge `RFC-FORGE-A2UI-001`.
- **Library:** `schemars` (verify version at impl) for JSON Schema — adopt-or-hand-author (decide in change task 2.3).
- **Risk:** low–medium (pure Layer-0 types; the schema-gen choice is the only fork).
- **Recommended agent:** general Rust; review with `rust-reviewer` + `type-design-analyzer`.
- **Gate to next:** serde round-trip + schema-validation tests pass.

### 3. `p1-c003-a2a-task-catalog`  ·  needs c001 + c002
**Goal:** Task catalog + `TaskRunner::run` dispatch through ports, permission enforcement, audit.
- **Library:** `a2a-protocol-types` 0.6.0 — **adopt behind `fpa-protocol` wrapper** (OPEN: vs hand-roll on `frf-agentproto`; resolve at execute, change task 1.1).
- **Risk:** medium (early crate; the adopt-vs-hand-roll fork). Mitigation: wrapper isolates the choice.
- **Recommended agent:** general Rust + `tdd-guide` (mockall fakes for the four ports); review with `rust-reviewer`.
- **Gate to next:** catalog dispatch + permission-deny + audit tests pass with fake adapters.

### 4. `p1-c004-mcp-transport`  ·  needs c003
**Goal:** `rmcp` MCP server (HTTP-Streaming only) + client; expose catalog as tools; define skill format.
- **Library:** `rmcp` (official MCP Rust SDK) — adopt (OPEN: version line 2.0.0 vs 1.7.0 + streamable-http feature; resolve at execute, change task 1.1).
- **Risk:** medium (version-line ambiguity; external transport). Mitigation: confirm canonical line before pinning (Base Rule 22).
- **Recommended agent:** general Rust + `docs-lookup` (rmcp API); review with `rust-reviewer`.
- **Gate to phase reflect:** server `initialize`/`tools/list`/`tools/call` + client round-trip pass; HTTP-Streaming-only confirmed.

---

## Cross-cutting (apply to every change)

- Hexagonal rule: domain/app import no adapter; wiring only in `fpa-gateway`.
- CI gate green per change: `./scripts/ci-check.sh` (fmt + clippy::pedantic -D warnings + check + test).
- Never log JWT/claims/secrets; `tracing` spans at port boundaries.
- One commit per change (or per task group); keep diffs reviewable.

## Open decisions to resolve at execute (do not silently pick)
1. **c003:** `a2a-protocol-types` adopt vs hand-roll on `frf-agentproto`.
2. **c004:** `rmcp` version line (2.0.0 vs 1.7.0) + streamable-http feature flag.
3. **c001:** `frf-agentproto` git-dep @ `proto-v1` — confirm cross-org SSH on CI machines before adding.

## Deferred to later phases (not this plan)
UI generation/embedded chat (#7), Tauri desktop+mobile (#8), OpenDesign plugin (#6), knowledge-base wiring (#5) — all blocked-on or large; revisit after forge gateway/A2UI registry land.

---

## Execute order

```
1) /opsx:apply p1-c001-composition-root
2) /opsx:apply p1-c002-project-domain-model
3) /opsx:apply p1-c003-a2a-task-catalog
4) /opsx:apply p1-c004-mcp-transport
```

First change to apply: **`p1-c001-composition-root`**.
