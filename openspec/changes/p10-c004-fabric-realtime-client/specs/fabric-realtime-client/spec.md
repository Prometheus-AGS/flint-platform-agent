## ADDED Requirements

### Requirement: The agent can subscribe to fabric's realtime change stream

`FabricClient` SHALL provide a `subscribe` operation that connects to fabric's
`/ws/v1/subscribe` WebSocket and yields a stream of `frf-agentproto::ContentBlock`
events. The agent MUST reuse `frf-agentproto`'s `ContentBlock` (not define its own),
and unknown/future frame variants MUST NOT panic (the `Unknown` catch-all).

#### Scenario: Subscribe yields change events

- **WHEN** the agent subscribes and a change occurs upstream (CDC)
- **THEN** the agent receives a deserialized `ContentBlock` for that change on the stream

#### Scenario: Unknown frame does not crash

- **WHEN** a frame with an unrecognized `type` arrives
- **THEN** it deserializes to `ContentBlock::Unknown` (or is skipped) without panicking

### Requirement: Subscription failures map to the port error surface

Transport failures (connect refused, unexpected close, deserialize error) SHALL map to
`PortError`; the client MUST NOT panic. The `health()` path is unchanged.

#### Scenario: Fabric unreachable on subscribe

- **WHEN** the fabric WS endpoint is unreachable
- **THEN** `subscribe` returns/ends with a `PortError`, not a panic
