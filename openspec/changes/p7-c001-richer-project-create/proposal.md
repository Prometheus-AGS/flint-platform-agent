## Why

`project.create` today accepts only `name` (+ optional `project_id`) and builds a
minimal `Project` — the nested aggregate (`applications`, `sub_agents`, `schemas`,
`realtime`, `entity_meta`) can't be supplied. All those nested types already derive
`Deserialize`, so **serde is the natural validator**: a full-shape input can be
mapped onto `Project` with precise structural rejection, no hand-authored JSON Schema
per nested field. This change accepts the richer input and, while here, cleans up the
routing so agent-owned aggregate writes don't masquerade as `TargetPort::Forge`.

## What Changes

- **Richer `create_project`:** accept a friendly-map input
  `{ name, project_id?, applications?, sub_agents?, schemas?, realtime?, entity_meta? }`.
  The server owns `id` (from `project_id` or a fresh v4) and `schema_version` (always
  the current `SCHEMA_VERSION` — the client never sets it). The nested collections are
  deserialized into the typed `Vec<…>`/params via serde; a malformed nested payload is
  rejected with a clear `InvalidInput`/`Downstream` error **before any store write**.
- **`TargetPort::Store` routing (G4):** add a `Store` variant to `TargetPort` for
  agent-owned aggregate writes; catalogue `project.create` as `TargetPort::Store` and
  route it through a `dispatch_store` arm — so store-backed kinds no longer ride under
  `TargetPort::Forge` (removes the "intercept before the forge call" muddiness).
- **Catalog schema** for `project.create` stays a light guard (`required: name`;
  `project_id`/nested arrays optional) — the deep validation is the serde map.

## Capabilities

### New Capabilities
- `richer-project-create`: `project.create` accepts and validates the full nested `Project` aggregate (serde-typed), persisting it whole via `ProjectStore`, routed through a dedicated `TargetPort::Store`.

### Modified Capabilities

## Impact

- `fpa-app`: `catalog.rs` (`TargetPort::Store` + `project.create` retarget + schema),
  `task_runner.rs` (`dispatch_store` arm; `create_project` maps the nested input).
- No new deps. No change to `fpa-ports`/`fpa-domain` (types already exist). No change
  to the gateway surfaces (same task kind, richer input).

## Open Questions
- **RESOLVED (assess lean):** friendly-map input (server owns id/schema_version);
  serde validates nested; `TargetPort::Store` for routing clarity.
