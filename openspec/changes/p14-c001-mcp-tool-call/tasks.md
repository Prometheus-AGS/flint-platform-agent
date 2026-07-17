## 1. Catalog entry + schema

- [ ] 1.1 Add `SCHEMA_MCP_TOOL_CALL` const to `catalog.rs`: a real (non-empty) schema —
      `{"type":"object","required":["name","arguments"],"properties":{"name":{"type":"string"},"arguments":{"type":"object"}}}`.
- [ ] 1.2 Add `mcp.tool.call` to `CATALOG`: `target: TargetPort::Mcp`,
      `required_role: "operator"`, `input_schema_json: SCHEMA_MCP_TOOL_CALL`, with a
      description and an inline comment recording the operator-floor rationale (invoke, not
      discovery) mirroring the `mcp.tool.list` / `gate.route.list` role comments.

## 2. Runner dispatch split

- [ ] 2.1 Replace the flat `TargetPort::Mcp => self.mcp.list_tools().await` arm
      (`task_runner/mod.rs:127`) with `TargetPort::Mcp => self.dispatch_mcp(entry.kind, &input).await`.
- [ ] 2.2 Add `async fn dispatch_mcp(&self, kind: &str, input: &serde_json::Value) -> Result<serde_json::Value, fpa_ports::PortError>`
      mirroring `dispatch_gate`: `"mcp.tool.list" => self.mcp.list_tools().await`;
      `"mcp.tool.call" => { let (name, arguments) = parse_tool_call(input)?; self.mcp.call_tool(&name, arguments).await }`;
      unknown kind → `Err(PortError::Downstream(...))` naming the kind (no `list_tools`
      fallback).
- [ ] 2.3 Add a `parse_tool_call(input) -> Result<(String, serde_json::Value), PortError>`
      helper alongside `parse_project_id` / `parse_table_name`: extract the `name` string and
      the `arguments` object from the already-schema-validated input. Return
      `PortError::Decode`-style clean errors if malformed (schema already guarantees shape;
      this is defense-in-depth, no `unwrap`/`expect`).

## 3. Audit + idempotency

- [ ] 3.1 Confirm the `run()` allow/complete `tracing` records log `operator`, `kind`,
      `decision`/`outcome`, and `signature_verified` — and **not** the input/arguments. (No
      code change expected; if any arm logs the payload, remove it.)
- [ ] 3.2 Document `mcp.tool.call` as non-idempotent-by-default in the `dispatch_mcp` /
      catalog doc-comment (Base Rule 35): idempotency is the downstream tool's property; the
      audit record is the safety net; no dedup machinery added.

## 4. Guards + tests

- [ ] 4.1 Extend `every_mcp_catalog_kind_is_dispatched`:
      `MCP_KINDS = &["mcp.tool.list", "mcp.tool.call"]`.
- [ ] 4.2 Unit: `mcp.tool.call` runs as operator → `FakeMcp.call_tool` receives the exact
      `name` **and** `arguments` unaltered; the returned value surfaces in the response.
- [ ] 4.3 Unit: a role below operator (e.g. viewer) is rejected by the permission check
      **before** any port call — `FakeMcp.call_tool` is NOT called (Base Rule 33).
- [ ] 4.4 Unit: an unknown Mcp kind routed through `dispatch_mcp` returns a clean
      `Downstream` error and does **not** fall back to `list_tools`.
- [ ] 4.5 Unit: invoking `mcp.tool.call` with a `tracing` capture asserts the `arguments`
      payload does not appear in any emitted log record.

## 5. Verification

- [ ] 5.1 The 3 catalog invariant tests (`lookup_known_and_unknown`, `kinds_are_unique`,
      `every_entry_has_valid_input_schema`) pass with the new entry + real schema.
- [ ] 5.2 The extended Mcp dispatch guard passes.
- [ ] 5.3 `cargo test -p fpa-app` green — the phase's final integration milestone (batch
      c001 + c002 assertions into this one run; ≤3 `cargo test` runs total for the phase).
- [ ] 5.4 `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt --all` clean.
- [ ] 5.5 Confirm scope kept: `mcp.tool.call` present; a **real** `application.deploy` write
      still absent (still refused — see p14-c002); `forge.create_entity` absent; no new port
      added.
