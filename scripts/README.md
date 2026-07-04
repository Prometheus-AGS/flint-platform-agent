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
reaps its background processes → **`brew upgrade colima`** (stale builds have
Apple-Silicon `vz` bugs — socket-provisioning + broken `restart`; current builds fix
them) → `colima delete --force` (with a `limactl` hard-reset fallback) →
`colima start --vm-type vz --mount-type virtiofs --cpu … --memory … --disk …` →
`docker context use colima` → waits for dockerd (15–45s) → verifies with `docker info`
+ `docker run --rm hello-world`.

**Runtime choice:** uses colima with the **`vz`** driver (Apple Virtualization.framework
— fast, sub-second boots) because our make-or-break integration point is the Rust
`testcontainers` crate, which speaks the **Docker API + Ryuk**. colima serves that
socket directly. (Apple's `container` 1.0 does **not** implement the Docker API, so
testcontainers needs a `socktainer` shim + `TESTCONTAINERS_RYUK_DISABLED` — two extra
unknowns; not used here. Podman works but adds Ryuk-compat risk.) Overrides:
`COLIMA_VMTYPE=qemu` (only if a vz regression resurfaces), `COLIMA_SKIP_UPGRADE=1`.

**Cautions:** it **quits Docker Desktop** (does not uninstall it — relaunch later,
just never run both engines at once). It leaves the privileged `com.docker.vmnetd`
helper alone. Safe to re-run.

Tune `COLIMA_CPUS` / `COLIMA_MEM` / `COLIMA_DISK` (env overrides) for your host —
12 GiB assumes ≥24 GiB of host RAM; drop to 10 on a 16 GiB Mac.
