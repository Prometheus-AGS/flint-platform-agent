#!/usr/bin/env bash
# One-command REAL smoke: agent + real gate + real forge PG + real fabric.
#
# Brings up smoke/compose.real.yml, waits for every plane's health, polls the agent's
# /healthz, runs the Playwright HTTP smoke (incl. the realtime-event assertion), then
# tears the stack down (down -v) on success AND failure.
#
# The forge GATEWAY is off by default (forge-full profile) — blocked on
# Know-Me-Tools/flint-forge#7 (dup migration versions). The default run converges on the
# planes that boot: agent + real gate + real fabric + real forge PG/bootstrap. Pass
# --forge-full to also build+run the forge gateway (only useful once #7 is fixed).
set -euo pipefail

cd "$(dirname "$0")"

# colima docker socket (testcontainers/compose need this; see memory).
export DOCKER_HOST="${DOCKER_HOST:-unix:///Users/gqadonis/.colima/default/docker.sock}"

COMPOSE=(docker compose -f compose.real.yml)
PROFILE_ARGS=()
NO_BUILD=""
for arg in "$@"; do
  case "$arg" in
    --forge-full)
      PROFILE_ARGS=(--profile forge-full)
      echo "==> forge-full profile ON (forge gateway included — needs flint-forge#7 fixed)" ;;
    --no-build)
      # Boot pre-built images (no recompile). This is the RELIABLE path once images
      # exist: the 12 GiB VM handles the RUNNING 8-service stack fine (~60s to healthy);
      # it's the concurrent BUILDS that OOM it. Rebuild explicitly only when source
      # changed: `docker compose -f compose.real.yml build <svc>` (one at a time).
      NO_BUILD="1"
      echo "==> --no-build: booting pre-built images (skip the compile step)" ;;
  esac
done

cleanup() {
  echo "==> tearing down (down -v)"
  "${COMPOSE[@]}" "${PROFILE_ARGS[@]}" down -v --remove-orphans || true
}
trap cleanup EXIT

# Build images SERIALLY, not concurrently. `up --build` compiles every buildable service
# in parallel — agent + fabric-gateway (+ gate + forge-PG) are large Rust/node builds, and
# running them together OOM-kills the 12 GiB VM (observed). Build one at a time (deps are
# cached across them), then `up` reuses the images. This is the wave-bringup the c005 spec
# calls for. --parallel 1 keeps buildkit from fanning out within a single build too.
echo "==> building images serially (avoids the concurrent-compile OOM on 12 GiB)"
# Build one service at a time so at most ONE heavy Rust/node compile is ever in flight
# (concurrent builds OOM-kill the 12 GiB VM). Cached layers make repeats fast. The
# default run is agent + gate + fabric (the planes that serve the agent's HTTP hops);
# forge services are forge-full-only (blocked on flint-forge#7).
BUILD_SVCS=(flint-gate fabric-gateway agent)
for arg in "$@"; do [[ "$arg" == "--forge-full" ]] && BUILD_SVCS=(forge-postgres flint-gate fabric-gateway agent); done
[[ -n "$NO_BUILD" ]] && BUILD_SVCS=()
for svc in "${BUILD_SVCS[@]}"; do
  echo "    build: $svc"
  "${COMPOSE[@]}" "${PROFILE_ARGS[@]}" build "$svc" || {
    echo "!! build failed for $svc"; exit 1;
  }
done

echo "==> starting the real stack (images pre-built; --wait for plane health)"
"${COMPOSE[@]}" "${PROFILE_ARGS[@]}" up --wait --wait-timeout 600

echo "==> waiting for the agent /healthz on :8088"
for i in $(seq 1 30); do
  if curl -sf http://localhost:8088/healthz >/dev/null 2>&1; then
    echo "    agent healthy"
    break
  fi
  [[ $i -eq 30 ]] && { echo "!! agent did not become healthy"; docker logs fpa-smoke-real-agent-1 2>&1 | tail -30; exit 1; }
  sleep 2
done

echo "==> running the real smoke (Playwright HTTP)"
if [[ ! -d node_modules ]]; then
  echo "    installing smoke deps"
  npm install --silent
fi
SMOKE_BASE_URL="http://localhost:8088" \
  FABRIC_BASE_URL="http://localhost:28080" \
  npx playwright test smoke.real.spec.ts

echo "OK: real smoke passed"
