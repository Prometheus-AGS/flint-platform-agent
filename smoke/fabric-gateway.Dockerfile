# Smoke-local build of flint-realtime-fabric's frf-gateway.
#
# WHY THIS EXISTS (not fabric's own Dockerfile)
# ---------------------------------------------
# fabric's `crates/frf-gateway/src/routes/admin_ui.rs` embeds the admin SPA at
# compile time with rust-embed 8.11:
#
#     #[derive(RustEmbed)]
#     #[folder = "../../admin-ui/dist"]
#     struct AdminUiAssets;
#
# `routes/mod.rs` declares `pub mod admin_ui;` UNCONDITIONALLY, so this module
# compiles in EVERY build (the `dev-endpoints` feature only decides whether it's
# wired into the router, not whether it compiles). rust-embed's derive only
# generates `AdminUiAssets::get()` when the `#[folder]` path EXISTS at build
# time. fabric's own Dockerfile copies `crates/ proto/ Cargo.*` but NEVER
# `admin-ui/`, so inside the image `../../admin-ui/dist` is absent and the derive
# emits a struct with no `get()`:
#
#     error[E0599]: no associated function or constant named `get` found for
#                   struct `AdminUiAssets`
#
# That is a Dockerfile↔source drift in fabric's `main` (its working tree HAS a
# built admin-ui/dist; the image build context does not). fabric's `main` is
# dirty and not ours to edit, so we DO NOT patch the sibling repo. Instead this
# smoke-local Dockerfile mirrors fabric's build and adds ONE step: materialise a
# minimal `admin-ui/dist/index.html` before `cargo build`, so rust-embed sees a
# real folder and generates `get()`. The admin SPA is irrelevant to the
# realtime-receipt proof (which uses /v1/publish + /ws/v1/subscribe + /healthz);
# a placeholder index.html is sufficient to make the module compile.
#
# Keep this in lock-step with fabric's Dockerfile: if fabric starts building the
# real admin-ui in its image (or drops the module), delete this override and
# point compose.real.yml back at `dockerfile: Dockerfile`.

FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    clang libclang-dev protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependency compilation (same layout as fabric's Dockerfile).
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY proto/ proto/

# The one addition over fabric's Dockerfile: give rust-embed a real folder to
# embed. Without a file present the derive still needs the directory to exist to
# emit a working `get()`; a placeholder index.html covers both.
RUN mkdir -p admin-ui/dist \
    && printf '<!doctype html><title>smoke</title><body>fabric admin ui (smoke placeholder)</body>' \
       > admin-ui/dist/index.html

ARG CARGO_FEATURES=""

RUN if [ -n "$CARGO_FEATURES" ]; then \
        cargo build --release -p frf-gateway --features "$CARGO_FEATURES"; \
    else \
        cargo build --release -p frf-gateway; \
    fi

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates curl libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/frf-gateway /usr/local/bin/frf-gateway

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/frf-gateway"]
