# Plan — integration-proof-and-debt-closure

**Phase:** integration-proof-and-debt-closure
**Backend:** OpenSpec (`openspec/changes/p5-c00N-*`)
**Changes:** 4 (all validate `--strict`)
**Working philosophy:** implementation-first — implement c001→c003 fully, ONE
batched `cargo check` at the section boundary, then c004 lands the single justified
`cargo test` integration milestone.

---

## Ordered change list

### 1. p5-c001-project-store  (G2) — **first**
Agent-owned `ProjectStore` port + in-memory adapter; `project.create` writes the
nested aggregate to the store, not forge.
- **Why first:** unblocks the integration proof's `project.create` hop; it's the
  operator-decided persistence model everything else assumes. Touches `fpa-ports`,
  `fpa-app`, `fpa-gateway::state`.
- **Depends on:** nothing.
- **Recommended agent:** `rust-reviewer` after implementation (port + adapter is
  core-domain-adjacent; wants a trait-boundary review).

### 2. p5-c002-forge-rest-path-fix  (G1) — **second**
Correct REST insert to `POST /{schema}/{table}` (no `/rest` prefix, schema-qualified).
- **Why here:** independent of c001; a correctness fix to `fpa-forge` +
  `fpa-gateway::config`. Sequenced after c001 only because c001 removes the
  `project.create → forge` call, so c002's caller-update surface is smaller/cleaner.
- **Depends on:** c001 (soft — c001 removes the sole `project.create` forge-write
  caller, avoiding a churn conflict on `create_entity`'s signature).
- **Recommended agent:** `rust-reviewer`; the path/contract is source-verified so
  review focus is the signature change + status mapping.

### 3. p5-c003-security-debt-closure  (G3–G6) — **third**
Gate write-guard refuse, `/agui/stream` auth, JWKS single-flight, `signature_verified`
in the task audit.
- **Why here:** independent of c001/c002; grouped security fixes. Sequenced third so
  the whole production surface (store + forge path + auth/guard) is complete before
  the proof.
- **Depends on:** nothing hard (touches `fpa-gateway` agui/jwks/identity + `fpa-app`
  task_runner/AuthContext).
- **Recommended agent:** **`security-reviewer`** (mandatory — auth/crypto/access-
  control surface; same discipline that caught 2 CRITICALs in phase 4).

--- SECTION BOUNDARY: one batched `cargo check` across the workspace here ---

### 4. p5-c004-integration-proof  (G7) — **last**
End-to-end test: authenticate → project.create → list_routes → fabric.health across
AG-UI/A2A/MCP; planes mocked at HTTP boundary + real `ProjectStore` + in-test RSA JWKS.
- **Why last:** proves the *whole shape* — requires c001 (store), c002 (forge path),
  c003 (auth/guard) all present. This is the phase's ONE justified `cargo test`.
- **Depends on:** c001, c002, c003 (hard — all three).
- **Recommended agent:** `tdd-guide` / general — integration-test authoring; verify
  the ephemeral-key crate version (Base Rule 22) before adding it.

---

## Dependency graph

```
c001 ─┐
c002 ─┤ (c002 soft-after c001)
c003 ─┤
       └─► c004  (hard: needs c001+c002+c003)
```

## Execution contract (per implementation-first philosophy)

- Implement **c001 → c002 → c003 in full** (all production code) before the batched
  `cargo check`. Do NOT `cargo check`/`test` per-change.
- One `cargo check` at the section boundary (after c003).
- **c004** authors the integration test, then **one full `cargo test`** — the first
  of the ≤3 test-wait budget for this goal. Fix to green.
- Reserve the remaining test-waits for real integration re-runs only.

## Compliance notes

- No new production dependencies (c001–c003). c004 may add ONE dev-dep for the
  ephemeral RSA keypair — verify current version first (Base Rule 22, 27).
- Hexagonal boundary preserved: `ProjectStore` port in `fpa-ports` (domain-only
  import); adapter + wiring in `fpa-app`/`fpa-gateway` (composition root).
- No `unwrap`/`expect` in libs; `#[non_exhaustive]` on any new public enum; secrets
  never logged (c003 G6 audit must not log token/claims).

## Handoff to execute

4 validated OpenSpec changes, ordered c001→c002→c003 (production) then c004
(integration proof). Ordering is dependency-driven: c004 hard-depends on the other
three; c002 soft-follows c001 to avoid a `create_entity` signature-churn conflict.
First change to apply: **p5-c001-project-store**. Execute per implementation-first:
build c001–c003, one batched check, then c004's single justified `cargo test`.
