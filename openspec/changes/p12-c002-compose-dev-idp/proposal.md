## Why

With the RSA key pair (c001), the fabric gateway needs to be pointed at the JWKS URL so
it can verify the smoke's bearer at runtime. Two compose changes are needed:

1. **A JWKS server** — a tiny service that serves `smoke/dev-idp/jwks.json` as a static
   file so `OryIdentityVerifier` can fetch it.
2. **`GATEWAY_JWKS_URL`** updated to point at the JWKS server (currently set to a
   placeholder in `compose.fabric.yml` and to gate's signing-keys in `compose.real.yml`).
3. **Keto `:4467` port mapping** — the write port is needed by the smoke spec to seed
   relation tuples from the host; it is currently not mapped.

## What Changes

- **`smoke/compose.fabric.yml`** and **`smoke/compose.real.yml`**: add a `dev-idp-jwks`
  service (`nginx:alpine` or `python3 -m http.server` one-shot) serving
  `smoke/dev-idp/jwks.json`; update `fabric-gateway.GATEWAY_JWKS_URL` →
  `http://dev-idp-jwks:8080/jwks.json`; add Keto port mappings (`14466:4466`,
  `14467:4467`) to `fabric-keto`.
- The JWKS server is a **read-only static file** — no new container logic, just nginx
  default with one `location`. Alternatively, `python3 -m http.server 8080` via a shell
  entrypoint. Both are smoke-owned; nothing written into the fabric repo.

## Capabilities

### New Capabilities
- `compose-dev-idp`: The fabric gateway is wired to verify bearers signed with the dev RSA key; Keto's write port is accessible from the host for tuple seeding.

## Impact

- `smoke/compose.fabric.yml`, `smoke/compose.real.yml` (each adds 1 service + edits 2-3
  env vars + 2 port mappings). No agent Rust change. Both compose files remain backward-
  compatible (the JWKS server only matters when the smoke issues a real bearer).
