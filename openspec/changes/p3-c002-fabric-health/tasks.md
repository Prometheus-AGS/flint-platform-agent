## 1. Implement fabric health

- [ ] 1.1 Add `reqwest` (workspace) to `fpa-fabric`; give `FabricAdapter` a `reqwest::Client`.
- [ ] 1.2 Add `wiremock` 0.6 as a dev-dependency.
- [ ] 1.3 `health()` ← `GET {endpoint}/healthz`: 2xx → `Ok(())`, else `Downstream`, unreachable → `Transport`.

## 2. Verification (wiremock)

- [ ] 2.1 `cargo check/clippy/fmt` green.
- [ ] 2.2 wiremock: 200 `/healthz` → `Ok(())`.
- [ ] 2.3 wiremock: 503 → `Downstream`; unreachable endpoint → `Transport`.
- [ ] 2.4 Confirm `fabric.health` task now returns `{"ok":true}` end-to-end (runner already maps `health() -> ok`).
