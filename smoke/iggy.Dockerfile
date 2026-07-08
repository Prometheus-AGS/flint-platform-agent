# Protocol-matched Iggy server for the real smoke (p12-c003 realtime-receipt).
#
# WHY THIS EXISTS
# ---------------
# flint-realtime-fabric pins its Iggy *client* to a fork, not a released crate:
#
#   iggy = { git = "https://github.com/GQAdonis/iggy", branch = "master" }
#   # Cargo.lock: iggy 0.10.1-edge.2
#   #   git+https://github.com/GQAdonis/iggy?branch=master#9a0f799b7becafb919e937ad12a35044b9edcb33
#
# The upstream `iggyrs/iggy:latest` image (used by fabric's own compose.yml and
# compose.ci.yml) floats far past that fork's protocol. Against `:latest` the
# fabric client's TCP sign-in / metadata call returns
# `invalid response with status: 3 (invalid_command)`, the events stream is
# never created, and a REAL /fabric/subscribe fails deep in the LogBroker →
# 500 on the WS pipeline → the agent's connect_async fails → 502 at the bridge.
# (That same mismatch is why fabric logs "fixture channel pre-creation failed
# (non-fatal)" on boot — non-fatal for boot, fatal for a real subscribe.)
#
# THE FIX
# -------
# Build the iggy *server* from the SAME fork at the SAME rev the client is
# locked to, so client and server speak the identical wire protocol. This is a
# smoke-local image; nothing is written into the fork or the sibling repos.
#
# The pinned rev below MUST equal the `#<rev>` in flint-realtime-fabric's
# Cargo.lock `iggy` entry. When fabric bumps its iggy client, bump IGGY_REV here
# to match (keep the two in lock-step or the invalid_command mismatch returns).

FROM rust:1.96-slim-bookworm AS builder

ARG IGGY_REV=9a0f799b7becafb919e937ad12a35044b9edcb33

WORKDIR /build
RUN apt-get update && apt-get install -y \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libhwloc-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Clone the fork at the exact locked rev. Shallow-fetch just that commit so the
# build context stays tiny and the rev is unambiguous.
RUN git init . \
    && git remote add origin https://github.com/GQAdonis/iggy \
    && git fetch --depth 1 origin "${IGGY_REV}" \
    && git checkout FETCH_HEAD

# Smoke-local SOURCE PATCH — the fork does not boot its TCP server on Linux.
# --------------------------------------------------------------------------
# core/server_common/src/executor.rs forces compio's blocking (asyncify) thread
# pool to ZERO on every non-macOS/aarch64 target:
#
#     #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
#     proactor.thread_pool_limit(0);
#
# with a standing FIXME(hubcio). On Linux the shard boots, io_uring works (QUIC
# and WebSocket listeners bind fine — this is NOT a seccomp/io_uring problem),
# but the moment the TCP/HTTP/WebSocket startup path dispatches a blocking op to
# the asyncify pool, compio-driver hits `if self.thread_limit == 0 { panic!(...) }`
# (asyncify.rs:118 "the thread pool is needed but no worker thread is running")
# and every critical task fails → the server shuts itself down → exit(0) with the
# TCP server never bound → fabric's client gets nothing → a real /fabric/subscribe
# 502s. The fork's own snapshot/mod.rs comment names the fix: "Enable thread pool
# by removing/increasing thread_pool_limit(0)". The macOS carve-out proves the
# fork runs correctly WITHOUT forcing the limit — compio then uses its documented
# default of 256 (compio-driver lib.rs: "default value is 256").
#
# So we neutralise that one line: comment it out so Linux also gets the default
# 256-thread pool. This is a smoke-local edit of a throwaway fork checkout inside
# THIS image build — nothing is written into the fork repo or any sibling repo.
# The `cpu_allocation` value is IRRELEVANT to this panic (it reproduces with
# "all", a numeric count, and "1" alike); the only trigger is thread_pool_limit(0).
#
# Keep in lock-step with IGGY_REV: if the fork lands the FIXME (guards the line by
# platform-that-needs-a-pool, or drops it), delete this sed. The `|| true` guard
# lets the build proceed if a rev bump removes the line, so a stale patch can't
# silently break the build; the grep-verify line below fails loudly instead.
RUN sed -i 's/^\( *\)proactor\.thread_pool_limit(0);/\1\/\/ thread_pool_limit(0); \/\/ smoke: neutralised (see iggy.Dockerfile) — Linux needs a nonzero asyncify pool/' \
        core/server_common/src/executor.rs \
    && if grep -qE '^ *proactor\.thread_pool_limit\(0\);' core/server_common/src/executor.rs; then \
         echo "ERROR: thread_pool_limit(0) patch did not apply — fork layout changed at IGGY_REV" >&2; exit 1; \
       fi

# Only the TCP server is needed on :8090 — skip the `web` npm static build the
# fork's own Dockerfile runs (fabric never touches the admin web UI in the smoke).
RUN cargo build --bin iggy-server --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    liblzma5 \
    libhwloc15 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/core/server/config.toml /config.toml
COPY --from=builder /build/target/release/iggy-server ./iggy-server

# Patch the fork's default config.toml for a container:
#
#  1. Rebind every listener from 127.0.0.1 → 0.0.0.0. The fork's default config
#     binds tcp/http/quic/websocket to LOOPBACK, which is unreachable from the
#     fabric-gateway container (env vars like IGGY_TCP_ADDRESS do NOT override an
#     explicit `address =` in the config file, so we must edit the file).
#  2. Set sharding cpu_allocation "numa:auto" → "all". colima's VM exposes no NUMA
#     topology, so "numa:auto" can't resolve a NUMA layout; "all" pins one shard
#     per visible core. (This is a correctness/portability nicety for the VM — it
#     is NOT what fixed the boot panic. The boot panic was thread_pool_limit(0),
#     patched in the builder stage above; it reproduced identically under "all",
#     a numeric count, and "1".)
RUN sed -i \
      -e 's/address = "127\.0\.0\.1:/address = "0.0.0.0:/g' \
      -e 's/^cpu_allocation = "numa:auto"/cpu_allocation = "all"/' \
      /config.toml

# IGGY_CONFIG_PATH pins config discovery to the patched file regardless of CWD.
ENV IGGY_CONFIG_PATH=/config.toml

CMD ["/iggy-server"]
