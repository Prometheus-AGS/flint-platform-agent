## 1. Reuse frf-agentproto + WS dep

- [ ] 1.1 Add a workspace dep on `frf-agentproto` (path: `../flint-realtime-fabric/crates/frf-agentproto`) for `ContentBlock`. Verify it builds as a path dep from this workspace.
- [ ] 1.2 Add `tokio-tungstenite` (verify current version, Base Rule 22) to `fpa-fabric` for the WS client; `futures` for the stream.

## 2. Port method

- [ ] 2.1 `FabricClient::subscribe(&self, spec: SubscriptionSpec, bearer: Option<&str>) -> Result<BoxStream<'static, Result<ContentBlock, PortError>>, PortError>` in `fpa-ports` (async-trait). Define a minimal `SubscriptionSpec` (or reuse fabric's shape) — confirm from `frf-gateway/src/routes/subscribe.rs`.

## 3. WS client adapter (fpa-fabric)

- [ ] 3.1 `subscribe`: build the `ws://{endpoint}/ws/v1/subscribe?<spec>` URL; forward the bearer (header/query per fabric's expectation); `tokio_tungstenite::connect_async`.
- [ ] 3.2 Map each incoming text/binary frame → `serde_json::from_slice::<ContentBlock>` → yield on the `BoxStream`; unknown type → `ContentBlock::Unknown` (no panic); close/error → `PortError`.
- [ ] 3.3 `health()` unchanged. No `unwrap`/`expect` in the adapter.

## 4. Verification

- [ ] 4.1 `cargo check/clippy/fmt` green (frf-agentproto path dep resolves; WS dep builds).
- [ ] 4.2 Unit: a fake WS server (or a serde round-trip of `ContentBlock` frames) exercises deserialize + `Unknown` + error mapping (no live fabric needed for the unit test).
- [ ] 4.3 The live write→CDC→receive assertion lands in the smoke (p10-c005), against real fabric.
