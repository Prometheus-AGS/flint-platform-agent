#!/usr/bin/env bash
#
# run-durable-proof.sh — prove the durable ProjectStore against a REAL Postgres.
#
# Runs the `fpa-store-pg` `#[ignore]`d durability test (put -> fresh pool -> get ->
# list -> restart-survival) against an ephemeral Postgres spun by the `testcontainers`
# crate. testcontainers resolves the Docker daemon via DOCKER_HOST / the standard
# socket — NOT the docker CLI *context* — so on colima we must export DOCKER_HOST
# pointed at colima's socket, or it fails with "Connection refused".
#
# Usage:  ./smoke/run-durable-proof.sh
#
set -uo pipefail

die() { printf '\033[1;31m[fail] %s\033[0m\n' "$*" >&2; exit 1; }
say() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }

command -v docker >/dev/null 2>&1 || die "docker CLI not found on PATH"

# Resolve the Docker endpoint from the active colima context (fall back to whatever
# the current context reports if 'colima' isn't the named context).
DOCKER_ENDPOINT=$(docker context inspect colima --format '{{.Endpoints.docker.Host}}' 2>/dev/null)
[ -n "$DOCKER_ENDPOINT" ] || DOCKER_ENDPOINT=$(docker context inspect "$(docker context show)" --format '{{.Endpoints.docker.Host}}' 2>/dev/null)
[ -n "$DOCKER_ENDPOINT" ] || die "could not resolve a docker endpoint from the CLI context"

export DOCKER_HOST="$DOCKER_ENDPOINT"
# The reaper/mount path override wants the raw socket path (no unix:// scheme).
export TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE="${DOCKER_ENDPOINT#unix://}"

say "Docker endpoint: $DOCKER_HOST"

# Pre-flight: is the daemon actually serving? (bounded — never hang)
timeout 15 docker info >/dev/null 2>&1 \
  || die "Docker daemon not reachable at $DOCKER_HOST. Recover with: ./scripts/reset-colima.sh"

say "Running the fpa-store-pg durability proof against a real Postgres"
# The test is #[ignore]d by default; --ignored runs it. --nocapture shows the flow.
cargo test -p fpa-store-pg -- --ignored --nocapture
rc=$?

if [ "$rc" -eq 0 ]; then
  printf '\n\033[1;32m✔ Durable-store proof PASSED against a real Postgres.\033[0m\n'
else
  die "durability proof FAILED (exit $rc)"
fi
