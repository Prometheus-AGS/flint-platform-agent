# Analysis — project-artifact-depth (SKIPPED)

**Phase:** project-artifact-depth
**Date:** 2026-07-03
**Status:** **SKIPPED** — `/kbd-analyze --skip "in-house serde over existing domain
types; no external deps or research"`

---

## Why skipped

All three goals are **in-house Rust over types that already exist**, satisfiable with
serde + the existing `ProjectStore` port. No library, framework, or skeleton is a
candidate, so the tiered research pipeline (Tier 1–4) would return nothing and waste
the query budget:

- **Richer `project.create`** — `serde_json::from_value` over the already-`Deserialize`
  nested `Project` types *is* the validation. No new crate.
- **`application.define` home** — a store-backed aggregate mutation over the existing
  `ProjectStore` port. No new crate.
- **p1–p4 archival** — the `openspec` CLI (already in use). No new crate.

Assess resolved every design question with a sensible lean (below); each is
decidable at spec, none needs external evidence or an operator gate.

## Design leans carried to spec (from assess §4)

1. **`project.create` input shape** → **friendly-map** `{name, project_id?,
   applications?, sub_agents?, schemas?, realtime?, entity_meta?}` mapped onto
   `Project` (server owns `id`/`schema_version`); serde validates the nested `Vec`s.
2. **`application.define`** → **require the project to exist** (reject unknown
   `project_id`) and **upsert** the `ApplicationDef` by its id.
3. **Routing** → introduce a small **`TargetPort::Store`** variant so
   `project.create` + `application.define` route cleanly (not intercepted under
   `TargetPort::Forge`).
4. **p1–p4 partials** → inspect `p2-c002` (12/13) and `p1-c004` (11/12); finish the
   trailing task if trivial, else **skip + note** — archive the 12 complete ones now.

## Candidates

None. `library-candidates.json` records zero candidates and the four gaps as
`build_required` (in-house).

## Handoff to spec

Analyze skipped (justified — no external deps). Spec directly from the assessment +
the four leans above: richer `project.create` (serde map), `application.define`
store-backed upsert, optional `TargetPort::Store` routing cleanup, and p1–p4
archival (12 now, 2 partials inspected).
