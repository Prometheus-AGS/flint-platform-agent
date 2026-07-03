# Analysis — live-smoke-and-durable-store

**Phase:** live-smoke-and-durable-store (re-scoped: durable store + p5 archival)
**Date:** 2026-07-03
**Mode:** stack-specified (agent-owned Postgres, decided at assess)
**Research:** Tier 3 (registry version verification) run; Tiers 1/2/4 not needed —
the driver family is set by sibling consistency, not discovery.

---

## 1. Scope (post-assess)

Live smoke is **deferred** (operator). This phase = **durable `ProjectStore`** (new
`fpa-store-pg` adapter crate, agent-owned Postgres) + **archive p5 changes**. The
only real analyze question is the exact dependency set + versions for the Postgres
adapter and its test.

---

## 2. Dependency decisions (Tier 3 verified, 2026-07-03)

| Crate | Version (verified) | Decision | Basis |
|---|---|---|---|
| `tokio-postgres` | **0.7.18** | **adopt** (driver) | flint-realtime-fabric uses `tokio-postgres = "0.7"` (`with-serde_json-1`). Sibling-consistency (Base Rule 9) beats `sqlx`'s compile-time query checking here — the store does one JSONB upsert + one select, not a large query surface. Same `with-serde_json-1` feature to bind `serde_json::Value`. |
| `deadpool-postgres` | **0.14.1** | **adopt** (pool) | The agent is a long-lived server; a connection pool is required (unlike fabric's narrower usage). `deadpool-postgres` is the idiomatic pool over `tokio-postgres`, async, small. |
| `sqlx` | 0.9.0 | **reject** | Compile-time checking needs a live DB at build time or offline metadata — friction for CI and a different driver family than the sibling. Not worth it for a two-query adapter. |
| `testcontainers` + `testcontainers-modules` | **0.27.3 / 0.15.0** | **adopt** (dev-dep) | Ephemeral `postgres` container for the round-trip/restart test. `-modules` provides the ready Postgres image. Dev-only; the test is `#[ignore]`d-by-default (needs Docker, which is down here). |
| `deadpool` (core) | (via deadpool-postgres) | transitive | — |

**No production dep beyond `tokio-postgres` + `deadpool-postgres`.** Both are new to
the workspace (the agent had no Postgres dep). `rustls` TLS for the DB connection is
deferred — local/trusted-network Postgres this phase; add `tokio-postgres-rustls`
when a remote DB lands (note it in the reflection).

---

## 3. Architecture (hexagonal placement)

- **New crate `fpa-store-pg`** (workspace member) — the adapter. Implements
  `fpa_ports::ProjectStore`. Depends on `fpa-ports`, `fpa-domain`, `tokio-postgres`,
  `deadpool-postgres`, `serde_json`. **Not** depended on by `fpa-app` (app layer stays
  infra-free, Rule 16); only `fpa-gateway` (composition root) imports it.
- **Schema:** one table, `Project` stored **whole** as JSONB:
  ```sql
  CREATE TABLE IF NOT EXISTS fpa_projects (
      id            uuid PRIMARY KEY,
      name          text NOT NULL,
      schema_version integer NOT NULL,
      body          jsonb NOT NULL,          -- the full Project aggregate
      updated_at    timestamptz NOT NULL DEFAULT now()
  );
  ```
  `put` = `INSERT … ON CONFLICT (id) DO UPDATE`; `get` = `SELECT body WHERE id = $1`,
  deserialized back to `Project`. The agent owns/creates this table (a bundled
  `schema.sql` run at adapter init, or a documented migration) — it is the agent's
  own store, not a forge/fabric table.
- **Composition root:** `AppState::new` selects the store by config
  (`FPA_PROJECT_DB_URL` present ⇒ `PgProjectStore`, else `InMemoryProjectStore`), so
  the swap is config-only and tests keep the in-mem path. New env var
  `FPA_PROJECT_DB_URL` (optional; redacted in `Debug`).

---

## 4. Build-vs-adopt (all BUILD/ADOPT-lib, no skeleton)

| Gap | Decision | Note |
|---|---|---|
| G2 durable ProjectStore | **build** adapter, **adopt** `tokio-postgres`+`deadpool-postgres` | new `fpa-store-pg` crate |
| G3 store location | **resolved** (agent PG) | operator decision at assess |
| G4 archive p5 | **build** (housekeeping) | `/opsx:archive` ×4 |

No `adopt`-a-whole-solution candidate — the adapter is thin, hand-written over the
driver. `library-candidates.json` records the crate adoptions.

---

## 5. Test strategy

- **Fast path (no Docker):** unit tests for the SQL-string builders / serde
  round-trip of the `Project` ↔ JSONB mapping, and the in-mem adapter (already
  present) — run on every `cargo test`.
- **Integration (needs Docker):** a `testcontainers` Postgres test proving
  `put`→restart(new pool)→`get` survives — the durable proof. **`#[ignore]`d by
  default** (Docker is down here); documented run command:
  `cargo test -p fpa-store-pg -- --ignored`. This is the phase's one heavier test and
  will only actually run once colima is started — noted honestly, not silently.

---

## 6. Open questions

None blocking. Resolved: driver (`tokio-postgres`), pool (`deadpool-postgres`),
test infra (`testcontainers` + modules, `#[ignore]`d), store location (agent PG),
schema (single JSONB table). Residual (reflection): TLS for a remote DB, and a real
migration tool if the schema grows beyond one table.

---

## 7. Handoff to spec/plan

Durable store = new **`fpa-store-pg`** adapter crate implementing `ProjectStore` over
**`tokio-postgres` 0.7.18** + **`deadpool-postgres` 0.14.1** (verified), storing the
`Project` whole as JSONB in a single agent-owned table; composition root picks
Pg-vs-InMem by `FPA_PROJECT_DB_URL`. Test: unit round-trip always; a `testcontainers`
0.27 Postgres restart-survival test `#[ignore]`d-by-default (Docker down here).
Second change: archive p5-c001..c004. No operator gates remain. Spec these two.
