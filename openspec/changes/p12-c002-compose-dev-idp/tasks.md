## 1. JWKS server service

- [x] 1.1 Add `dev-idp-jwks` service to both `compose.fabric.yml` and `compose.real.yml`:
  `image: nginx:alpine`, mounts `./dev-idp/jwks.json:/usr/share/nginx/html/jwks.json:ro`,
  exposes port 8080 internally (or any available port). A default nginx config serves the
  file at `http://dev-idp-jwks:8080/jwks.json`. Simple healthcheck (or none needed —
  fabric's JWKS fetch has a retry).

## 2. Update GATEWAY_JWKS_URL + Keto port mapping

- [x] 2.1 `compose.fabric.yml`: update `fabric-gateway.GATEWAY_JWKS_URL` from the
  placeholder (`http://fabric-gateway:8080/.well-known/jwks.json`) to
  `http://dev-idp-jwks:8080/jwks.json`. Add `depends_on: dev-idp-jwks`.
- [x] 2.2 `compose.real.yml`: update `fabric-gateway.GATEWAY_JWKS_URL` from
  `http://flint-gate:4457/signing-keys` to `http://dev-idp-jwks:8080/jwks.json`
  (in the realtime-receipt context the smoke mints its own bearer, not gate's). Add
  `depends_on: dev-idp-jwks`.
- [x] 2.3 `compose.fabric.yml` + `compose.real.yml`: add port mappings to `fabric-keto`:
  `- "14466:4466"` (read) and `- "14467:4467"` (write). Both needed from the host for
  the Keto seed + health check.

## 3. Verification

- [x] 3.1 `docker compose -f smoke/compose.fabric.yml config` — parses without error.
- [x] 3.2 `docker compose -f smoke/compose.real.yml config` — parses without error.
- [x] 3.3 Brief up test (just the JWKS server + `curl http://localhost:<host_port>/jwks.json`)
  — confirms the static file is served. No full stack boot needed for c002 alone.
