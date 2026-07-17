---
license: MIT
name: kbd-analyze
version: '1.0.0'
description: >
  Run the Analyze stage of the KBD lifecycle: research the engineering
  landscape — existing open-source libraries, frameworks, and skeletons that
  fit the assessed gaps, plus stack discovery when none is specified — and write
  the candidate set that the Spec and Plan stages consume.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-analyze

Run the **Analyze** phase of the KBD lifecycle — between Assess and Spec.

## What this does

Turns the assessment's gaps and unknowns into researched, evidence-backed
build-vs-adopt decisions before any spec is written. This is the engineering
counterpart to the evolver's business-landscape Analyze: KBD Analyze researches
libraries/frameworks/skeletons and writes its findings into
`.kbd-orchestrator/`, never into evolver state.

Outputs to `.kbd-orchestrator/phases/<phase>/`:

- `analysis.md` — narrative: landscape, candidate evaluation, build-vs-adopt
  calls, open questions.
- `library-candidates.json` — machine contract for Spec/Plan
  (schema: `references/schemas/library-candidates.schema.json`).
- `stack-recommendation.md` — only in stack-discovery mode.
- appends to `decision-log.md`.

## The research pipeline

Follow `references/research-pipeline.md` exactly — tiers, budget caps, modes,
and evidence format live there as the single source of truth:

1. **Tier 1** `gh search repos/code` → existing frameworks, skeletons.
2. **Tier 2** Context7 / docfork → confirm API fit + version constraints.
3. **Tier 3** registries (`npm view` / `cargo search` / PyPI) → maintenance health.
4. **Tier 4** firecrawl/tavily → stack comparisons, only when 1–3 insufficient.

Hard budget: `max_queries_per_tier` (8), `max_minutes` (20). On a cap, stop and
report partial findings with lowered confidence — never loop to chase
completeness.

### Modes

- **Stack specified** — rank candidates within the named stack per gap.
- **Stack discovery** — produce 2–3 scored stack options in
  `stack-recommendation.md`, then research candidates against the pick. A
  contested choice (score gap < 15%) escalates via `/pmpo-elicit` when
  available; otherwise flag it for the user in `analysis.md` and the decision
  log — never silently pick a contested stack.

### Contested stack escalation — operative protocol

When the top two stack options are within 15% of each other:

1. Construct the elicitation request:
   - `question`: "Two stacks are equally matched: `<A>` (<scoreA>%) vs `<B>` (<scoreB>%). Which should we use?"
   - `hints`: [`<A> key advantage`, `<B> key advantage`, `primary tradeoff`]
   - `criticality`: high
   - `caller`: kbd-analyze

2. **On Claude Code** — use `AskUserQuestion` with the two stack names as options plus
   "Research further" and "Accept highest-ranked (implicit)". Record the answer in
   `decision-log.md` with `provenance` and `elicitation_id`.

3. **On all other platforms** — call `pmpo-elicit-checkpoint.sh`:
   ```
   bash "${CLAUDE_PLUGIN_ROOT}/skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh" \
     ".kbd-orchestrator/phases/${PHASE}/elicitations/kbd-analyze-$(date +%s)" \
     "Two stacks are equally matched: <A> vs <B>. Which?" \
     "high" "kbd-analyze" \
     "<A> advantage" "<B> advantage"
   ```
   Pause analysis (exit 2 signal). On resume, call `pmpo-elicit-resume.sh` and apply result.

4. **If pmpo-elicit is unavailable** — flag the contest in `analysis.md` under "Open
   Questions", note both options with scores, and ask the user inline before continuing.

Record in `decision-log.md` on resolution:
```
### <timestamp> — Contested stack choice
Options: <A> vs <B> | Score gap: <N>%
Decision: <chosen> | Provenance: <user|research|implicit>
Elicitation ID: <id>
```

## Skipping

For phases that need no external research:

```
/kbd-analyze --skip "<reason>"
```

writes a skip handoff so the Spec gate passes deliberately rather than by drift.

## Progress Signals (MANDATORY)

**FIRST tool call of every turn:** Read `.kbd-orchestrator/position-reminder.txt` (if it exists) to get the current phase, step N of T, and next command. If that file is absent, read `.kbd-orchestrator/current-waypoint.json`.

Before any other action, emit to plain response text (BEFORE any tool call):

```
Starting kbd-analyze — <phase-name> (step N of T)
```

When the candidate set is written (or the stage is skipped), emit:

```
Completed kbd-analyze — <phase-name> (step N of T)
```

**How to get N and T (MANDATORY — never estimate):**
- Read `.kbd-orchestrator/phases/<phase>/progress.json` → `changes_completed` = N, `changes_total` = T
- If `progress.json` is absent, read `current-waypoint.json` → `changes_completed` / `changes_total`

Use the canonical phase name from the argument or `current-waypoint.json`. Emit to plain response text — no tool call needed.

## How to invoke

1. **Confirm the active phase** — argument or `current-waypoint.json`.
2. **Stage gate** — `kbd_stage_gate analyze` (requires the assess handoff).
3. **Read inputs** — `assessment.md`; `prior-context.md` (memory recall) and an
   ideation mindmap id when greenfield.
4. **Evolver bridge** — when `evolver-bridge.json` exists, also invoke
   `/evolve-analyze` and merge its `analysis.json` findings (annotated by source).
5. **Run the tiered pipeline** per `references/research-pipeline.md`.
6. **Write artifacts** — `analysis.md`, `library-candidates.json`,
   `stack-recommendation.md` (discovery mode), `decision-log.md`.
7. **Write handoff** — `kbd_stage_handoff_write analyze "<candidate count, key adopt verdicts, open questions>" analysis.md library-candidates.json`.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate analyze || exit 2
kbd_hooks_fire analyze before "$phase" 1 1
# … run pipeline, write artifacts …
kbd_hooks_fire analyze after  "$phase" 1 1
kbd_stage_handoff_write analyze "<summary>" analysis.md library-candidates.json
```

## Examples

```
/kbd-analyze                             # uses active waypoint phase
/kbd-analyze canonical-lifecycle         # explicit phase
/kbd-analyze --skip "no external deps needed this phase"
```

## Hook integration

Fires `analyze:before` / `analyze:after` (the `analyze` hook kind is in the
allowed enum in `shared/lib/hooks.sh`). See orchestrator `SKILL.md` → "Hooks".
