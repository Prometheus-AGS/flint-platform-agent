# Execution — forge-integration-and-real-dispatch

**Phase:** forge-integration-and-real-dispatch
**Date:** 2026-06-30
**Changes:** 0/4 complete

## Backend selection

**Backend: `openspec`** (4 validated changes; spec→task traceability). Task
execution driven per the plan order, CI-gated between each.

## Dispatch contract

1. `p2-c001-credential-threading`  ← first (root)
2. `p2-c002-forge-read-integration`  (needs c001)
3. `p2-c003-catalog-input-validation`  (independent)
4. `p2-c004-task-store`  (independent)

Per change: implement tasks → `./scripts/ci-check.sh` green → live/wiremock
smoke where applicable → mark DONE in `progress.json` → next. One commit per change.

## Pre-flight (no blockers this phase)

- `reqwest` already in the workspace; `wiremock` 0.6 is a standard crates.io
  dev-dep (verify MSRV vs 1.93 at add — expected fine).
- No cross-org git deps. No new toolchain change expected.
- c001's A2A bearer path is fully unblocked. The **MCP identity** sub-part
  depends on the open question below.

## Open decisions (resolve at the relevant change)

- **c001 / Q4:** does `POST /mcp` carry a gate JWT (Authorization header)? If yes,
  reuse the c001 gate extractor for MCP `tools/call`. If not, MCP keeps a
  scoped-down default and the identity debt stays partially open. **Resolve before
  the MCP part of c001.**
- **c002 / Q3:** `describe_table` via OpenAPI vs GraphQL introspection.
- **c003:** minimal required-field check vs `jsonschema` crate (verify MSRV first
  — the phase-1 `a2a-protocol-types` lesson).

## First pending change

`p2-c001-credential-threading`.
