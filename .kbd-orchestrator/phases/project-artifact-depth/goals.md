# Goals — project-artifact-depth

> Seeded from `live-smoke-and-durable-store/reflection.md` → "Recommended Next Phase"
> (Option B, no-infra). The durable store shipped but is Docker-gated; this phase adds
> **product depth that needs no external infra** — richer Project artifacts and the
> `application.define` persistence home — plus the p1–p4 archival backlog.
>
> The domain types already exist (`fpa-domain::project::{ApplicationDef, SubAgentDef,
> SchemaDef, ComponentRef}`); this phase is about **accepting, validating, and
> persisting** them end-to-end, not defining new ones.

## Primary goals

1. **Richer `project.create` input.** Today `project.create` accepts only `name`
   (+ optional `project_id`) and builds a minimal `Project`. Extend it to accept and
   **validate** the nested aggregate — `applications`, `sub_agents`, `schemas`,
   `realtime`, `entity_meta` — against a proper JSON Schema in the catalog, then
   persist the full aggregate via the existing `ProjectStore`. Invalid nested input
   must be rejected before any store write (consistent with the existing
   validate-before-dispatch discipline).

2. **`application.define` persistence home.** `application.define` currently falls to
   the forge "write API pending" guard. Give it a real home: an application is a
   member of a project's `applications` list, so `application.define` should **load
   the target project from `ProjectStore`, add/replace the `ApplicationDef`, and
   persist** — an agent-owned mutation of the Project aggregate, not a forge write.
   Requires a `project_id` in the input.

3. **Archive the p1–p4 OpenSpec changes.** Clear the archival backlog: the completed
   p1–p4 changes still sit in the active `changes/` list. `openspec archive` the
   `✓ Complete` ones into `specs/` (skip any still showing partial task counts, or
   finish/annotate them first).

## Success criteria

- `project.create` with a full nested payload validates + stores the whole aggregate;
  a malformed nested payload is rejected (unit-tested), with no partial write.
- `application.define` adds an `ApplicationDef` to an existing project and persists it;
  `get` returns the project with the new application; an unknown `project_id` is a
  clean error (unit-tested). Round-trips through the in-memory store (the durable path
  is the same port).
- The p1–p4 `✓ Complete` changes are archived; `openspec list` no longer shows them as
  active; `openspec/specs/` gains their capabilities.

## Open questions (for /kbd-assess → /kbd-analyze)

- **`application.define` semantics:** add-only, or upsert-by-application-id? And does
  it require the project to already exist (reject if absent) or create-if-missing?
  (Lean: require the project to exist; upsert the application by its id.)
- **JSON Schema depth:** how strictly to validate the nested `Project` input — full
  structural schema for every nested type, or a pragmatic subset now? (The domain
  types are `#[non_exhaustive]` / additive; the schema should not over-constrain.)
- **p1–p4 archival:** a couple of earlier changes show partial task counts
  (`p2-c002` 12/13, `p1-c004` 11/12). Archive only the `✓ Complete` ones, or
  reconcile the partials first? (Lean: archive complete ones; note the partials.)

## Explicitly out of scope this phase (still deferred)

Live 3-plane smoke (infra-gated — its own future phase); durable-store runtime proof
(`--ignored` container test, needs Docker); Postgres TLS (rustls connector); MCP
multi-server; fabric WS subscriptions; OpenDesign; A2UI/React UI; Tauri;
knowledge-base.
