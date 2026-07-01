# Analysis — forge-integration-and-real-dispatch

**Phase:** forge-integration-and-real-dispatch
**Date:** 2026-06-30
**Mode:** stack-specified (Rust/Axum + reqwest + jsonwebtoken, integrating flint-forge Quarry)
**Inputs:** `assessment.md`.

> Light analyze phase: this is integration against a **known contract**, so
> there's almost no build-vs-adopt surface. The only real external question was
> the test-mock crate. The substantive work (credential threading, task store)
> is internal design, deferred to spec.

---

## 1. Landscape summary

Nothing new needs adopting at runtime. `fpa-forge` calls forge's Quarry over
HTTP with **`reqwest` (already in the workspace)**; the GraphQL body is standard
`{query, variables, operationName}` built with `serde_json` — a typed
`graphql_client` would need schema codegen and is overkill for a few read
queries. The gate-JWT→forge-bearer→RLS path uses `jsonwebtoken`/raw-token
forwarding already present from c001/c003.

The one genuine research question — **how to test `fpa-forge` without a live,
Postgres-backed forge (assessment Q1)** — resolves cleanly on a **consistency
signal**: `wiremock` 0.6 is already used by **flint-gate** to test its reqwest
HTTP client, which is the closest sibling to `fpa-forge`. It's async/tokio-native
(our tests are `#[tokio::test]`). Adopt it as a dev-dependency. (`httpmock`, used
by fabric's Ory adapters, is a viable alternative but wiremock matches the gate
client-testing precedent.)

## 2. Build-vs-adopt calls

| Gap | Verdict | Confidence |
|---|---|---|
| HTTP client (fpa-forge) | **reqwest** (already present) | high |
| GraphQL body | **build with serde_json** (no graphql_client) | high |
| Test HTTP mocking | **adopt `wiremock` 0.6** (dev-dep; gate precedent) | high |
| Credential threading | **build** (AuthContext.bearer vs request-context — spec) | medium |
| In-memory task store | **build** (`RwLock<HashMap>`, no crate) | high |

## 3. The one design decision that matters (for spec)

**Credential threading is the crux, and it's a BUILD/design call, not a library
choice.** Forwarding the operator's gate bearer to forge means a per-request
credential must flow through the ports. Two shapes:

- **(A) Add `bearer: Option<String>` to `fpa-app::AuthContext`** — smallest
  change; the runner already receives `AuthContext`, so the forge port method
  gains a bearer parameter (or reads it from the context passed down).
- **(B) A dedicated request-context object** threaded separately — cleaner
  separation of "who" (auth) from "how to call downstream" (credential), but more
  plumbing.

Recommendation: **(A)** for this phase (YAGNI — one credential, one downstream
that needs it), with a note that (B) is the refactor if more downstreams need
per-request secrets. **Decide at spec.** Non-negotiable regardless: never log the
bearer (extend the c003 audit-skip to the forge adapter).

## 4. Evidence (tiered)

- **Tier 1 (gh search):** skipped — no framework/skeleton discovery needed for a
  known-contract integration.
- **Tier 3 (cargo search):** `wiremock` 0.6.5, `httpmock` 0.8.3, `mockito` 1.7.2
  all current.
- **Tier 3 (sibling inspection):** `wiremock` used by `flint-gate-client` +
  `flint-gate-core`; `httpmock` used by fabric `frf-identity-ory` +
  `frf-authz-keto`. → wiremock aligns with the gate HTTP-client precedent.
- **Tiers 2/4:** not needed.

## 5. Open questions (carried to spec)

1. **Q1 (local vs fixtures):** ✅ resolved toward **fixtures via wiremock** for
   unit/integration tests. A live-forge smoke test remains optional (needs
   Postgres 18 + a seeded schema) — treat as manual, not CI, this phase.
2. **Q2 (credential threading shape):** `AuthContext.bearer` (recommended) vs
   request-context. **Decide at spec.**
3. **Q3 (metadata source):** `GET /openapi.json` (public, pre-compiled) for
   `list_tables`; GraphQL for data. Recommend OpenAPI for the table list,
   GraphQL introspection only if `describe_table` needs more than OpenAPI carries.
4. **Q4 (MCP caller identity):** unchanged — does `POST /mcp` carry a gate JWT?
   Gates the MCP-identity debt. Still open; confirm at spec.

## 6. Handoff to Spec

Adopt set is tiny: **reqwest** (present) + **wiremock 0.6** (dev-dep). Everything
else is internal BUILD: the credential-threading design (Q2 — recommend
`AuthContext.bearer`), the serde_json GraphQL body, and the `RwLock<HashMap>` task
store. Spec must decide Q2 (threading shape), Q3 (OpenAPI vs GraphQL for
describe), and Q4 (MCP identity), and should keep the phase **read-only**.
