## 1. Runner script

- [ ] 1.1 Add `smoke/run-durable-proof.sh` (executable): resolve `DOCKER_HOST` from `docker context inspect colima --format '{{.Endpoints.docker.Host}}'`; export it + `TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE=${DOCKER_HOST#unix://}`.
- [ ] 1.2 Pre-flight: `docker info` reachable? If not, die with a pointer to `scripts/reset-colima.sh`.
- [ ] 1.3 Run `cargo test -p fpa-store-pg -- --ignored --nocapture`; propagate the exit code.

## 2. Verification

- [ ] 2.1 `bash -n smoke/run-durable-proof.sh` (syntax) + `chmod +x`.
- [ ] 2.2 Run it: the durability test executes against a real Postgres and PASSES (record the pass in the change/reflection — this is the live proof).
