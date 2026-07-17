# Plan — a2a-task-catalog-writes (p14)

**Backend:** OpenSpec · **Changes:** 2 · **Analyze:** skipped (no library candidates) ·
**Test budget:** ≤3 `cargo test` runs — this phase uses **1** (batch c001 + c002).

## Source

- `assessment.md` — GW1 GREEN (implementable, no new port), GW2 RED (blocked, declared
  refused write).
- `handoffs/spec.handoff.json` — 2 validated OpenSpec changes.
- Both changes wire against **existing** ports (`McpClient::call_tool` is real; the GW2
  blocker is a missing gate contract, not a library) → no `library: cand-###` annotations.

## Ordered changes

### 1. p14-c001-mcp-tool-call  *(real deliverable — do first)*

- **Scope:** `crates/fpa-app/src/catalog.rs`, `crates/fpa-app/src/task_runner/mod.rs`,
  `crates/fpa-app/src/task_runner/tests.rs`.
- **Delivers:** GW1 (`mcp.tool.call` catalog entry → `TargetPort::Mcp` / operator /
  `SCHEMA_MCP_TOOL_CALL`), GW3 (`dispatch_mcp` split with clean unknown-kind refusal +
  extended `every_mcp_catalog_kind_is_dispatched`), GW4 (arguments-not-logged test +
  non-idempotent doc). GW5 (invariants) falls out of the real schema + unique kind.
- **No new port** — `McpClient::call_tool` + the `fpa-mcp` adapter already exist end-to-end.
- **Why first:** it is the phase's one real new invoke kind; c002 is documentation that
  references the same runner file, so landing c001's `dispatch_gate`-neighboring edits first
  avoids a second pass over `task_runner/mod.rs`.
- **Recommended agent:** primary implementer (Rust). Revier: `rust-reviewer` after edits.
- **QA gate:** artifact-refiner **runs** (3 files, behavioral change).

### 2. p14-c002-application-deploy-refused-write  *(honest-refusal docs — do second)*

- **Scope:** `crates/fpa-app/src/task_runner/mod.rs` (refusal message string only),
  `task_runner/tests.rs` (one regression test), spec + a `.kbd-orchestrator` memory.
- **Delivers:** GW2 — `application.deploy` stays a **declared refused write**; refusal
  message tightened to name the missing gate admin write contract; regression test asserts
  it still refuses (never `Ok`) + message marker; 6-step gate-write dependency documented in
  the spec and a memory.
- **No** catalog change, **no** new port, **no** sibling edit, **no** behavior change beyond
  message text.
- **Why second:** trivially small and message/docs-only; it touches the same file c001 edits
  (`task_runner/mod.rs`), so sequencing after c001 keeps the diff clean and avoids a merge
  seam.
- **Recommended agent:** primary implementer (message + memory).
- **QA gate:** artifact-refiner **may be skipped** — <3 files, effectively message + docs
  (kbd-execute skip rule).

## Ordering rationale

c001 → c002. Both edit `task_runner/mod.rs`; doing the substantive `dispatch_mcp` split
(c001) before the one-line refusal-message change (c002) means a single coherent pass over
the runner rather than two interleaved ones. c002 has no dependency on c001's behavior — the
order is about diff hygiene, not correctness.

## Integration milestone (test budget)

One `cargo test -p fpa-app` run covers **both** changes' assertions:
- c001: catalog invariants (3), extended Mcp dispatch guard, invoke-passthrough,
  sub-operator-denied-before-port, unknown-kind-refusal, arguments-not-logged.
- c002: `application.deploy` still-refuses + message-marker regression.

Then `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` at phase
end. Reserve the other 2 test runs for re-runs if the first surfaces a real integration gap.

## Guardrails carried from assessment

- `mcp.tool.call` role = **operator** floor (invoke, not discovery; not admin).
- Unknown Mcp kind → clean `Downstream`, **never** a `list_tools` fallback.
- `application.deploy` **never returns `Ok`** — assert on refusal, never fabricate success.
- No new port; no sibling-repo edit; no `unwrap`/`expect` in lib crates; ≤500 lines/file;
  never log arguments/tokens/claims.
