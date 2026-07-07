#!/usr/bin/env bash
#
# run.sh — one-command live smoke for the Flint Platform Agent.
#
# Brings the compose stack up (agent + postgres + wiremock stub), waits for the agent
# on :8088, installs + runs the Playwright HTTP smoke, and tears the stack down on
# exit (success OR failure).
#
# Prereq: healthy Docker (source-built colima+lima). If docker hangs, recover with
# ../scripts/reset-colima.sh. Usage:  ./smoke/run.sh
#
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
COMPOSE="$HERE/compose.smoke.yml"
BASE_URL="${FPA_SMOKE_BASE_URL:-http://localhost:8088}"
# MUST match compose.smoke.yml's FPA_GATE_JWT_KEY so the minted bearer verifies.
export FPA_GATE_JWT_KEY="${FPA_GATE_JWT_KEY:-smoke-hs256-secret-not-a-real-credential}"

say() { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
die() { printf '\033[1;31m[fail] %s\033[0m\n' "$*" >&2; exit 1; }

teardown() { say "Tearing down"; docker compose -f "$COMPOSE" down -v >/dev/null 2>&1 || true; }
trap teardown EXIT

command -v docker >/dev/null 2>&1 || die "docker not found"
docker info >/dev/null 2>&1 || die "Docker not reachable — run ../scripts/reset-colima.sh"

say "Bringing up the smoke stack (build if needed)"
docker compose -f "$COMPOSE" up --build -d || die "compose up failed"

say "Waiting for the agent on ${BASE_URL}/healthz"
ok=""
for i in $(seq 1 40); do
  if [ "$(curl -s -o /dev/null -w '%{http_code}' --max-time 3 "$BASE_URL/healthz" 2>/dev/null)" = "200" ]; then
    ok=1; echo "  healthz 200 (attempt $i)"; break
  fi
  sleep 3
done
[ -n "$ok" ] || { docker compose -f "$COMPOSE" logs agent | tail -30; die "agent never became healthy on :8088"; }

say "Installing Playwright smoke deps (smoke/)"
# HTTP-only smoke (Playwright `request` API) — NO browser needed, so we do NOT run
# `playwright install` (that download is slow and pointless here).
( cd "$HERE" && npm install --silent ) || die "npm install failed"

say "Running the Playwright smoke against the live agent"
# Pass smoke.spec.ts explicitly so the stub runner never accidentally picks up
# smoke.real.spec.ts (which needs the real-sibling stack).
if ( cd "$HERE" && npx playwright test smoke.spec.ts ); then
  printf '\n\033[1;32m✔ LIVE SMOKE PASSED against the containerized agent.\033[0m\n'
else
  die "live smoke FAILED — see the Playwright output above"
fi
