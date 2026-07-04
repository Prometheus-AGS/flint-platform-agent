# scripts/

Developer tooling for the Flint Platform Agent. These are **machine-local dev
utilities**, not part of the built binary or CI.

## `reset-colima.sh`

Tears down a wedged Docker setup on macOS and brings up **one** clean colima VM
sized to build/run the container-based smoke stacks (pgrx/Rust image builds are
memory-hungry).

**When you need it:** the docker CLI hangs or `docker info` returns `EOF`. The usual
cause is **Docker Desktop and colima both running** and racing over the docker
socket. This script makes colima the sole engine, recreates its VM clean + large,
points the CLI at it, and verifies the daemon actually serves.

```bash
./scripts/reset-colima.sh
# or size it for your host (defaults: 6 CPU / 12 GiB / 120 GiB):
COLIMA_MEM=10 COLIMA_CPUS=4 ./scripts/reset-colima.sh
```

**What it does:** kills frozen docker/colima CLI calls → quits Docker Desktop and
reaps its background processes → `colima delete --force` (with a `limactl` hard-reset
fallback) → `colima start --cpu … --memory … --disk … --vm-type vz --mount-type
virtiofs` → `docker context use colima` → verifies with `docker info` +
`docker run --rm hello-world`.

**Cautions:** it **quits Docker Desktop** (does not uninstall it — relaunch later,
just never run both engines at once). It leaves the privileged `com.docker.vmnetd`
helper alone. Safe to re-run.

Tune `COLIMA_CPUS` / `COLIMA_MEM` / `COLIMA_DISK` (env overrides) for your host —
12 GiB assumes ≥24 GiB of host RAM; drop to 10 on a 16 GiB Mac.
