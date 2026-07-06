## 1. Real fabric services (proven standalone in smoke/compose.fabric.yml; folds into compose.real.yml at c005)

- [x] 1.1 Modeled on fabric's OWN `compose.ci.yml` (their self-contained /healthz stack): `fabric-gateway` (`build` from `../flint-realtime-fabric` Dockerfile, `CARGO_FEATURES=dev-endpoints`), `fabric-iggy` (`iggyrs/iggy:latest`), `fabric-keto` (`oryd/keto:v0.12`), `fabric-postgres` (`postgres:17-alpine`, logical WAL). surrealdb + flint-gate omitted (not needed for health). PROVEN: all 4 healthy.
- [x] 1.2 Gateway env wired: `IGGY_CONNECTION_STRING`, `KETO_BASE_URL`, `KETO_NAMESPACE`, `GATEWAY_JWKS_URL` (placeholder — hard-required at parse even in dev), `DEV_NO_AUTH=true`, `CDC_ENABLED=false` (health-only; c005 flips true). `depends_on`: iggy/postgres `service_healthy`, keto `service_started`.
- [x] 1.3 Services `fabric-` prefixed (no collision with gate/forge PG). Two fixes vs fabric's broken untracked `compose.ci.yml`: (a) keto v0.12 needs a config FILE — vendored `smoke/fabric-config/keto.yml` + `serve -c`; (b) `GATEWAY_JWKS_URL` must be set.
- [x] 1.4 Agent env `FPA_FABRIC_ENDPOINT` → `http://fabric-gateway:8080` (host 28080 standalone).

## 2. Verification

- [x] 2.1 fabric gateway builds (rust:latest, full workspace); iggy/keto/postgres healthy; gateway healthy. PROVEN.
- [x] 2.2 `GET /healthz` on the real fabric gateway → **HTTP 200** `{"status":"ok","version":"0.1.0"}` (the agent's exact `fabric.health` hop). PROVEN.
- [x] 2.3 Generous `start_period` (gateway 40s — heaviest). No OOM on the 12 GiB VM in the standalone run. **FINDING (for c005):** iggy client/server version mismatch — gateway logs `invalid_command` / `Failed to get cluster metadata` against `iggyrs/iggy:latest`; degrades gracefully (health stays 200) but MAY affect c005's realtime event path (the bus rides on iggy). Consider pinning the iggy tag at c005. Recorded in memory.
