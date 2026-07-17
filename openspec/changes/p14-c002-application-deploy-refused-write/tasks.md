## 1. Refusal message

- [ ] 1.1 In `dispatch_gate` (`task_runner/mod.rs`), tighten the `application.deploy`
      write-refusal `Downstream` message to name the missing contract:
      `"gate route-write not implemented: 'application.deploy' — no verified flint-gate admin write endpoint (GateAdmin exposes list_routes only)"`.
      Do NOT change behavior: it still returns `Err(PortError::Downstream(..))`, never `Ok`.

## 2. Dependency documentation

- [ ] 2.1 Record the blocker as a project memory: `GateAdmin` exposes `list_routes` only;
      `GateAdapter` does `GET /routes` only; no gate admin **write** endpoint is verified
      against gate source; a real `application.deploy` needs the six steps enumerated in the
      spec. Link it from `MEMORY.md`.
- [ ] 2.2 Ensure the spec delta (`specs/task-catalog/spec.md`) enumerates the six steps a
      real gate write requires, so the next owner does not re-derive them.

## 3. Verification

- [ ] 3.1 Unit: `application.deploy` submitted as `admin` still **refuses** — returns a
      `Downstream`/error outcome, never a success result. (Assert on refusal, not success.)
- [ ] 3.2 Unit: the refusal message contains the marker naming the missing contract
      (e.g. `"list_routes only"`), so the honest-refusal wording is regression-guarded.
- [ ] 3.3 Rolls into the phase's single `cargo test -p fpa-app` milestone shared with
      p14-c001 (do NOT spend a separate test run on this docs/message change).
- [ ] 3.4 `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` clean.
- [ ] 3.5 Confirm no fake write: `application.deploy` never returns `Ok`; no `GateAdmin`
      write method added; no `GateAdapter` write path added; no sibling repo edited.
