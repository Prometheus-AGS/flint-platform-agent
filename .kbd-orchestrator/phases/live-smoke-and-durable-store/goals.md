# Goals — live-smoke-and-durable-store

> Seeded from `integration-proof-and-debt-closure/reflection.md` → "Recommended Next
> Phase". Phase 5 proved the system end-to-end **against mocked planes**; this phase
> closes the two things that leaves open — **real wire compatibility** and **durable
> persistence** — plus the p5 archival housekeeping.

## Primary goals

1. **Live smoke against real siblings.** Stand up real forge (pgrx Postgres-18 image)
   + gate + fabric (their compose files) and run the same operator flow the p5
   integration proof runs — `authenticate → project.create → list_routes (gate) →
   fabric.health` — against the **live** services. Fix any wire drift the
   HTTP-boundary mocks hid (paths, headers, status codes, auth handshake). This is
   the one thing the mock-boundary proof left unproven (p5 debt #1).

2. **Durable `ProjectStore`.** Add a Postgres-backed adapter behind the existing
   `ProjectStore` port so projects survive a restart (p5 debt #2). The port already
   exists; this is a new adapter + composition-root wiring — no call-site changes.
   Keep the in-memory adapter for tests.

3. **Archive the p5 OpenSpec changes.** `/opsx:archive` p5-c001..c004 into `specs/`
   (p5 debt #3) — housekeeping so the spec baseline reflects shipped capabilities.

## Success criteria

- A live-smoke test (gated/ignored by default, run explicitly) drives the real flow
  against running forge/gate/fabric and passes — or the wire drift it surfaces is
  fixed and documented.
- `project.create` persists to Postgres; a round-trip survives a store restart
  (proven by an integration test against a real/testcontainer Postgres).
- The four p5 changes are archived; `openspec/specs/` reflects the shipped
  capabilities.

## Open questions (for /kbd-assess → /kbd-analyze)

- **Can the full sibling stack run here?** forge needs a pgrx Postgres-18 image
  build; gate + fabric have compose. How heavy is standing all three up on the dev
  machine / CI? If heavy, scope the live smoke **one plane at a time** (forge first,
  then gate, then fabric) rather than all-at-once.
- **Where does the durable `ProjectStore` live?** The agent's own Postgres, or a
  forge `flint_meta.projects` table written via the now-correct
  `POST /{schema}/{table}`? (Revisits the p5 operator decision now that the forge
  write path is fixed — the earlier "no forge table" blocker is partly lifted.)
  Likely an **operator decision**.
- **Postgres driver + test strategy:** `sqlx` vs `tokio-postgres` vs `deadpool`;
  testcontainers vs a dev-compose Postgres for the durable-store test. Verify current
  versions (Base Rule 22).

## Explicitly out of scope this phase (still deferred)

Richer `project.create` input (applications/sub-agents/schemas), `application.define`
persistence home, MCP multi-server composition, fabric WS subscriptions, OpenDesign
plugins, A2UI/React UI generation, Tauri packaging, knowledge-base.
