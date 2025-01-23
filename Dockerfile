FROM skyuoi/ourchat:aphine-base AS chef
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release

FROM alpine:latest AS ourchat-server

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/config /etc/ourchat

CMD ["server", "-c", "/etc/ourchat/ourchat.toml"]

FROM alpine:latest AS http-server

COPY --from=builder /app/target/release/http_server /usr/local/bin/http_server
COPY --from=builder /app/config /etc/ourchat

CMD ["http_server", "-c", "/etc/ourchat/http.toml"]