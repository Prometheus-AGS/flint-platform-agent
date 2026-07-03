## 1. Gate admin client

- [x] 1.1 Add `reqwest` (workspace) to `fpa-gate`; give `GateAdapter` a `reqwest::Client`.
- [x] 1.2 Add `wiremock` 0.6 as a dev-dependency.
- [x] 1.3 Confirm the admin routes path (bare `/routes` vs `/v1/admin/routes`) against gate's admin mount / `flint-gate-client` base-URL construction; take the admin base URL from config.

## 2. Implement list_routes

- [x] 2.1 `list_routes` → `GET {admin_base}/…/routes` with the admin bearer.
- [x] 2.2 Map 401/403 → `Unauthorized`, other non-2xx → `Downstream`, unreachable → `Transport`.
- [x] 2.3 Gate-plane write kinds (e.g. `application.deploy`) return `Downstream("gate route-write not implemented this phase")` — no write.

## 3. Verification (wiremock)

- [x] 3.1 `cargo check/clippy/fmt` green.
- [x] 3.2 wiremock: 200 routes → returned; 401 → Unauthorized; unreachable → Transport.
