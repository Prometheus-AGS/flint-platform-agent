# Execution — project-and-application-management

**Phase:** project-and-application-management
**Date:** 2026-06-30
**Changes:** 0/4 complete

## Backend selection

**Backend: `openspec`** (spec-backed traceability; changes already scaffolded +
validated). Task execution is driven through **`/kbd-apply`** (the KBD-owned apply
driver that fires per-task hooks and updates `progress.json`/waypoint) — **not**
bare `/opsx:apply`, which is KBD-unaware.

| Backend | Considered | Verdict |
|---|---|---|
| openspec | ✅ | **chosen** — 4 validated changes; spec→task traceability |
| native-tool | — | n/a (no separate planning tool in play) |
| hybrid | — | unnecessary |
| manual | — | n/a |

## Dispatch contract

Apply in plan order, one change at a time, CI-gated between each:

1. `p1-c001-composition-root`  ← **first / unblocked**
2. `p1-c002-project-domain-model`
3. `p1-c003-a2a-task-catalog`
4. `p1-c004-mcp-transport`

Per change: implement tasks → `./scripts/ci-check.sh` green → artifact-refiner QA
gate (if ≥3 non-doc files) → `/opsx:verify` → `/opsx:archive` → mark DONE in
`progress.json` → next.

## Pre-flight dependency resolution (blocking checks done at execute)

1. **Cross-org SSH:** ✅ authenticated to GitHub (GQAdonis). Both orgs reachable.
2. **⚠️ `frf-agentproto` git-dep @ `proto-v1` is NOT resolvable.** The fabric
   remote (`Prometheus-AGS/flint-realtime-fabric`) has **only `main` — the
   `proto-v1` tag exists locally but was never pushed.** A `tag = "proto-v1"` git
   dependency would fail to resolve on every machine/CI.
   - **Resolution:** c001 does **not** need `frf-agentproto` (it uses
     `reqwest`/`jsonwebtoken` only). **Defer the fabric git-dep to c003**, where
     protocol parity first matters. Before c003, either (a) push the `proto-v1`
     tag to the fabric remote, or (b) pin the git-dep to a **commit SHA on
     `main`** (`696f68e…`) instead of the tag, or (c) hand-roll A2A on local
     types (the c003 fallback). **Decision owner: operator, at c003.**
3. **`rmcp` version line (c004):** unresolved; confirm canonical line +
   streamable-http feature before c004.
4. **A2A adopt-vs-hand-roll (c003):** unresolved; decide at c003 task 1.1.

→ **c001 is fully unblocked.** Its only external deps (`reqwest`, `jsonwebtoken`)
are uncontested and on crates.io.

## First pending change

`p1-c001-composition-root` — next command: `/opsx:apply p1-c001-composition-root`
(driven via `/kbd-apply` when the apply driver is available).
