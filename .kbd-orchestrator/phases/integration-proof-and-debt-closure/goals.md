# Goals — integration-proof-and-debt-closure

> Seeded from `gate-admin-and-auth-hardening/reflection.md` → "Recommended Next Phase".
> This is the **holistic integration proof** the implementation-first philosophy
> calls for: all four planes are implemented, but the system has **never run
> end-to-end against the real stack**. Prove the shape, then close the honest debt.

## Primary goals

1. **End-to-end integration proof.** Stand up (or mock at the HTTP boundary) a real
   forge + gate + fabric, and drive one full operator flow through the agent's
   protocol surfaces:
   `authenticate → project.create (forge REST) → list_routes (gate admin) → fabric.health`
   exercised across **AG-UI**, **A2A**, and **MCP**. This is the "test the proven
   shape of the whole system" milestone — the first real `cargo test` integration
   pass justified under the ≤3-per-goal budget.

2. **Close the deferred security-review findings** (from c002):
   - **H2** — authenticate `/agui/stream` (no longer a stub-exempt path).
   - **H4** — close the JWKS cache single-flight / TOCTOU double-fetch window.
   - **M2** — surface `signature_verified` in an audit record (never log the token).

3. **Fix the misplaced gate write-kind guard** (debt #1). `application.deploy`
   (→ `TargetPort::Gate`) must **refuse cleanly** with a "gate route-write not
   implemented" error, not silently fall through to `list_routes`.

4. **Confirm forge table/collection names** (debt #3). Verify the real forge REST
   table names against a running forge (or `fdb-reflection`) and replace the
   `"Projects"` / `"Applications"` guesses in `dispatch_forge` — keep them
   config-overridable.

## Success criteria

- One integration test binary drives the full authenticate→create→list→health flow
  and passes (green `cargo test`).
- `/agui/stream` rejects unauthenticated requests; JWKS fetch is single-flight;
  an audit record captures `signature_verified` (no secrets logged).
- `application.deploy` returns a clean refusal (asserted by a test), not a route list.
- Forge table names are verified-from-source (or explicitly documented as
  config-driven with the real defaults).

## Open questions (for /kbd-assess → /kbd-analyze)

- **Can the full sibling stack run locally / in compose** (forge Postgres 18 + gate
  + fabric) for a real integration test, or do we mock each plane at the HTTP
  boundary (wiremock-style) for a first proof?
- **Which IdP backs `FPA_JWKS_URL`** (Ory Kratos vs Hydra) for a real direct-token
  verification test — and is a test JWKS/keypair available?

## Explicitly out of scope this phase (still deferred)

Forge update/delete, MCP multi-server composition, fabric WS subscriptions,
OpenDesign plugins, A2UI UI generation / React-Vite build, Tauri packaging,
knowledge-base, durable task store. (L2/L3 security findings remain low-priority.)
