FROM skyuoi/ourchat:aphine-base AS chef
WORKDIR /app

FROM chef AS planner

COPY server /app/server
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY server /app/server
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY service /app/service
COPY .cargo /app/.cargo

RUN cargo build --release

FROM alpine:latest AS ourchat-server

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY config /etc/ourchat

CMD ["server", "-c", "/etc/ourchat/ourchat.toml"]

FROM alpine:latest AS http-server

COPY --from=builder /app/target/release/http_server /usr/local/bin/http_server
COPY config /etc/ourchat
COPY resource/logo.png /etc/resource/logo.png

CMD ["http_server", "-c", "/etc/ourchat/http.toml"]