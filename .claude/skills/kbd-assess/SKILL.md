---
license: MIT
name: kbd-assess
version: '1.0.0'
description: >
  Assess the current project codebase against the active phase goals.
  Project-agnostic: reads AGENTS.md, spec files, and codebase to produce
  a structured assessment. Works for any technology stack or language.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-assess

Run the **Assess** phase of the KBD lifecycle for any project.

## What this does

Inspects the current codebase and produces a structured gap report against the
active phase's goals. Output written to
`.kbd-orchestrator/phases/<phase-name>/assessment.md`.

Also reads `progress.json` to incorporate any work completed by other tools
(Roo, Cursor, Cline, Codex, etc.) since the last session.

## Progress Signals (MANDATORY)

**FIRST tool call of every turn:** Read `.kbd-orchestrator/position-reminder.txt` (if it exists) to get the current phase, step N of T, and next command. If that file is absent, read `.kbd-orchestrator/current-waypoint.json`.

Before any other action, emit to plain response text (BEFORE any tool call):

```
Starting kbd-assess — <phase-name> (step N of T)
```

When all steps are complete, emit:

```
Completed kbd-assess — <phase-name> (step N of T)
```

**How to get N and T (MANDATORY — never estimate):**
- Read `.kbd-orchestrator/phases/<phase>/progress.json` → `changes_completed` = N, `changes_total` = T
- If `progress.json` is absent, read `current-waypoint.json` → `changes_completed` / `changes_total`

Use the canonical phase name from the argument or `current-waypoint.json`. Emit to plain response text — no tool call needed.

## How to invoke

1. **Discover project identity** — read `.kbd-orchestrator/project.json` or infer
   from `AGENTS.md`, `CLAUDE.md`, `README.md`, `package.json`, `Cargo.toml`, etc.
2. **Confirm the active phase** — from argument or `.kbd-orchestrator/current-waypoint.json`
3. **Resume from progress** — read `.kbd-orchestrator/phases/<phase>/progress.json`
   to account for cross-tool work done
4. **Load specs** — read `openspec/specs/*.md` if OpenSpec is available,
   otherwise read the canonical spec files defined in `.kbd-orchestrator/project.json`
5. **Inspect the codebase** — scan feature directories, components, routes, etc.
6. **Follow the assess protocol** in `../prompts/assess.md`
7. **Write assessment file** to `.kbd-orchestrator/phases/<phase>/assessment.md`
8. **Update progress.json** with `assessment_complete: true`

## Examples

```
/kbd-assess                              # uses active waypoint phase
/kbd-assess phase-1-foundation           # explicit phase name
/kbd-assess phase-2-sales-module         # for a new project phase
```

## Hook integration

Source the hooks library and fire `assess:before` immediately after the
"Starting kbd-assess —" Progress Signal, and `assess:after` immediately
before the "Completed kbd-assess —" Progress Signal. The existing
Progress Signals continue to fire — hooks are complementary, not a
replacement.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"

# Starting kbd-assess — <phase>            ← existing Progress Signal
kbd_hooks_fire assess before "$phase" 1 1   # ← new
# … write assessment.md …
kbd_hooks_fire assess after  "$phase" 1 1   # ← new
# Completed kbd-assess — <phase>           ← existing Progress Signal
```

See orchestrator `SKILL.md` → "Hooks" for the full event taxonomy,
override semantics, and `KBD_HOOK_*` payload.

## Stage gate & handoff

Assess is the first stage, so its gate always passes — call it anyway for
uniformity. After writing `assessment.md`, record the handoff that the next
stage (analyze, or plan when analyze is skipped) reads first:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate assess || exit 2
# … write assessment.md …
kbd_stage_handoff_write assess "<1–3 sentences: key gaps found, open questions for analyze/plan>" assessment.md
```

Phases without a `handoffs/` directory are legacy: the gate warns and passes.
A deliberate stage skip is recorded with `kbd_stage_handoff_skip <stage>
"<reason>"`. Schema: `references/schemas/handoff.schema.json`.
