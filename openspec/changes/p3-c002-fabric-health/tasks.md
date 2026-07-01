## 1. Implement fabric health

- [x] 1.1 Add `reqwest` (workspace) to `fpa-fabric`; give `FabricAdapter` a `reqwest::Client`.
- [x] 1.2 Add `wiremock` 0.6 as a dev-dependency.
- [x] 1.3 `health()` ← `GET {endpoint}/healthz`: 2xx → `Ok(())`, else `Downstream`, unreachable → `Transport`.

## 2. Verification (wiremock)

- [x] 2.1 `cargo check/clippy/fmt` green.
- [x] 2.2 wiremock: 200 `/healthz` → `Ok(())`.
- [x] 2.3 wiremock: 503 → `Downstream`; unreachable endpoint → `Transport`.
- [x] 2.4 Confirm `fabric.health` task now returns `{"ok":true}` end-to-end (runner already maps `health() -> ok`).
