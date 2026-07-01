### 2026-07-01T20:40:27Z — Analyze: minimal adopt (reuse-heavy)
Core goals reuse existing code/deps: writes via c002 graphql_query (map 403→Unauthorized);
fabric health via reqwest GET /healthz. Only new-dep question = WS client for the
stretch subscription: tokio-tungstenite (house standard — fabric + gate both use it;
siblings 0.26, latest 0.29). Recommend defer WS. Write-kind guard + test hardening = BUILD.
Open for spec: Q1 build-vs-proxy mutations (recommend build), Q2 collection names, Q3 WS scope.
