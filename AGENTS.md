# AGENTS.md

The canonical guidance for this repository lives in **[CLAUDE.md](./CLAUDE.md)**.

All agents and harnesses (Codex, OpenCode, Gemini CLI, Roo, Cline, Kilo Code,
Librefang, and any other Prometheus/UAR-compatible agent) must read and follow
`CLAUDE.md`. It contains the project overview, architecture, build commands,
quality gates, and the full Prometheus Base Rules Set (Rules 1–40).

## Working Philosophy (Fast Iteration — Implementation-First)

See `CLAUDE.md` → **"Fast Iteration Workflow"** for the full rules. In short:

- **Implement the whole plan first, then test.** Pieces only have meaning once the
  system fits together — get ALL the code done and connected (no gaps, no
  unimplemented important pieces), *then* write integration tests around the proven
  shape of the code.
- **Prefer full integration tests of whole sections** over unit tests that validate
  nothing important. Err toward more code implemented correctly.
- **Wait for the test suite ≤ 3 times per goal/epoch** (one `cargo test` run each);
  compile-checks don't count.
- **Compile only when necessary** — Rust builds are expensive. Use `cargo check`
  sparingly at section boundaries (batch changes), reserve `cargo test` for the ≤3
  integration milestones, and `--release` only for production at the end.
- Dev compile-time settings (`[profile.dev]`, `ld64.lld`/mold, `sccache`) are
  configured and must stay on.

→ See [CLAUDE.md](./CLAUDE.md).
