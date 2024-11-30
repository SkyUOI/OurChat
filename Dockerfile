FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN apt update && apt install cmake protobuf-compiler -y

RUN cd /app/server && cargo build --release

FROM debian:stable-slim

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/config /etc/ourchat

CMD ["server", "-c", "/etc/ourchat/ourchat.toml"]
