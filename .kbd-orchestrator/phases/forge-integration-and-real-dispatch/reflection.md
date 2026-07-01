# Reflection — forge-integration-and-real-dispatch

**Phase:** forge-integration-and-real-dispatch
**Date:** 2026-07-01
**Changes:** 4/4 executed (p2-c001–c004), all CI-green and pushed.

> Written against the sycophancy gate: names deltas and debt, not just wins. This
> phase turned the agent from a shell into something that can actually read fabric
> state under RLS. Unlike phase 1 (blocked on forge), this phase was unblocked and
> delivered its core goal.

---

## 1. Goal achievement

Five goals (goals.md), scored against what shipped:

| Goal | Verdict | Evidence |
|---|---|---|
| Implement `fpa-forge` (read-only) against forge HTTP | **MET** | c002: real `GET /openapi.json` metadata + `POST /graphql` data; 6 wiremock tests |
| Forward gate JWT as bearer for RLS | **MET** | c001: bearer threaded surfaces→runner→forge port; forge `graphql_query` sends `Authorization: Bearer`; missing/401 → `Unauthorized` |
| Per-kind input-schema validation | **MET** | c003: `jsonschema` validation → `InvalidInput`; real MCP `inputSchema` |
| In-memory task store (`status`/`cancel`) | **MET** | c004: `TaskStore`; live status/404/cancel-409 verified |
| Propagate gate identity into MCP | **MET** | c001: MCP `tools/call` requires gate JWT via `OptionalFromRequestParts` |

**Overall: ~90% — all five goals MET at the read/plumbing level.** The residual
10% is depth of test coverage and a few small guards (see debt), not missing
capability. Notably this phase **closed 3 of the 8 phase-1 debts** (MCP identity,
input validation, task store).

---

## 2. Delivered changes

| Change | Delivered | Commit |
|---|---|---|
| c001 credential-threading | raw bearer on `OperatorContext`/`AuthContext` (redacted Debug); `ForgeMetadata` methods take `bearer`; MCP identity required | `9df0bf9` |
| c002 forge-read-integration | `ForgeAdapter` real HTTP (OpenAPI + GraphQL), error mapping, `graphql_query` helper; wiremock | `9e58417` |
| c003 catalog-input-validation | `jsonschema` 0.46; per-kind `input_schema_json`; runner validation; MCP schemas | `7f29ba6` |
| c004 task-store | `TaskStore` (tokio `RwLock<HashMap>`); real A2A status/cancel; `ApiError::conflict` | `0a37183` |

All on `origin/main` (+ doc/decision-log commits).

---

## 3. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with artifact-refiner QA | **0/4** |
| First-pass CI-gate pass rate | 4/4 (after in-loop fixes) |
| Changes needing mid-implementation rework | 2 (c001 axum `OptionalFromRequestParts`; c004 tokio dep scope) |

**artifact-refiner QA gate did not run** — the refiner/hooks subsystem is not
installed (same as phase 1). Quality enforced by `./scripts/ci-check.sh` (fmt +
clippy::pedantic -D warnings + check + test) + wiremock/live smoke per change.
Process gap, not a quality gap — but no formal constraint report exists.

---

## 4. Technical debt introduced / carried

**New this phase:**
1. **Write-kind guard missing** — write kinds (`project.create`, `application.define`) route to the read fallback in `dispatch_forge` instead of returning `Downstream("write API pending")`. Spec task c002/3.3 was checked but not actually implemented. **Honest miss.**
2. **Checkbox overstatement** — a few auto-marked tasks overstate dedicated unit-test coverage: c001 4.2/4.3 (bearer-carried / none) verified via smoke not unit tests; c002 4.5 (live-forge smoke) deferred; c004 1.1 `TaskRecord` has no timestamps despite the task text. Corrected where caught; noted here.
3. **Task store is in-memory + single-process** — lost on restart; not shared across instances. Fine for now (YAGNI), but not durable.
4. **`describe_table` bearer ignored** — reads public OpenAPI only; if per-tenant schema visibility is ever needed it must move to an RLS'd GraphQL introspection.

**Still carried from phase 1:**
5. All non-forge adapters (`fabric`, `gate`) still `PortError::Downstream("not implemented")`.
6. Interim JWT signature verification (unverified when `FPA_GATE_JWT_KEY` unset).
7. Forge **writes** unimplemented (read-only phase by design).
8. MCP client: single endpoint, no live round-trip test.

---

## 5. Lessons captured

- **MSRV-first verification paid off.** Both new deps (`jsonschema` 0.46, `wiremock` 0.6) were MSRV-checked against 1.93 via `cargo add --dry-run` *before* adding — no repeat of phase 1's `a2a-protocol-types` surprise. This is now a reliable habit.
- **House-skill-first for external crates.** Loading `mcp-server` in phase 1 saved this phase from re-deriving MCP; the JSON-RPC pattern extended cleanly for MCP identity. Confirms: consult the pattern skill at analyze/execute before adopting a framework crate.
- **axum 0.8 `Option<Extractor>` needs `OptionalFromRequestParts`**, not plain `FromRequestParts`. The blanket impl was removed; you must impl it explicitly. Cost one build cycle.
- **`null` vs empty-object in schema validation.** No-arg MCP/A2A calls send `null` arguments; validating `null` against `{"type":"object"}` rejects legitimate calls. Normalize `null → {}` before validation. A real correctness catch, not a test tweak.
- **Redact secrets in manual `Debug`, don't derive.** `OperatorContext`/`AuthContext` carry the bearer; deriving `Debug` would leak it. Manual `Debug` with `<redacted>` + `skip_all` on the instrument span, verified by a no-token-in-logs smoke.
- **Dep scope matters:** `tokio` used in non-test code can't be a dev-dependency — a 30-second fix but a clean reminder to place deps by where they're used.

---

## 6. Recommended Next Phase

**`forge-writes-and-realtime`** — extend from read-only to a working
administrator: forge **mutations** (GraphQL under RLS) for `project.create` /
`application.define`, and wire the **fabric** adapter (realtime health +
subscriptions) so `fabric.health` and live AG-UI events work.

Scope (proposed):
1. Forge **write** operations via GraphQL mutations (bearer→RLS), replacing the
   read-fallback guard — and add the explicit write-kind guard (debt #1).
2. Implement `fpa-fabric` against the realtime fabric (health first, then a
   subscription that feeds AG-UI SSE) — closes carried debt #5 (fabric half).
3. Harden test coverage for the checkbox-overstated items (debt #2): unit tests
   for bearer-carried/none and MCP schema advertisement.
4. (Stretch) durable task store or at least timestamps + list.

**Prerequisites / open questions:**
- Does forge expose GraphQL **mutations** for project/application create today, or
  only reads? (Determines whether writes are in-scope or another blocked item.)
- Fabric's client transport (gRPC/WS) — confirm the endpoint contract like we did
  for forge this phase.

Still deferred: OpenDesign plugin (#6), React/Vite UI + generator (#7), Tauri
(#8), knowledge base (#5), gate admin real adapter, full JWT/JWKS verification.

---

## 7. Reflect handoff

Phase MET all 5 goals at read/plumbing level; agent now reads fabric under RLS
with gate-bearer forwarding, validates inputs, and tracks task state. Deltas:
write-kind guard not implemented (honest miss), a few checkboxes overstate unit
coverage, task store non-durable. Corrective actions: next phase = forge writes +
fabric adapter + test-coverage hardening. Recommended next phase:
**forge-writes-and-realtime** — pending confirmation forge exposes mutations.
