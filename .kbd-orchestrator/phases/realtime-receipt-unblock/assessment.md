# Assessment — realtime-receipt-unblock

**Phase:** realtime-receipt-unblock
**Stage:** assess
**Date:** 2026-07-08
**Backend:** OpenSpec (`openspec/changes/p12-c00*` — carried, unarchived)
**Verdict: BLOCKED — no agent-side work is actionable this phase.**

---

## What this phase depends on

This phase is a single fabric-gated gate: it becomes actionable **only** when the
upstream broker fix (`Prometheus-AGS/flint-realtime-fabric#2`) ships. Every task
in `goals.md` is downstream of that fix. So the assessment is really one
question — *has the fix landed?* — checked against the **live upstream issue and
the in-sync local fabric checkout**, not against my earlier (now-stale) notes.

## Upstream blocker status — CONFIRMED STILL OPEN (re-grounded against latest code)

| Signal | Finding |
|---|---|
| Issue `flint-realtime-fabric#2` | **OPEN** — `closedAt: null`, 0 comments, no labels, no assignee, updated 2026-07-07 |
| Open fix PR referencing #2 / broker / subscribe | **None** (`gh pr list --state open` → `[]`) |
| Local fabric checkout | `main @ 7d3d1e9`, **0 ahead / 0 behind origin** — this IS the latest |
| `broker.rs` touched since I filed? | **Yes** — by `39770a2` ("phases 16-17 production hardening"), so I re-read the source directly rather than trust line numbers |

### The divergence is unchanged at `7d3d1e9` (verified in source, not assumed)

| Path | stream | topic | file:line |
|---|---|---|---|
| `publish` | `stream_name(tenant_id)` = `tenant-{tenant_id}` | `topic_name(path)` | `broker.rs:93-94` |
| `ensure_channel` (creates the stream) | `tenant-{tenant_id}` | `topic_name(path)` | `broker.rs:265-266` |
| **`subscribe`** | **`channel-{channel_id}`** | **`"events"`** (hardcoded) | `broker.rs:129-130` |

Subscribe still consumes from `channel-<uuid>`/`events`, which **nothing creates**
→ against a real Iggy `consumer.init()` fails "Stream not found" → WS upgrade
fails → the agent's `/fabric/subscribe` bridge 502s. Exactly the KI-1 root cause.

**Worse, not better:** the `39770a2` "hardening" commit *added* `get_consumer_offset`
(`:56-57`), and `store_offset` (`:179-180`) + `ack` (`:225-226`) all copy the same
wrong `channel-{...}`/`"events"` naming. The divergence propagated into the new
offset/ack paths instead of being fixed. Fabric's own integration test
(`tests/publish_subscribe.rs:29`) is still `#[ignore]`d — this path has still never
run green in their CI.

## Gap analysis against phase goals

| Goal | Status | Gap / blocker |
|---|---|---|
| Watch fabric#2 | **DONE (this assess)** | Re-verified against latest code: still OPEN, divergence intact, no fix in flight. |
| Flip `test.fixme` → `test`, prove 5/5 | **BLOCKED** | Would 502 exactly as before — flipping now = knowingly faking a red test green. Base Rule 5 / "never fake green" forbids it. |
| `/opsx:verify` + `/opsx:archive` c001–c003 | **BLOCKED** | c003 verification is still honestly PARTIAL (4/5). Archiving now would archive an unproven receipt. All three changes remain in `openspec/changes/` (verified on disk). |
| Update KNOWN-ISSUES KI-1 → RESOLVED | **BLOCKED** | KI-1 is still OPEN and accurate — must not be flipped. |

## Current on-disk state (unchanged, correct as-is)

- `smoke/smoke.real.spec.ts:152` — `test.fixme(...)` marker present. **Leave it.**
- `smoke/KNOWN-ISSUES.md:10` — KI-1 `Status: OPEN`. **Accurate — leave it.**
- `openspec/changes/p12-c00{1,2,3}` — all three unarchived. Correct.
- `openspec/changes/p12-c003-.../tasks.md:34` — task 2.1 `[~]` with the
  upstream-block annotation. Correct.
- Branch `feat/p12-realtime-receipt` — the committed 4/5 proof-of-life stands.

## Recommendation

**Do not proceed to Analyze/Plan/Execute for the receipt-unblock work — there is
nothing to build; the agent side is already proven and the only remaining work is
gated on an upstream fix that has not shipped.** The correct move is the
reflection's stated alternative:

1. **Let the unblock ride as a background watch.** Re-enter this phase (re-run
   `/kbd-assess`) only when fabric#2 closes or a `broker.rs` commit aligns
   subscribe's naming with publish. When that happens the work is mechanical:
   flip one marker → `run-real.sh --no-build` → confirm true 5/5 → verify+archive
   → KI-1 RESOLVED.
2. **Pivot this session to the next unrelated administrative surface** rather than
   idle-block — candidates from the reflection: **A2A task-catalog expansion** or
   **gate route-admin**. That keeps forward progress on the agent while fabric#2
   is out of our hands (constraint stands: nothing written into any sibling repo).

**Optional nudge (out of scope for the agent's own work, operator's call):** the
fix is small and entirely inside fabric — align subscribe/offset/ack stream+topic
to `stream_name(tenant_id)` + `topic_name(path)` and un-`#[ignore]` the test.
Could be done in a separate fabric-repo session if the operator wants to unblock
directly. Not this agent's code to change.
