## Why

Operator directive: real fabric is for **events/realtime**, and the smoke must prove it
end-to-end (write → CDC → agent receives the change). Today the agent's fabric surface
is **only `GET /healthz`** — its code notes realtime subscriptions over
`/ws/v1/subscribe` are "a later phase," and `FabricClient` has only `health()`. To
drive real events the agent needs a **realtime subscription client**. This is the
biggest (and only new-agent-code) change in the phase.

## What Changes

- **Reuse `frf-agentproto` (do NOT reinvent — Base Rule 12 / "Shared Agent-Protocol
  Contract"):** add a dependency on `../flint-realtime-fabric/crates/frf-agentproto`
  for its `ContentBlock` (`#[serde(tag="type")]`, `TextDelta`/`StateSnapshot`/
  `ChangeEvent`/… `#[serde(other)] Unknown`).
- **New port method:** `FabricClient::subscribe(spec) -> BoxStream<ContentBlock>` in
  `fpa-ports` (async-trait; a stream of agent-proto content blocks).
- **WS client adapter** in `fpa-fabric`: connect to `{fabric}/ws/v1/subscribe` (the
  fabric gateway's WebSocket), forward the auth bearer, deserialize each frame into
  `ContentBlock`, surface as a `BoxStream`. Maps transport/close to `PortError`.
- The `health()` path is unchanged. No gateway-surface change on the agent (this is a
  client the agent *uses*, not a new inbound route) — though a later phase may bridge
  fabric change-events into the agent's AG-UI stream.

## Capabilities

### New Capabilities
- `fabric-realtime-client`: The agent can subscribe to fabric's real CDC change stream (`/ws/v1/subscribe`) and receive `frf-agentproto::ContentBlock` events — enabling the write→CDC→agent-receives-event smoke.

## Impact

- `fpa-ports` (new `subscribe` on `FabricClient`), `fpa-fabric` (WS client adapter +
  `frf-agentproto` dep + a WS dep — `tokio-tungstenite`, verify current version).
  `fpa-gateway` composition unaffected (same adapter, new method). New deps: verify
  `tokio-tungstenite` version (Base Rule 22) + the `frf-agentproto` path dep.

## Open Questions
- **`/ws/v1/subscribe` request shape** — the `SubscriptionSpec` (what to subscribe to:
  table/tenant/filter) and auth (bearer in a header or query) — confirm from
  `frf-gateway/src/routes/subscribe.rs` at execute.
- **`frf-agentproto` as a path dep across repos** — fine for a local smoke; note it's
  not portable (a published crate would be better long-term). Confirm acceptable.
- **Trigger for the smoke event:** fabric ships a `dev` route "to exercise the full
  subscribe fan-out" — use it, or drive a real forge DB write that CDC picks up.
  Decide at spec-of-smoke (p10-c005).
