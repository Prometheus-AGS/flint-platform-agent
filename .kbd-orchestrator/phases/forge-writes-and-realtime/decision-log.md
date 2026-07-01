### 2026-07-01T20:40:27Z — Analyze: minimal adopt (reuse-heavy)
Core goals reuse existing code/deps: writes via c002 graphql_query (map 403→Unauthorized);
fabric health via reqwest GET /healthz. Only new-dep question = WS client for the
stretch subscription: tokio-tungstenite (house standard — fabric + gate both use it;
siblings 0.26, latest 0.29). Recommend defer WS. Write-kind guard + test hardening = BUILD.
Open for spec: Q1 build-vs-proxy mutations (recommend build), Q2 collection names, Q3 WS scope.

### 2026-07-01T20:44:46Z — Spec: 3 changes (followed analyze recommendation)
Per user "follow your recommendation": build typed mutations (Q1), DEFER WS (Q3).
- p3-c001-forge-write-mutations  writes via typed pg_graphql mutations + write-kind guard (debt #1); 403→Unauthorized
- p3-c002-fabric-health          real fpa-fabric::health via GET /healthz (debt #5)
- p3-c003-test-hardening          close phase-2 test-coverage gaps (debt #2); test-only capability spec
WS subscription NOT specced (deferred to a later realtime phase per recommendation).
All 3 pass openspec validate. Deps: c001,c002,c003 all independent.

### 2026-07-01T20:46:44Z — Plan: ordered 3 (all independent)
Order by value/risk (no deps): c001 writes → c002 health → c003 test-hardening.
First: p3-c001-forge-write-mutations. WS deferred. Waypoint refreshed.
