## 1. Retarget + schema

- [ ] 1.1 Retarget `application.define` catalog entry to `TargetPort::Store` (variant from p7-c001).
- [ ] 1.2 Catalog schema: `required: ["project_id","application"]` where `application` is an `ApplicationDef`-shaped object (or `["project_id","name"]` if building a minimal `ApplicationDef` server-side — pick and document). `required_role` stays `operator`.

## 2. define_application handler

- [ ] 2.1 In the store dispatch arm, add `"application.define" => self.define_application(&input).await`.
- [ ] 2.2 Parse `project_id` (uuid) + the `ApplicationDef` (serde) from input; map errors → InvalidInput/Downstream.
- [ ] 2.3 `store.get(project_id)`: `None` → `Downstream("unknown project '<id>'")`, store nothing.
- [ ] 2.4 Upsert: replace the app in `project.applications` with the same `id`, else push. Immutable-style (build a new `applications` vec / new `Project`), consistent with the coding rules.
- [ ] 2.5 `store.put(mutated)`; return the mutated project.

## 3. Verification

- [ ] 3.1 `cargo check/clippy/fmt` green (batched).
- [ ] 3.2 Unit: define into an existing project → get returns it in `applications`.
- [ ] 3.3 Unit: define twice same app id → single entry, second definition wins.
- [ ] 3.4 Unit: unknown `project_id` → Downstream/not-found, store unchanged.
