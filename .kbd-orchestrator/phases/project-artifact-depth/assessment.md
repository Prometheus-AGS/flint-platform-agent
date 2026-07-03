# Assessment — project-artifact-depth

**Phase:** project-artifact-depth
**Date:** 2026-07-03
**Stage:** assess (grounded in the domain + runner code the goals touch)

> Good news up front: the nested domain types all exist and derive serde
> `Serialize`/`Deserialize` (+ `JsonSchema` under the `schema` feature), so the
> richer-input work is mostly **wiring existing types through validation +
> persistence**, not new modelling. One small design decision (input shape) and one
> routing-clarity call are the only real questions.

---

## 1. Goals recap (from goals.md)

1. Richer `project.create` — accept + validate the nested aggregate, persist whole.
2. `application.define` persistence home — mutate the Project via `ProjectStore`.
3. Archive the p1–p4 OpenSpec backlog.

---

## 2. Codebase state per goal (evidence)

### Goal 1 — richer `project.create` (small, serde does the work)
- `create_project` (`task_runner.rs`) today reads only `name` + optional
  `project_id` and calls `Project::new` (minimal aggregate).
- **`Project` deserializes cleanly:** `id`/`schema_version`/`name` required, and
  `applications`/`sub_agents`/`schemas`/`realtime`/`entity_meta` are all
  `#[serde(default)]` (`project/mod.rs`). Every nested type
  (`ApplicationDef`, `SubAgentDef`, `SchemaDef`, `ComponentRef`, …) derives
  `Deserialize`.
- **Therefore serde IS the structural validator** — `serde_json::from_value::<Project>`
  rejects wrong types/shapes with a precise error. No hand-authored JSON Schema for
  every nested field is needed (DRY; avoids drift from the `#[non_exhaustive]` types).
- **Input-shape mismatch to resolve (design):** the current task input is
  `{ name, project_id }`; a full `Project` is `{ id, schema_version, name,
  applications, … }`. Options: (a) accept a `Project`-shaped body directly
  (deserialize the whole thing, generate `id`/`schema_version` if omitted); or
  (b) keep the friendly `{ name, project_id, applications?, sub_agents?, … }` input
  and map it onto `Project` (server owns `id`/`schema_version`). **(b) is better** —
  the client shouldn't set `schema_version`, and `id` is server-assigned unless
  `project_id` is given. Catalog schema stays a light guard (`required: name`); the
  deep validation is the serde map into the typed nested `Vec`s.

### Goal 2 — `application.define` home (new store-backed handler)
- Catalogued `TargetPort::Forge`, `SCHEMA_EMPTY`, `required_role: operator`. It
  currently falls to the `dispatch_forge` `other =>` guard → `Downstream("write API
  pending")` (the "forge-backed application row" comment at `task_runner.rs:156` is
  **stale** — there is no such row).
- **Real home:** an application belongs to a project's `applications` list. So
  `application.define` should: require `project_id` + an `ApplicationDef` (or the
  fields to build one); **load the project from `ProjectStore`**, upsert the
  `ApplicationDef` by its id, `put` the project back. Agent-owned aggregate mutation
  — no forge. Unknown `project_id` ⇒ clean `Downstream`/`NotFound`-style error.
- **Routing-clarity note:** `project.create` is `TargetPort::Forge` but intercepted
  before the forge call (store-backed). `application.define` will be the same. This
  works but muddies "Forge = forge". **Optional cleanup:** a `TargetPort::Store`
  variant (or route both `project.*`/`application.define` store-kinds before the
  `TargetPort` match) would read cleaner. Not required for correctness — flag for
  the plan.

### Goal 3 — archive p1–p4 (mechanical, with 2 partials)
`openspec list` shows p1–p4: **12 `✓ Complete`**, **2 partial** —
`p2-c002-forge-read-integration` (12/13) and `p1-c004-mcp-transport` (11/12).
- Archive the 12 complete ones now.
- The 2 partials: either mark the trailing task done (if it truly is — likely a
  verification checkbox) after a quick look, or **skip + note** them for a later
  reconcile. `openspec archive` refuses/《warns》on incomplete changes, so they can't
  be force-archived cleanly without finishing the task line.

---

## 3. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | `project.create` accepts only `name` — no nested input | Small (serde-mapped) | No |
| G2 | Input shape: friendly-map vs raw-`Project` | **Design** (lean: friendly-map) | No |
| G3 | `application.define` has no home (falls to guard) | Small-med (new store handler) | No |
| G4 | Routing clarity: store-kinds under `TargetPort::Forge` | Optional cleanup | No |
| G5 | p1–p4 archival: 12 complete + 2 partial | Small; partials need a look | No |

---

## 4. Open questions (for analyze/plan)

1. **`project.create` input shape** — friendly `{name, project_id, applications?, …}`
   mapped to `Project` (server owns id/schema_version), vs accept a raw `Project`
   body. **Lean: friendly-map.** (Minor — decidable at spec without the operator.)
2. **`application.define` semantics** — require the project to exist (reject if
   absent) and **upsert** the `ApplicationDef` by id. Confirm add-only vs upsert.
   **Lean: require-exists + upsert.**
3. **Routing** — introduce `TargetPort::Store` for agent-owned aggregate writes, or
   keep intercepting `Forge`-catalogued kinds? **Lean: small `TargetPort::Store`** for
   clarity (both `project.create` + `application.define` move to it).
4. **p1–p4 partials** — finish the trailing task on `p2-c002`/`p1-c004` and archive,
   or skip + note? **Lean: inspect; finish if trivial, else skip + note.**

None of these need an operator decision — all are decidable at spec with sensible
leans. No external research needed (all in-house types + serde).

---

## 5. Handoff to analyze/plan

All three goals are **in-house Rust over types that already exist** — no new deps, no
infra, no operator gate. Richer `project.create` is a serde map of the nested
aggregate (serde = the validator; catalog schema stays a light guard).
`application.define` becomes a store-backed aggregate mutation (load project → upsert
`ApplicationDef` → put), mirroring `project.create`. A small `TargetPort::Store`
variant would clean up routing for both (optional). p1–p4 archival is mechanical for
the 12 complete changes; 2 partials (`p2-c002`, `p1-c004`) need a trailing-task look
or a skip-note. Analyze can likely **skip external research** and go straight to spec.
