# KBD Constraints — Flint Platform Agent

Project-specific blocking and warning rules consumed by the KBD orchestrator.
Derived from `CLAUDE.md` (quality gates + Prometheus Base Rules 1–40). When a
proposed change violates a **BLOCKING** rule, the orchestrator must halt and
surface it before proceeding.

## Blocking Constraints (halt on violation)

- **No `unwrap()` / `expect()` in library crates.** Use `thiserror` in libs;
  `anyhow` only at binary entry points (`fpa-gateway`, `fpa-cli`). _(machine-checkable: grep `\.unwrap()` / `\.expect(` in `crates/*/src` excluding the two bins)_
- **Clippy gate must stay green:** `cargo clippy --workspace --all-targets -- -D warnings` (pedantic). A new warning is a failure, not a nit.
- **rustfmt must be clean:** `cargo fmt --all --check` passes.
- **No file over 500 lines.** Split into a directory module when approaching the limit.
- **Never log** JWTs, claims, relation tuples, tenant identifiers, or secrets.
- **Hexagonal dependency rule is absolute.** `fpa-domain` and `fpa-app` must NOT
  depend on any adapter crate (`fpa-forge`, `fpa-fabric`, `fpa-gate`, `fpa-mcp`)
  or on `fpa-gateway`/`fpa-cli`. Composition happens only in interface crates. _(machine-checkable: inspect `[dependencies]` of `fpa-domain` and `fpa-app`)_
- **MCP server surface is HTTP-Streaming only.** Do not add a stdio transport to the agent's MCP server role.
- **Public enums are `#[non_exhaustive]`; identifiers are `#[repr(transparent)]` newtypes.**
- **No hardcoded secrets.** Use environment variables; validate presence at startup.
- **Edition is `2024` by design.** Do not downgrade the workspace to 2021 to "match" siblings (they are pending migration).

## Warning Constraints (flag, do not necessarily halt)

- **Tests are part of completion.** New behavior ships with tests; run `cargo test --workspace`.
- **`tracing` spans across every port boundary.**
- **Reuse the shared protocol contract** (`frf-agentproto::ContentBlock` shape:
  tagged enums + `#[serde(other)] Unknown`). Do not invent ad-hoc protocol or
  A2UI schemas — extend the canonical primitive set.
- **Verify dependency versions before adding them** (Base Rules 22–23, 27); prefer existing workspace deps.
- **Small, reviewable changes.** Separate mechanical from behavioral changes.
- **Agent actions must be auditable**; human override must always exist (Base Rules 25, 34).

## Verification Commands

| Purpose | Command |
|---|---|
| Build health | `cargo check --workspace` |
| Lint gate | `cargo clippy --workspace --all-targets -- -D warnings` |
| Format check | `cargo fmt --all --check` |
| Tests | `cargo test --workspace` |
| Full CI gate | `./scripts/ci-check.sh` |
| MSRV floor | `cargo +1.93 check --workspace` |

> Full rationale for every rule lives in `CLAUDE.md`. This file is the
> machine-actionable subset the orchestrator enforces.
