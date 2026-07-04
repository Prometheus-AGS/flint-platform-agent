# Assessment — project-read-and-list

**Phase:** project-read-and-list
**Date:** 2026-07-04
**Stage:** assess (grounded in the dispatch + store code the goals touch)

> Clean phase: no infra, no operator gate, all in-house over the store the last three
> phases built. The core is a retarget (`project.inspect`/`project.list` → `Store`)
> plus one new port method (`ProjectStore::list()`). Fully unit-testable in-memory.

---

## 1. Goals recap (from goals.md)

1. `project.inspect` → `ProjectStore.get` (retarget from Forge).
2. `project.list` → new `ProjectStore::list()` (retarget from Forge; new port method + both adapters).
3. Fold in the trivial p6 archival.

---

## 2. Codebase state per goal (evidence)

### Goal 1 — `project.inspect` → store (small retarget)
- Today: catalogued `TargetPort::Forge`; `dispatch_forge` routes
  `"forge.table.describe" | "project.inspect"` → `forge.describe_table("<unspecified>")`
  — i.e. it queries **forge** (which has no projects table) and ignores the
  `project_id`. Wrong home.
- Fix: retarget the catalog entry to `TargetPort::Store`; **remove** `project.inspect`
  from the `dispatch_forge` shared arm (leaving `forge.table.describe` alone); add a
  `"project.inspect"` arm to `dispatch_store` that parses `project_id` and calls
  `store.get` → whole aggregate; `None` → clean `Downstream("unknown project …")`.
  Catalog schema already requires `project_id` (`SCHEMA_PROJECT_ID`).

### Goal 2 — `project.list` → new port method (small-med)
- Today: `TargetPort::Forge`; `dispatch_forge` routes
  `"forge.table.list" | "project.list"` → `forge.list_tables` — queries forge, wrong.
- **New port method:** `ProjectStore::list() -> Result<Vec<Project>, PortError>` on the
  `fpa-ports` trait.
  - `InMemoryProjectStore` (`RwLock<HashMap<ProjectId, Project>>`): `list` =
    `read().await.values().cloned().collect()` — trivial.
  - `PgProjectStore`: `list` mirrors `get` with a full-table read —
    `SELECT body FROM fpa_projects` via `client.query(...)`, map each row's `body`
    through `serde_json::from_value` (same JSONB deserialization as `get`).
- Retarget `project.list` to `Store`; remove it from the `dispatch_forge` arm; add a
  `"project.list"` arm to `dispatch_store` → `store.list()` → `{"projects":[…]}`.
- **Ordering:** `HashMap` iteration is unordered; if a stable order is wanted, sort by
  id (or leave unordered for now). Pg `SELECT` is also unordered without `ORDER BY`.
  (Lean per goals: full list now, order/pagination later.)

### Goal 3 — fold in p6 archival (mechanical)
`openspec list` confirms `p6-c001-durable-project-store` and `p6-c002-archive-p5-changes`
are `✓ Complete`. `openspec archive` both into `specs/`. The 2 known partials
(`p2-c002` 12/13, `p1-c004` 11/12) stay active (still deferred — same as p7-c003).

---

## 3. Gap summary

| # | Gap | Size | Blocker? |
|---|---|---|---|
| G1 | `project.inspect` queries forge, not the store | Small (retarget + store arm) | No |
| G2 | `project.list` queries forge; no `ProjectStore::list()` exists | Small-med (new port method + 2 adapters) | No |
| G3 | Pg `list` runtime-unproven (Docker down) | Carried | No — in-mem `list` is the runnable proof |
| G4 | p6 archival backlog | Trivial | No |

---

## 4. Open questions (for analyze/plan — all with leans)

1. **Pagination/filtering on `project.list`?** → **full list now** (YAGNI; the store
   is small; paginate when a real dataset demands it). Decidable at spec.
2. **`project.inspect` shape** → **whole aggregate** (the read of what `create` wrote).
3. **List ordering** → unordered is acceptable now; if a test wants determinism, sort
   by id in the handler (cheap). Note for spec.
4. **Per-operator visibility/RLS** → the store has no ownership column; return **all**
   this phase (single-tenant), note per-operator scoping as future debt (gate/forge
   remain the authz authority — consistent with the cross-plane contracts).
5. **Pg `list` test** → the container test is `#[ignore]`d (Docker down); the in-mem
   `list` is the runnable proof this session. Add a Pg `list` assertion inside the
   existing `#[ignore]`d test.

None need external research or an operator decision — all decidable at spec.

---

## 5. Handoff to analyze/plan

All in-house over the existing `ProjectStore`; no deps, no infra, no operator gate.
Two retargets (`project.inspect`/`project.list` from `Forge` → `Store`, removing them
from `dispatch_forge`, adding `dispatch_store` arms) + one new port method
(`ProjectStore::list()`, implemented in both adapters — in-mem trivial, Pg a
full-table `SELECT body` mirroring `get`). p6 archival is mechanical. Analyze can
**skip** (no external research). Leans: full list now, whole-aggregate inspect,
single-tenant list + note RLS debt, Pg `list` proven via the `#[ignore]`d container
test.
