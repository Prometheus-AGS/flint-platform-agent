## 1. Real fabric services (into compose.real.yml)

- [ ] 1.1 Port fabric's services from `../flint-realtime-fabric/compose.yml`: `fabric-gateway` (`build: { context: ../../flint-realtime-fabric }`), `iggy-server` (`iggyrs/iggy:latest`), `keto` (`oryd/keto:v0.12`, mount its config), `surrealdb`, `fabric-postgres` (`postgres:17`).
- [ ] 1.2 Wire the gateway env (`IGGY_CONNECTION_STRING`, `KETO_BASE_URL`, DB) + `depends_on` (iggy service_healthy, keto service_started, surreal/pg service_healthy) exactly as fabric's compose.
- [ ] 1.3 Rename services with a `fabric-` prefix to avoid collisions with gate/forge postgres etc.
- [ ] 1.4 Agent env `FPA_FABRIC_ENDPOINT` → `http://fabric-gateway:<port>`.

## 2. Verification

- [ ] 2.1 fabric gateway builds; iggy/keto/surreal/pg become healthy; gateway boots.
- [ ] 2.2 `curl` fabric-gateway `/healthz` → 200; the agent's `fabric.health` hop hits it.
- [ ] 2.3 Generous `start_period` on healthchecks (fabric is the heaviest); if a dep OOMs on the 12 GiB VM, record it as a real finding.
