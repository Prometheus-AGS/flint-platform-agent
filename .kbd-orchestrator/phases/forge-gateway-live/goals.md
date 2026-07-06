# Goals

- Boot the REAL forge Quarry gateway (fdb-gateway) in the smoke once flint-forge#7 is fixed (dup migration versions + .dockerignore) — the authored fdb-gateway Dockerfile + roles/flint_meta bootstrap are ready; flip --forge-full on and prove the agent's forge-read hops (openapi.json, /{schema}/{table}, /graphql) against the real gateway.
- Wire smoke/run-real.sh into CI (the deferred no-infra Option B) so the real-sibling smoke guards regressions — decide GH Actions vs a make target; keep the --no-build fast path.
- Re-verify flint-forge#7 status first (check if forge fixed the pins/migrations/.dockerignore); if still open, either help land the forge fixes in a separate session or keep forge gateway gated and focus this phase on CI wiring + a real realtime-receipt proof via a dev IdP.
