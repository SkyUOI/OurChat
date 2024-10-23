FROM rust:latest AS builder

WORKDIR /app

COPY . .

CMD cd /app/server && cargo build --release

FROM debian:stable-slim

COPY --FROM=builder /app/server/target/release/server /usr/local/bin/server

CMD ["server"]
