## ADDED Requirements

### Requirement: The realtime test proves end-to-end EventEnvelope receipt

`smoke.real.spec.ts` SHALL contain a realtime test that mints a dev RS256 bearer, seeds
the required Keto relation tuples, subscribes to the agent's `/fabric/subscribe` SSE
bridge, publishes a `EventEnvelope` with a known `id` to fabric's `/v1/publish`, and
asserts the agent's SSE bridge emits that envelope within a timeout. The existing HTTP
hop tests SHALL remain passing.

#### Scenario: Agent receives a fabric EventEnvelope end-to-end

- **WHEN** the smoke mints a dev bearer, seeds Keto tuples, subscribes via the agent's SSE
  bridge, and publishes the envelope to fabric
- **THEN** the agent's SSE stream emits the EventEnvelope with the expected id within 15s

#### Scenario: Keto relation tuples gate the receive correctly

- **WHEN** the subscribe tuple and the view tuple for the envelope id are seeded
- **THEN** the fabric subscribe pipeline allows both the upgrade and the per-event delivery

#### Scenario: The existing HTTP hops are unaffected

- **WHEN** the full real smoke runs (all 5 tests including the updated realtime test)
- **THEN** all 5 pass (the subscribe + realtime is test 5; the 4 HTTP hops are tests 1-4)
