# syntax=docker/dockerfile:1.7
# BuildKit is required for the cache mounts below (`--mount=type=cache`).
# Docker 23+ enables BuildKit by default; the syntax directive pins the
# frontend so older daemons also pick it up.

FROM skyuoi/ourchat:alpine-base AS chef
WORKDIR /app

# Planner: cargo-chef extracts the dependency recipe from the workspace
# manifests. This layer only invalidates when Cargo.toml/Cargo.lock change,
# so dependency compilation can be cached separately from application code.
FROM chef AS planner
COPY .cargo /app/.cargo
COPY server /app/server
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ARG SCCACHE_OSS_BUCKET=""
ARG SCCACHE_OSS_ENDPOINT=""
# sccache is only enabled when SCCACHE_OSS_BUCKET is provided at build time
# (CI use case). Local builds skip sccache entirely and rely on BuildKit's
# target cache mount for incremental compilation. The Alpine sccache package
# defaults to OSS mode when invoked via RUSTC_WRAPPER, so we cannot "always
# on with local fallback" — it has to be all-or-nothing.

# Cook the dependencies first. The cache mounts persist the cargo registry
# and target dir across builds, so changing application code only recompiles
# the crates that actually changed.
#
# service/ must be present at cook time because server/pb/build.rs walks
# ../../service for .proto files when its build script runs.
COPY --from=planner /app/recipe.json recipe.json
COPY .cargo /app/.cargo
COPY service /app/service
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/root/.cache/sccache \
    if [ -n "$SCCACHE_OSS_BUCKET" ]; then \
        export RUSTC_WRAPPER=/usr/bin/sccache \
               SCCACHE_OSS_BUCKET="$SCCACHE_OSS_BUCKET" \
               SCCACHE_OSS_ENDPOINT="$SCCACHE_OSS_ENDPOINT" \
               SCCACHE_OSS_NO_CREDENTIALS=1; \
    fi && \
    cargo chef cook --release --recipe-path recipe.json

# Now layer the real source on top and build the binary. Deps are already
# in the cache mount, so this step is incremental.
COPY server /app/server
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
# The binary lives inside the cache mount, which does not exist in the
# final image — copy it to a stable path the next stage can COPY from.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    --mount=type=cache,target=/root/.cache/sccache \
    if [ -n "$SCCACHE_OSS_BUCKET" ]; then \
        export RUSTC_WRAPPER=/usr/bin/sccache \
               SCCACHE_OSS_BUCKET="$SCCACHE_OSS_BUCKET" \
               SCCACHE_OSS_ENDPOINT="$SCCACHE_OSS_ENDPOINT" \
               SCCACHE_OSS_NO_CREDENTIALS=1; \
    fi && \
    cargo build --release --bin server --features=official && \
    cp /app/target/release/server /tmp/server

# NOTE: `docker/config/server_info.json` is excluded from the build context
# via .dockerignore, so the JWT signing secret is NEVER baked into the
# image. The server generates a fresh, per-deployment secret on first
# start when the file is absent.
FROM alpine:3 AS ourchat-server

# su-exec is used by the entrypoint to drop from root to the `ourchat` user
# at startup, after fixing the ownership of bind-mounted runtime directories.
RUN apk add --no-cache su-exec

# Create a dedicated unprivileged user/group. The server runs as `ourchat`
# instead of root so that any exploit gains a non-root principal.
RUN addgroup -S ourchat && adduser -S -G ourchat -h /app ourchat

COPY --from=builder /tmp/server /usr/local/bin/server
COPY docker/config /etc/ourchat

# Lay out writable runtime directories and hand them to the non-root user.
RUN mkdir -p /app/log /app/files_storage /app/resource && \
    chown -R ourchat:ourchat /etc/ourchat /app

COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

WORKDIR /app

# The entrypoint starts as root only to chown the bind-mounted runtime dirs,
# then immediately drops to the unprivileged `ourchat` user before exec'ing
# the server. This makes the image work on any host without manual chown.
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

# HTTP (7777) and gRPC (7779). The debug console (7776) is disabled by default.
EXPOSE 7777 7779

# `/v1/status` returns 200 with no auth (server/src/httpserver/status.rs).
# alpine's busybox provides wget. Use 127.0.0.1, not localhost — the server
# binds 0.0.0.0 (IPv4), and localhost may resolve to ::1 (IPv6) first.
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=5 \
    CMD wget -q -O /dev/null http://127.0.0.1:7777/v1/status || exit 1

CMD ["server", "-c", "/etc/ourchat/ourchat.toml", "--server-info", "/etc/ourchat/server_info.json"]
