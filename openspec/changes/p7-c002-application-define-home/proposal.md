## Why

`application.define` has no real home: it is catalogued `TargetPort::Forge` and falls
to the `dispatch_forge` guard → `Downstream("write API pending")` (the "forge-backed
application row" comment is stale — no such row exists). An application is a member of
a project's `applications` list, so its natural home is an **agent-owned mutation of
the Project aggregate**: load the project, upsert the `ApplicationDef`, persist.

## What Changes

- Retarget `application.define` to `TargetPort::Store` (the variant added in p7-c001).
- Implement `define_application` in the store dispatch arm: require `project_id` +ï¸
  an `ApplicationDef` (or the fields to build one); **load** the project from
  `ProjectStore`; if absent → clean error (unknown project); **upsert** the
  `ApplicationDef` by its id into `project.applications` (replace same-id, else push);
  `put` the mutated project back; return it.
- Catalog schema for `application.define`: `required: ["project_id", "application"]`
  (or `["project_id","name"]` if building a minimal `ApplicationDef` server-side —
  choose in tasks); `required_role` stays `operator`.

## Capabilities

### New Capabilities
- `application-define-home`: `application.define` persists an `ApplicationDef` into its project's aggregate via `ProjectStore` (load → upsert-by-id → put), rejecting an unknown project.

### Modified Capabilities

## Impact

- `fpa-app`: `catalog.rs` (retarget + schema), `task_runner.rs` (`define_application`
  in the store arm). Depends on p7-c001's `TargetPort::Store`.
- No new deps; no domain change (`ApplicationDef` exists).

## Open Questions
- **RESOLVED (assess lean):** require the project to exist (reject unknown
  `project_id`); **upsert** the `ApplicationDef` by id (replace same-id, else append).
