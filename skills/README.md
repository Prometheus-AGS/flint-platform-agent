# Agent Skills — MCP Tool Convention

Skills document the administrative capabilities this agent exposes as **MCP
tools**. Each MCP tool is backed 1:1 by an entry in the A2A task catalog
(`crates/fpa-app/src/catalog.rs`), so a skill is the human/agent-facing
description of one catalog task kind.

## Format

Each skill is a directory under `skills/<skill-name>/` containing a `SKILL.md`
with YAML front-matter, mirroring the Prometheus/Anthropic skill convention:

```markdown
---
name: <skill-name>
kind: <catalog task kind, e.g. forge.table.list>
required_role: <role the operator must hold>
description: <one line — what it does>
---

<body: when to use it, inputs, example>
```

## Contract

- `kind` MUST match a `CatalogEntry.kind` in `fpa-app::catalog::CATALOG`.
  A skill whose `kind` is not catalogued is invalid — the MCP `tools/list`
  surface is generated from the catalog, not from these files, so a stale skill
  simply won't appear as a callable tool.
- `required_role` is documentation of the catalog's enforced role; the runtime
  authority is `CatalogEntry.required_role`, enforced by `TaskRunner` before any
  port call.
- Skills are descriptive, not authoritative: the catalog is the source of truth.
  Keep them in sync (a future check can assert every `kind` here exists in the
  catalog).

## Invocation path

```
MCP host → tools/call { name: <kind>, arguments } → fpa-gateway routes/mcp.rs
        → TaskRunner (permission check → port dispatch → audit)
        → tool result (content: [{ type: text, text }])
```
