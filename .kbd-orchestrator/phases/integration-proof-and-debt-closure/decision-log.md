# Decision Log — integration-proof-and-debt-closure

### 2026-07-03 — Project persistence model (G2)
Question: Where does the nested Project aggregate persist? (forge has no projects
table; forge REST insert is flat-row POST /{schema}/{table}).
Options: agent-owned ProjectStore port | forge flint_meta.projects JSONB | prove
wiring with existing table.
Decision: **Agent-owned ProjectStore port** (in-memory now, durable later).
Provenance: **user** (AskUserQuestion, this session).
Rationale: nested aggregate fits poorly in a flat forge row; TaskStore precedent;
unblocks the integration proof with no cross-repo forge migration; G1 REST-path fix
still lands for correctness, decoupled from project.create.

### 2026-07-03 — Integration-proof approach (assess open-Q2)
Decision: mock each plane at the HTTP boundary (wiremock) for the first green proof;
project.create hits the REAL in-memory ProjectStore. Live smoke against real
siblings is a documented follow-on. Provenance: research/implicit (forge has no
ready compose; fabric/gate do but a first proof shouldn't gate on 3 live services).

### 2026-07-03 — JWKS test-key approach (assess open-Q3)
Decision: mint an ephemeral RSA keypair in-test, serve its public JWK via wiremock
as the IdP. Deterministic verify + single-flight proof without a live Kratos/Hydra.
Real IdP behind FPA_JWKS_URL stays deployment config. Provenance: research/implicit.

### 2026-07-03 — External research tiers
Decision: Tiers 1–4 NOT run (justified skip). Every gap is in-house Rust or a
port+in-memory adapter following the existing TaskStore pattern; no external library
is a candidate. Confidence: high (sibling contracts read from source this phase).
