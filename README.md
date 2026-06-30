# Flint Platform Agent

The **administrative agent for the entire Prometheus Flint fabric**. A single
Axum server that exposes the platform's administrative and operational
capabilities through open agent protocols, so an operator can manage and exercise
the fabric from **any** protocol-compatible harness (Claude Desktop, Claude Code,
OpenCode, Codex, custom Tauri/CLI harnesses, etc.).

> **Status: scaffold.** The hexagonal workspace, protocol surfaces, and CI are in
> place; handler bodies are stubbed with `todo!()`. See [`CLAUDE.md`](./CLAUDE.md)
> for the authoritative architecture and contribution rules.

## Protocol Surfaces

The agent speaks four open standards from one binary:

| Surface | Role |
|---|---|
| **AG-UI** | Agent → UI event stream (run lifecycle, text deltas, tool calls, state) |
| **A2A** (Agent2Agent) | Administrative **task** lifecycle (submit / status / cancel) |
| **A2UI** | Dynamic UI primitives that render in any host and bind to fabric functions |
| **MCP** | App (UI surfacing), client (downstream servers), and server (HTTP-Streaming only) |

It administers the three sibling planes of the fabric:

- **[flint-forge](https://github.com/Prometheus-AGS/flint-forge)** — sovereign data & edge-compute plane (source of fabric metadata/data).
- **[flint-realtime-fabric](https://github.com/Prometheus-AGS/flint-realtime-fabric)** — realtime spine; owns the canonical agent-protocol types the agent reuses.
- **[flint-gate](https://github.com/Prometheus-AGS/flint-gate)** — AI-native auth proxy / API gateway.

## Architecture

A strict hexagonal (ports & adapters) Rust workspace. The dependency rule is
enforced at the Cargo level — **domain and app layers never import an adapter**;
composition happens only in the interface crates.

```
fpa-domain      Layer 0 — pure types (zero infra deps)
fpa-protocol    AG-UI / A2A / A2UI / MCP wire types
fpa-ports       Layer 1 — trait seams (one port per plane)
fpa-app         Layer 1 — use-cases against ports only
fpa-forge       adapter → ForgeMetadata   (Quarry metadata/data)
fpa-fabric      adapter → FabricClient     (realtime spine)
fpa-gate        adapter → GateAdmin        (gate admin API)
fpa-mcp         adapter → McpClient        (downstream MCP client)
fpa-gateway     Axum composition root (bin) — the ONLY crate importing adapters
fpa-cli         ops / dev CLI (bin `fpa`)
```

## Build & Run

Requires the toolchain pinned in [`rust-toolchain.toml`](./rust-toolchain.toml)
(channel 1.93; MSRV 1.93; edition 2024).

```bash
cargo check --workspace                                 # type-check
cargo run -p fpa-gateway                                # run the gateway
cargo test --workspace                                  # tests
cargo clippy --workspace --all-targets -- -D warnings   # lint gate (pedantic)
cargo fmt --all                                         # format
./scripts/ci-check.sh                                   # full local CI gate
```

The gateway binds `0.0.0.0:8088` by default; override with `FPA_GATEWAY_ADDR`:

```bash
FPA_GATEWAY_ADDR=127.0.0.1:9090 cargo run -p fpa-gateway
curl localhost:9090/healthz                             # -> ok
```

### Endpoints (stubbed)

| Method | Path | Surface |
|---|---|---|
| `GET` | `/healthz` | liveness |
| `GET` | `/agui/stream` | AG-UI Server-Sent Events |
| `POST` | `/a2a/tasks` | submit an administrative task |
| `GET` | `/a2a/tasks/{id}` | task status |
| `POST` | `/a2a/tasks/{id}/cancel` | cancel a task |
| `POST` | `/mcp` | MCP JSON-RPC 2.0 (HTTP-Streaming) |

## Development Workflow

This repo uses **OpenSpec** (spec-driven changes) and the **KBD** orchestrator.
Both states are version-controlled so they travel across machines.

- OpenSpec is configured for Codex, Claude Code, Kimi, and OpenCode
  (`/opsx:*` commands; specs under `openspec/`).
- KBD configuration lives in `.kbd-orchestrator/`.

## Contributing

Read [`CLAUDE.md`](./CLAUDE.md) first — it carries the architecture, quality
gates (no `unwrap`/`expect` in libs, `clippy::pedantic -D warnings`, 500-line
file limit), and the full Prometheus Base Rules. [`AGENTS.md`](./AGENTS.md)
points all non-Claude harnesses to the same guidance.

## License

Proprietary — Prometheus AGS.
