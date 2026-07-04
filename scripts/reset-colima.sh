#!/usr/bin/env bash
#
# reset-colima.sh — tear down a wedged Docker setup and bring up ONE clean colima
# VM sized to build/run two heavy (pgrx/Rust) smoke stacks concurrently.
#
# Why: Docker Desktop + colima were both running and racing over the docker socket,
# which wedged the daemon (API returns EOF) and froze the CLI. This script makes
# colima the single Docker engine, recreates its VM clean + large, points the CLI
# at it, and verifies the daemon actually serves.
#
# Safe to re-run. Tune CPU/MEM/DISK below for your host before running.
#
# Usage:
#   chmod +x reset-colima.sh
#   ./reset-colima.sh
#
set -uo pipefail   # NOT -e: we want to continue past best-effort cleanup failures

# ---- sizing (edit for your host) --------------------------------------------
# 6 CPU / 12 GiB is enough for TWO concurrent pgrx image builds + Postgres + a few
# service containers. Drop to --memory 10 if the host has only 16 GiB total.
CPUS="${COLIMA_CPUS:-6}"
MEM="${COLIMA_MEM:-12}"     # GiB
DISK="${COLIMA_DISK:-120}"  # GiB
PROFILE="${COLIMA_PROFILE:-default}"

say()  { printf '\n\033[1;36m==> %s\033[0m\n' "$*"; }
warn() { printf '\033[1;33m[warn] %s\033[0m\n' "$*"; }
die()  { printf '\033[1;31m[fail] %s\033[0m\n' "$*" >&2; exit 1; }

command -v colima >/dev/null 2>&1 || die "colima not found on PATH"
command -v docker >/dev/null 2>&1 || die "docker CLI not found on PATH"

# ---- 1. kill any frozen docker/colima CLI invocations ------------------------
say "Killing any frozen docker/colima CLI commands (not daemons)"
# Match stuck client calls; leave the actual daemons/VM helpers alone for now.
pkill -9 -f 'docker (ps|version|run|info|build|compose)' 2>/dev/null && echo "  cleared hung docker CLI calls" || echo "  none found"
pkill -9 -f 'colima (restart|start|stop) *$'            2>/dev/null && echo "  cleared wedged colima subcommand" || echo "  none found"

# ---- 2. quit Docker Desktop (the competing engine) ---------------------------
say "Quitting Docker Desktop so colima is the sole Docker engine"
if pgrep -f 'Docker.app' >/dev/null 2>&1; then
  osascript -e 'quit app "Docker Desktop"' 2>/dev/null && echo "  asked Docker Desktop to quit gracefully"
  sleep 4
  # Reap any lingering Desktop background processes.
  for p in 'Docker Desktop' 'com.docker.backend' 'com.docker.virtualization' 'com.docker.build' 'docker-sandbox'; do
    pkill -f "$p" 2>/dev/null && echo "  reaped: $p"
  done
  # NOTE: intentionally NOT touching com.docker.vmnetd (privileged helper; harmless).
else
  echo "  Docker Desktop not running — good"
fi

# ---- 3. tear down the wedged colima VM --------------------------------------
say "Deleting the wedged colima VM (profile: $PROFILE)"
colima delete --force --profile "$PROFILE" 2>/dev/null && echo "  colima delete ok" || warn "colima delete failed — trying a hard limactl reset"
if colima status --profile "$PROFILE" >/dev/null 2>&1; then
  warn "VM still present; forcing via limactl"
  limactl stop   -f "colima${PROFILE:+-$PROFILE}" 2>/dev/null || limactl stop   -f colima 2>/dev/null
  limactl delete -f "colima${PROFILE:+-$PROFILE}" 2>/dev/null || limactl delete -f colima 2>/dev/null
fi

# ---- 3b. confirm the colima build (must be current for vz) -------------------
# This machine runs a SOURCE-BUILT colima installed at /usr/local/bin/colima
# (e.g. v0.8.1-190-g466d247) that fixes the Apple-Silicon vz socket-provisioning +
# restart bugs. Do NOT `brew upgrade` — that would clobber the purpose-built binary
# with the older released one. Just report which colima we're about to use.
say "Using colima on PATH (do NOT brew-upgrade a source build)"
echo "  colima: $(command -v colima)  version: $(colima version 2>/dev/null | head -1)"

# ---- 4. start a clean, large VM ---------------------------------------------
# VM driver: vz (Apple Virtualization.framework) — the FAST path on Apple Silicon,
# working on the current source build. Override with COLIMA_VMTYPE=qemu only if a vz
# regression resurfaces. virtiofs is the fast mount and is vz-only; qemu needs sshfs.
VMTYPE="${COLIMA_VMTYPE:-vz}"
say "Starting a clean colima VM: ${CPUS} CPU / ${MEM} GiB / ${DISK} GiB (vm-type: ${VMTYPE})"
COLIMA_START_ARGS=(--profile "$PROFILE" --cpu "$CPUS" --memory "$MEM" --disk "$DISK" --vm-type "$VMTYPE")
if [ "$VMTYPE" = "vz" ]; then
  COLIMA_START_ARGS+=(--mount-type virtiofs)
else
  COLIMA_START_ARGS+=(--mount-type sshfs)
fi
colima start "${COLIMA_START_ARGS[@]}" \
  || die "colima start failed — check 'colima start --help' and host resources"

# ---- 5. point the docker CLI at colima --------------------------------------
say "Pointing the docker CLI at the colima context"
CTX="colima"; [ "$PROFILE" != "default" ] && CTX="colima-$PROFILE"
docker context use "$CTX" 2>/dev/null && echo "  docker context = $CTX" || warn "could not switch context to $CTX (check 'docker context ls')"
unset DOCKER_HOST 2>/dev/null || true   # a stale DOCKER_HOST env would override the context

# ---- 6. verify the daemon actually serves (go/no-go) -------------------------
say "Verifying the daemon serves (dockerd can take 15-45s after boot)"
SERVING=""
for i in $(seq 1 10); do
  out=$(timeout 10 docker info --format 'cpus={{.NCPU}} mem={{.MemTotal}}' 2>/dev/null)
  if [ -n "$out" ] && [ "$out" != "cpus=0 mem=0" ]; then SERVING="$out"; break; fi
  echo "  waiting for dockerd (attempt $i/10): ${out:-<no response>}"
  sleep 6
done

[ -n "$SERVING" ] || die "docker daemon not serving after boot (EOF/cpus=0). If you used vz, re-run with the default driver: COLIMA_VMTYPE=qemu $0"
echo "  daemon serving: $SERVING"

say "Smoke: docker run --rm hello-world"
if timeout 120 docker run --rm hello-world 2>&1 | grep -qi 'Hello from Docker'; then
  printf '\n\033[1;32m✔ Docker is healthy on colima. You are unblocked.\033[0m\n'
  docker info --format '  cpus={{.NCPU}} mem_bytes={{.MemTotal}} context='"$CTX"
else
  die "hello-world did not run — daemon reachable but containers fail (in-VM socket perms?). Check: colima ssh -- docker info"
fi
