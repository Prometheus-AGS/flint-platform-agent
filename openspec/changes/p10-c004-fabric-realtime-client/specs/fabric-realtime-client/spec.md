## ADDED Requirements

### Requirement: The agent can subscribe to fabric's realtime change stream

`FabricClient` SHALL provide a `subscribe` operation that connects to fabric's
`/ws/v1/subscribe` WebSocket and yields a stream of `frf-domain::EventEnvelope` events —
the fabric's real CDC change-stream wire type. The agent MUST reuse
`frf-domain::EventEnvelope` (not define its own), and unknown/future `EventKind` variants
MUST NOT panic (`EventKind` is `#[non_exhaustive]`; deserialize is lenient).

> **Grounding correction (2026-07-04):** the earlier draft of this requirement named
> `frf-agentproto::ContentBlock`. Verified against
> `../flint-realtime-fabric/crates/frf-gateway/src/routes/subscribe.rs`: `/ws/v1/subscribe`
> serializes `EventEnvelope`, not `ContentBlock`. `ContentBlock` is the payload for the
> *separate* `/ws/v1/agents` bus. For the forge-write → CDC → agent-receives-change smoke,
> `EventEnvelope { kind: EntityChange, .. }` is the correct type. Corrected by operator
> decision (Base Rule 10).

#### Scenario: Subscribe yields change events

- **WHEN** the agent subscribes to a channel and a change occurs upstream (CDC)
- **THEN** the agent receives a deserialized `EventEnvelope` for that change on the stream

#### Scenario: Unknown event kind does not crash

- **WHEN** a frame carries an unrecognized `EventKind` (or future field)
- **THEN** it deserializes without panicking (lenient serde / `#[non_exhaustive]` kind)

### Requirement: Subscribe uses the fabric's real request contract

`subscribe` SHALL connect to `ws://{endpoint}/ws/v1/subscribe?channel=<UUID>` with the
`channel` as a UUID query parameter, forwarding the operator's bearer as an
`Authorization: Bearer <token>` header. A missing/invalid channel or bearer maps to the
fabric's documented rejection (400 bad channel, 401 no bearer) surfaced as `PortError`.

#### Scenario: Missing bearer is rejected

- **WHEN** the agent subscribes without a bearer against an auth-enforcing fabric
- **THEN** the connection is rejected (401) and `subscribe` returns a `PortError`, not a panic

### Requirement: Subscription failures map to the port error surface

Transport failures (connect refused, unexpected close, deserialize error) SHALL map to
`PortError`; the client MUST NOT panic. The `health()` path is unchanged.

#### Scenario: Fabric unreachable on subscribe

- **WHEN** the fabric WS endpoint is unreachable
- **THEN** `subscribe` returns/ends with a `PortError`, not a panic
