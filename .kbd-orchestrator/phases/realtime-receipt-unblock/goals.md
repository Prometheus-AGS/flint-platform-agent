# Goals

Seeded from `realtime-receipt` reflection → "Recommended Next Phase". This phase
is **fabric-gated**: the only agent-side work is a one-line marker flip once the
upstream broker fix lands. Do **not** re-open agent receipt work — the agent side
is already proven correct (opens the WS, forwards a verified RS256 bearer).

- **Watch the upstream blocker.** Track `Prometheus-AGS/flint-realtime-fabric#2`
  (KI-1: `frf-broker-iggy` subscribe/publish target different Iggy streams —
  subscribe reads `channel-{channel_id}`/`events`, publish writes
  `tenant-{tenant_id}`/`topic_name(path)`; nothing creates `channel-<uuid>` so
  `consumer.init()` fails "Stream not found" → the agent bridge 502s). Confirm
  the fix aligns subscribe's stream/topic naming with publish (and un-`#[ignore]`s
  fabric's own `crates/frf-broker-iggy/tests/publish_subscribe.rs`).

- **Flip the marker and prove 5/5.** When fabric ships the fix, change
  `test.fixme` → `test` on `realtime: agent receives a fabric EventEnvelope
  end-to-end` in `smoke/smoke.real.spec.ts`, re-pull the fabric image, and re-run
  `smoke/run-real.sh --no-build` for a **true 5/5** (real gate + real fabric +
  real Iggy + Keto, RS256 on both hops, no `DEV_NO_AUTH`). Never fake green — if
  it still 502s, the fabric fix is incomplete; re-verify #2, do not weaken the
  test.

- **Close the OpenSpec debt.** Once the smoke is genuinely 5/5, mark c003 task
  2.1 `[x]` (drop the `[~]` upstream-block annotation), then `/opsx:verify` +
  `/opsx:archive` `p12-c001` / `p12-c002` / `p12-c003` — currently held back only
  because c003's verification is honestly PARTIAL. Update `smoke/KNOWN-ISSUES.md`
  KI-1 → RESOLVED with the fabric commit/release that fixed it.

- **Constraints (carried, non-negotiable):** nothing written into
  `../flint-realtime-fabric` (or any sibling repo) — smoke-owned artifacts only;
  RS256 on both hops, no `DEV_NO_AUTH`; the `FPA_JWKS_URL` https-only guard stays;
  the committed dev-idp key remains an intentional throwaway smoke-only credential.

## Alternative if fabric#2 lingers

If #2 stays open, do **not** block on it — let the unblock ride as a background
watch and pivot to the next unrelated administrative surface (A2A task-catalog
expansion or gate route-admin). Re-enter this phase when the fabric fix lands.
