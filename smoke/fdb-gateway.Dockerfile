# Smoke image for forge's Quarry gateway (`fdb-gateway`) — authored HERE, built from
# the ../flint-forge crate via a read-only Docker build context. NOTHING is written
# into the forge repo (forge ships no Dockerfile for this crate; no forge phase does).
#
# Build context = the forge repo root (../flint-forge). The adjacent
# `fdb-gateway.Dockerfile.dockerignore` (BuildKit resolves it relative to THIS
# Dockerfile, not inside the context) trims forge's 34 GB target/ and .git so the
# context stays small — without placing a .dockerignore inside forge.
#
# Toolchain matches forge's own CI (Dagger builds on rust:1.90-bookworm; forge is
# edition 2021 / MSRV 1.85). `fdb-gateway` reads DATABASE_URL + KETO_BASE_URL at
# runtime and binds 0.0.0.0:8080 (hard-coded in its main.rs). It builds without a
# live DB (runtime sqlx, no compile-time query macros / .sqlx cache).

# ─────────────────────────── builder ───────────────────────────
FROM rust:1.90-slim-bookworm AS builder
WORKDIR /build

# pkg-config + git for build scripts; ca-certificates for cargo HTTPS; libssl-dev
# because forge's tree pulls openssl-sys transitively (native-tls in Cargo.lock).
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config ca-certificates git libssl-dev \
 && rm -rf /var/lib/apt/lists/*

# Copy the forge workspace (context = ../flint-forge, trimmed by the adjacent ignore).
COPY . .

# Self-contained build: neutralize any host rustc-wrapper/linker config (forge ships
# only .cargo/config.toml.example, but be defensive).
ENV RUSTC_WRAPPER="" \
    CARGO_BUILD_RUSTC_WRAPPER="" \
    CARGO_BUILD_RUSTFLAGS=""

# Build only the gateway binary, release, locked to forge's Cargo.lock.
RUN cargo build -p fdb-gateway --release --bin fdb-gateway --locked

# ─────────────────────────── runtime ───────────────────────────
FROM debian:bookworm-slim AS runtime
# ca-certificates for outbound HTTPS (Keto, etc.); libssl3 is the runtime shared lib
# for the openssl-sys linkage.
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10002 --home /home/fdb fdb
COPY --from=builder /build/target/release/fdb-gateway /usr/local/bin/fdb-gateway

USER fdb
# fdb-gateway binds 0.0.0.0:8080 (hard-coded in main.rs).
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/fdb-gateway"]
