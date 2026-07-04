## 1. Reuse frf-domain + WS dep

- [x] 1.1 Add a workspace dep on `frf-domain` (path: `../flint-realtime-fabric/crates/frf-domain`) for `EventEnvelope` / `EventKind` / `Channel`. It is lightweight (serde/serde_json/uuid/chrono — no prost/tonic). Verify it builds as a path dep from this workspace. (NOT `frf-agentproto` — that's prost/tonic-heavy and is the `/ws/v1/agents` type, not subscribe.)
- [x] 1.2 Add `tokio-tungstenite` (verify current version, Base Rule 22) to `fpa-fabric` for the WS client; `futures` for the stream.

## 2. Port method

- [x] 2.1 `FabricClient::subscribe(&self, channel: Uuid, bearer: Option<&str>) -> Result<BoxStream<'static, Result<EventEnvelope, PortError>>, PortError>` in `fpa-ports` (async-trait). `channel` is a UUID (fabric requires it); bearer forwarded as `Authorization: Bearer`.
- [x] 2.2 `fpa-ports` gains a dep on `frf-domain` (re-export `EventEnvelope` from the port module so adapters + app share one type). Add `uuid` + `futures` to `fpa-ports`.

## 3. WS client adapter (fpa-fabric)

- [x] 3.1 `subscribe`: build `ws://{endpoint}/ws/v1/subscribe?channel=<uuid>` (ws/wss from the endpoint scheme); set `Authorization: Bearer <token>` on the handshake request; `tokio_tungstenite::connect_async`.
- [x] 3.2 Map each incoming `Message::Text` → `serde_json::from_str::<EventEnvelope>` → yield on the `BoxStream`; deserialize error/close/protocol error → `PortError` (Transport/Decode). No panic. Non-text frames ignored/closed gracefully.
- [x] 3.3 `health()` unchanged. No `unwrap`/`expect` in the adapter.

## 4. Verification

- [x] 4.1 `cargo check/clippy/fmt` green (frf-domain path dep resolves; tokio-tungstenite builds).
- [x] 4.2 Unit: a fake WS server (tokio-tungstenite server side) sends an `EventEnvelope` JSON frame → assert the client yields the deserialized envelope; plus a missing-bearer/unreachable → `PortError` case. (No live fabric needed for the unit test.)
- [x] 4.3 The live forge-write→CDC→receive assertion lands in the smoke (p10-c005), against real fabric.
