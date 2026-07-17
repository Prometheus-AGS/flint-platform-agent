# Current Waypoint

**Phase:** a2a-task-catalog
**Status:** execution_ready
**Changes:** 0/3 complete
**Backend:** OpenSpec (phase p13)
**Active change:** p13-c001-split-task-runner

## Ordered changes (c001 first; c002/c003 both depend on c001)
1. **p13-c001-split-task-runner** ← next (mechanical: split 828-line `task_runner.rs` into
   `task_runner/{mod.rs, tests.rs}` under the 500-line CI gate before adding arms)
2. **p13-c002-forge-describe-name** (G1 real bug: thread validated `table` name into
   `describe_table`; argument-asserting test)
3. **p13-c003-gate-mcp-read-kinds** (G2+G3: add `gate.route.list` [Gate/operator] +
   `mcp.tool.list` [Mcp/viewer] reads; register `GATE_READ_KINDS`; add Mcp dispatch guard)

## Next command
```
/kbd-apply p13-c001-split-task-runner
```

Posture: agent-only (no sibling repo, no fabric), reads-only (writes deferred), no new
ports (all seams exist), flint-gate the only auth boundary, ≤3 `cargo test` runs.

Plan: `.kbd-orchestrator/phases/a2a-task-catalog/plan.md`
Updated: 2026-07-08T08:35:00Z
