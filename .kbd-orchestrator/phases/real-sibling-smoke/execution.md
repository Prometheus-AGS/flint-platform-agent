# Execution — real-sibling-smoke

**Stage:** execute
**Backend:** `openspec` (spec-backed traceability; changes under `openspec/changes/p10-c00*`).
**Date:** 2026-07-04

## Backend selection

OpenSpec is present and every change is `openspec validate --strict` clean. Execution
drives the changes in the plan's dependency order, one change per apply, KBD as source of
truth. Rust changes (c004) follow the repo's **implementation-first** rule: write the
whole change, one batched `cargo check`, cheap unit test; container changes (c001–c003,
c005, c006) are Docker/compose authoring + a live `run-real.sh` integration milestone.

## Dispatch order (from plan.md)

1. **c004** fabric-realtime-client — only new Rust; compile first.
2. **c002** real-forge — author `fdb-gateway` Dockerfile + migrate.
3. **c001** real-gate — reuse sibling recipe.
4. **c003** real-fabric — heaviest boot.
5. **c005** compose-real-and-smoke — integration milestone (a full-run wait).
6. **c006** full-pgrx-image — best-effort opt-in.

## Grounding correction applied at execute (c004)

Verified `/ws/v1/subscribe` against live `frf-gateway/src/routes/subscribe.rs`: it
serializes **`frf-domain::EventEnvelope`**, not `frf-agentproto::ContentBlock`
(ContentBlock is the `/ws/v1/agents` bus type). c004 spec/proposal/tasks + c005 amended
to `EventEnvelope`; operator confirmed the type choice (frf-domain path dep — light:
serde/uuid/chrono, no prost/tonic). Re-validated `--strict`. Recorded in memory
`fabric-subscribe-wire-contract`.

## Per-change QA

Rust changes: `cargo check` + `clippy -D warnings` + `fmt` as the gate (no artifact-refiner
constraint file wired for this Rust repo). The phase's real QA is c005's `run-real.sh`.
