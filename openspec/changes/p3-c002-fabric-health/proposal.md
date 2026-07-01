## Why

`fpa-fabric::health` returns `PortError::Downstream("not implemented")`, so
`fabric.health` never reports real liveness. The realtime fabric gateway serves
`GET /healthz` → `{status,version}`. This change implements the adapter against it
(closes carried debt #5, fabric half).

## What Changes

- Give `FabricAdapter` a `reqwest` client; `health()` calls `GET {endpoint}/healthz`.
- 2xx → `Ok(())`; non-2xx → `PortError::Downstream`; unreachable → `PortError::Transport`.
- No bearer required (health is public, like forge `/healthz`).

## Capabilities

### New Capabilities
- `fabric-health`: Real realtime-fabric liveness check via `GET /healthz`.

### Modified Capabilities

## Impact

- `fpa-fabric` (reqwest client + real `health()`); add `reqwest` + `wiremock` dev-dep.
- No new runtime deps beyond `reqwest` (present in the workspace).
- Independent of the forge write change.
