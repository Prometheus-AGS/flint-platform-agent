# Goals — live-smoke-and-durable-store

> Seeded from `integration-proof-and-debt-closure/reflection.md`. **RE-SCOPED at
> assess (operator decision, 2026-07-03):** the live 3-plane smoke is **deferred to
> its own future phase** — it can't run in this environment (Docker daemon
> unreachable; forge has no compose, only a heavy custom pgrx PG-18 image). This
> phase now focuses on the **durable ProjectStore** (implementable now) + **p5
> archival**. The phase name is retained for continuity; "live-smoke" is deferred.

## Primary goals (re-scoped)

1. **Durable `ProjectStore` (agent-owned Postgres).** Add a Postgres-backed adapter
   behind the existing `ProjectStore` port so projects survive a restart (p5 debt
   #2). The port already exists; this is a **new adapter crate** (`fpa-store-pg` —
   NOT in `fpa-app`, per hexagonal Rule 16) + composition-root wiring — no call-site
   changes. Keep the in-memory adapter for tests. Store the `Project` aggregate whole
   as a JSONB `body` column. Driver: `tokio-postgres` (sibling-consistent with
   fabric) unless analyze finds a better fit.

2. **Archive the p5 OpenSpec changes.** `/opsx:archive` p5-c001..c004 into `specs/`
   (p5 debt #3) — housekeeping so the spec baseline reflects shipped capabilities.

## Deferred to a future phase (operator decision)

- **Live 3-plane smoke** against real forge/gate/fabric — needs colima/Docker up, a
  heavy forge pgrx PG-18 build, and gate/fabric compose. Returns as its own phase
  once the stack can be stood up. (The mock-boundary proof from p5 stands until then.)
- **Durable store in a forge `flint_meta.projects` table** — rejected for this phase
  because it inherits the same forge/Docker blocker; agent-owned Postgres is the
  chosen persistence, consistent with the p5 decision.

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
