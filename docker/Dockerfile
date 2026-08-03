FROM skyuoi/ourchat:alpine-base AS chef
WORKDIR /app

FROM chef AS builder

ARG SCCACHE_OSS_BUCKET=""
ARG SCCACHE_OSS_ENDPOINT=""
ARG SCCACHE_ENABLED="false"

ENV SCCACHE_OSS_BUCKET=${SCCACHE_OSS_BUCKET}
ENV SCCACHE_OSS_ENDPOINT=${SCCACHE_OSS_ENDPOINT}
ENV SCCACHE_OSS_NO_CREDENTIALS=1

COPY .cargo /app/.cargo

COPY server /app/server
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY service /app/service

RUN if [ "$SCCACHE_ENABLED" = "true" ]; then \
        export RUSTC_WRAPPER=/usr/bin/sccache && \
        cargo build --release --bin server --features=official; \
    else \
        cargo build --release --bin server --features=official; \
    fi

# NOTE: `docker/config/server_info.json` is excluded from the build context via
# .dockerignore, so the JWT signing secret is NEVER baked into the image. The
# server generates a fresh, per-deployment secret on first start when the file
# is absent.
FROM alpine:3 AS ourchat-server

# su-exec is used by the entrypoint to drop from root to the `ourchat` user at
# startup, after fixing the ownership of bind-mounted runtime directories.
RUN apk add --no-cache su-exec

# Create a dedicated unprivileged user/group. The server runs as `ourchat`
# instead of root so that any exploit gains a non-root principal.
RUN addgroup -S ourchat && adduser -S -G ourchat -h /app ourchat

COPY --from=builder /app/target/release/server /usr/local/bin/server
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

CMD ["server", "-c", "/etc/ourchat/ourchat.toml", "--server-info", "/etc/ourchat/server_info.json"]
