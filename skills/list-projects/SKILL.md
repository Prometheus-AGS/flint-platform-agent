---
name: list-projects
kind: project.list
required_role: viewer
description: List the projects visible to the operator.
---

# List Projects

Lists fabric projects the operator can see. Backed by the `project.list` catalog
task, which dispatches to the forge port (`ForgeMetadata::list_tables` until
forge's project API ships).

## When to use

- The operator asks "what projects exist" / "show my projects".
- A UI needs to populate a project picker.

## Inputs

None required this phase. (Filtering/pagination arguments land with forge's
project API.)

## Example (MCP `tools/call`)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": { "name": "project.list", "arguments": {} }
}
```

Returns a `content` array with the project list as text. Requires the `viewer`
role; the runner denies the call otherwise.
