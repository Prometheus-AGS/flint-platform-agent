## Why

Operator directive: real fabric is for **events/realtime**, and the smoke must prove it
end-to-end (write → CDC → agent receives the change). Today the agent's fabric surface
is **only `GET /healthz`** — its code notes realtime subscriptions over
`/ws/v1/subscribe` are "a later phase," and `FabricClient` has only `health()`. To
drive real events the agent needs a **realtime subscription client**. This is the
biggest (and only new-agent-code) change in the phase.

## What Changes

- **Reuse `frf-domain::EventEnvelope` (do NOT reinvent — Base Rule 12 / "Shared
  Agent-Protocol Contract"):** add a dependency on
  `../flint-realtime-fabric/crates/frf-domain` for `EventEnvelope` / `EventKind` /
  `Channel`. This is the **real wire type** of `/ws/v1/subscribe` — verified against
  `frf-gateway/src/routes/subscribe.rs`, which serializes `EventEnvelope` JSON frames.
  (An earlier draft named `frf-agentproto::ContentBlock`; that is the type for the
  *separate* `/ws/v1/agents` bus, and its crate drags in prost/tonic. Corrected per
  Base Rule 10; see the c004 spec's grounding note.)
- **New port method:** `FabricClient::subscribe(channel, bearer) ->
  BoxStream<EventEnvelope>` in `fpa-ports` (async-trait; a stream of fabric envelopes).
- **WS client adapter** in `fpa-fabric`: connect to
  `ws://{endpoint}/ws/v1/subscribe?channel=<UUID>` with `Authorization: Bearer`,
  deserialize each text frame into `EventEnvelope`, surface as a `BoxStream`. Maps
  transport/close/decode to `PortError`.
- The `health()` path is unchanged. No gateway-surface change on the agent (this is a
  client the agent *uses*, not a new inbound route) — though a later phase may bridge
  fabric change-events into the agent's AG-UI stream.

## Capabilities

### New Capabilities
- `fabric-realtime-client`: The agent can subscribe to fabric's real CDC change stream (`/ws/v1/subscribe`) and receive `frf-domain::EventEnvelope` events — enabling the write→CDC→agent-receives-event smoke.

## Impact

- `fpa-ports` (new `subscribe` on `FabricClient` + `frf-domain` re-export + `uuid`/
  `futures` deps), `fpa-fabric` (WS client adapter + `frf-domain` dep + a WS dep —
  `tokio-tungstenite`, verify current version). `fpa-gateway` composition unaffected
  (same adapter, new method). New deps: verify `tokio-tungstenite` version (Base Rule 22)
  + the `frf-domain` path dep.

## Open Questions
- **`frf-domain` as a path dep across repos** — fine for a local smoke; note it's not
  portable (a published crate would be better long-term). Confirmed acceptable by the
  operator at execute.
- **Channel UUID for the smoke:** which channel the smoke subscribes to (the fabric
  gateway requires a UUID `channel` query param) — decide at p10-c005 alongside the
  forge-write / CDC trigger.
