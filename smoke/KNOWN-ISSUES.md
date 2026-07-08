# Smoke — Known Issues

Auditable record of blockers the real smoke has surfaced that are **not** the agent's
to fix. Each entry links the marked-off test/step to the upstream defect.

---

## KI-1 — Realtime receipt blocked: fabric subscribe/publish target different Iggy streams

**Status:** OPEN — upstream (`flint-realtime-fabric`), filed
[Prometheus-AGS/flint-realtime-fabric#2](https://github.com/Prometheus-AGS/flint-realtime-fabric/issues/2).
**Surfaced by:** `smoke.real.spec.ts › realtime: agent receives a fabric EventEnvelope
end-to-end` (marked `test.fixme` — reports SKIPPED, not FAILED).
**Phase:** p12-c003-realtime-receipt.

### Symptom

The agent's `/fabric/subscribe` SSE bridge returns **HTTP 502**
(`{"error":"fabric subscribe failed"}`) against the fully real stack (real gate + real
fabric + real Iggy + Keto, RS256 on both hops, no `DEV_NO_AUTH`). The other 4 real
smoke assertions pass.

### Root cause (in fabric, not the agent)

Auth is **not** the problem — the forwarded RS256 bearer verifies, and fabric reaches
deep into its subscribe pipeline before failing:

```
ws::subscribe → app::subscribe → port::LogBroker::subscribe → iggy::clients::consumer
ERROR iggy::clients::consumer: Stream: channel-<uuid> was not found.
```

In `flint-realtime-fabric/crates/frf-broker-iggy`, the subscribe and publish paths name
**different Iggy streams and topics**:

| Path | stream | topic |
|---|---|---|
| `publish` / `ensure_channel` (`broker.rs:93`, `:265`; `channel.rs:7`) | `tenant-{tenant_id}` | `topic_name(path)` — e.g. `entity_smoke_realtime` |
| `subscribe` (`broker.rs:129`) | `channel-{channel_id}` | `"events"` (hardcoded) |

Subscribe consumes from `channel-<uuid>`/`events`, which **nothing ever creates** —
`ensure_channel` and the boot-time fixture (`frf-gateway/src/main.rs:115`) both create
`tenant-<uuid>`. Against a real Iggy that rejects unknown streams, `consumer.init()`
errors → the WS upgrade fails → the agent's `connect_async` fails → the bridge 502s.

The same divergence exists in fabric's own integration test
(`crates/frf-broker-iggy/tests/publish_subscribe.rs`): it `ensure_channel`s +
`publish`es to `tenant-<nil>` but `subscribe`s from `channel-<id>`. That test is
`#[ignore]`, so this path **has never run green in CI** — it is a latent bug that the
first genuinely-real subscribe surfaces.

### Fix (upstream)

Align subscribe's stream/topic naming with publish (use `stream_name(tenant_id)` +
`topic_name(path)` on the subscribe/offset/poll/ack paths), and wire `ensure_channel`
into the subscribe path (or resolve the channel's tenant/path so subscribe and publish
provably meet on the same stream+topic). Un-`#[ignore]` `publish_subscribe.rs` so CI
guards it.

### What is proven regardless

The agent side is correct and committed as the proof-of-life: it opens the WS to
`/ws/v1/subscribe` and forwards the RS256 `FPA_FABRIC_BEARER`; fabric accepts and
verifies it. The 502 originates entirely inside fabric's broker adapter.

**To un-block:** once fabric ships the fix, flip `test.fixme` → `test` in
`smoke.real.spec.ts` and re-run `smoke/run-real.sh --no-build` for a true 5/5.
