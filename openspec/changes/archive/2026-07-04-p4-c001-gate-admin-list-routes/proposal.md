## Why

`fpa-gate::list_routes` returns `PortError::Downstream("not implemented")` and the
`GateAdapter` stub targets the wrong path (bare `/routes`). Gate is the last
unimplemented plane. Implement real route listing against gate's admin API.

## What Changes

- Give `GateAdapter` a `reqwest` client; `list_routes` → `GET {admin_base}/routes`,
  forwarding the admin bearer.
- **Path resolved from source (2026-07-02):** gate's `admin_router` mounts routes
  **bare** (`/health`, `/ready`, `/routes`, `/routes/:id`, `/api-keys`,
  `/signing-keys`, …) and `main.rs` serves `admin_app` **directly on the admin
  listener with NO `/v1/admin` nest**. So the real path is **`GET /routes`** (bare)
  on the admin port. `flint-gate-client`'s `ADMIN_PREFIX = "/v1/admin"` does **not**
  match the current server mount (stale client or reverse-proxy assumption) — do
  NOT use it. Take the admin base URL from config; append `/routes`.
- Map gate 401/403 → `PortError::Unauthorized`, other non-2xx → `Downstream`,
  unreachable → `Transport`.
- **Reads only this phase** — no route writes (`POST/DELETE /routes`); `application.deploy`
  route-upsert stays a follow-on.

## Capabilities

### New Capabilities
- `gate-admin-routes`: Real `fpa-gate` route listing against gate's admin API (`:4457`), completing the last unimplemented plane.

### Modified Capabilities

## Impact

- `fpa-gate` (reqwest client + real `list_routes`); add `reqwest` + `wiremock` dev-dep.
- No new runtime deps (reqwest present). **Reject** the `flint-gate-client` git-dep
  (WS-heavy, cross-org) per analyze.
- Independent of the other p4 changes.

## Open Questions
- **Path RESOLVED:** bare `GET /routes` on the admin port (verified from gate
  source). Adapter takes the admin base URL from config; appends `/routes`.
- New gate admin surface also exposes `/api-keys` and `/signing-keys` CRUD —
  out of scope here (reads = routes only), noted for a future admin phase.
