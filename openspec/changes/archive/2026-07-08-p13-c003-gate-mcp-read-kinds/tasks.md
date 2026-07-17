## 1. Catalog entries

- [ ] 1.1 Add `gate.route.list` to `CATALOG` (`catalog.rs`): `target: TargetPort::Gate`,
      `required_role: "operator"`, `input_schema_json: SCHEMA_EMPTY`, with a description.
- [ ] 1.2 Add `mcp.tool.list` to `CATALOG`: `target: TargetPort::Mcp`,
      `required_role: "viewer"`, `input_schema_json: SCHEMA_EMPTY`, with a description.

## 2. Runner dispatch + classification

- [ ] 2.1 Confirm `is_gate_write_kind` returns `false` for `gate.route.list` (only
      `application.deploy` is a write) so `dispatch_gate` returns `self.gate.list_routes()`.
- [ ] 2.2 Confirm `dispatch` routes `TargetPort::Mcp` (kind `mcp.tool.list`) to the existing
      `self.mcp.list_tools()` arm; wrap the result in the standard response envelope.
- [ ] 2.3 Register `gate.route.list` in `GATE_READ_KINDS` so
      `every_gate_catalog_kind_is_classified` passes.

## 3. Guards + tests

- [ ] 3.1 Add `every_mcp_catalog_kind_is_dispatched` (`MCP_KINDS = &["mcp.tool.list"]`)
      mirroring the Gate/Store guards.
- [ ] 3.2 Unit: `gate.route.list` runs as operator → `FakeGate.list_routes` called, no write
      refusal; a non-operator role is rejected by the permission check.
- [ ] 3.3 Unit: `mcp.tool.list` runs as viewer → `FakeMcp.list_tools` called; result
      surfaces the tool list.

## 4. Verification

- [ ] 4.1 The 3 catalog tests (`lookup_known_and_unknown`, `kinds_are_unique`,
      `every_entry_has_valid_input_schema`) pass with the two new entries.
- [ ] 4.2 Both guard tests pass (Gate with updated `GATE_READ_KINDS`, new Mcp guard).
- [ ] 4.3 `cargo test -p fpa-app` green — the phase's final integration milestone
      (batch c001+c002+c003 assertions into this run; ≤3 total for the phase).
- [ ] 4.4 `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` clean.
- [ ] 4.5 Confirm writes remain deferred: `mcp.call_tool` / `forge.create_entity` absent
      from the catalog; `application.deploy` still refused.
