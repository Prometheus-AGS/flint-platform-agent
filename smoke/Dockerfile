# Multi-stage build for the Flint Platform Agent gateway (smoke image).
#
# Builder: our MSRV toolchain (edition 2024, rust-version 1.93). Runtime: slim Debian
# with only CA certs. reqwest uses rustls-tls (no OpenSSL runtime dep), so the runtime
# image stays minimal.
#
# Build context = repo root:
#   docker build -f smoke/Dockerfile -t fpa-gateway:smoke .

# ─────────────────────────── builder ───────────────────────────
FROM rust:1.93-slim-bookworm AS builder
WORKDIR /build

# Build deps: pkg-config + git occasionally needed by transitive build scripts;
# ca-certificates so cargo can fetch over HTTPS. (No libssl-dev — rustls-tls.)
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config ca-certificates git \
 && rm -rf /var/lib/apt/lists/*

# Copy the whole workspace (the .dockerignore trims target/, .git, KBD state, AND
# .cargo/config.toml — the machine-local one references host-only sccache + ld64.lld).
COPY . .

# Belt-and-suspenders: neutralize any host rustc-wrapper/linker config that might slip
# in, so the container build is self-contained (uses the image's rustc + default linker).
ENV RUSTC_WRAPPER="" \
    CARGO_BUILD_RUSTC_WRAPPER="" \
    CARGO_BUILD_RUSTFLAGS=""

# Build only the gateway binary, release, locked to Cargo.lock.
RUN cargo build -p fpa-gateway --release --bin fpa-gateway --locked

# ─────────────────────────── runtime ───────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Non-root runtime user.
RUN useradd --system --uid 10001 --home /home/fpa fpa
COPY --from=builder /build/target/release/fpa-gateway /usr/local/bin/fpa-gateway

USER fpa
EXPOSE 8088
ENTRYPOINT ["/usr/local/bin/fpa-gateway"]
