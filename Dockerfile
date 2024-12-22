FROM rustlang/rust:nightly-alpine AS builder

WORKDIR /app

RUN apk add cmake protobuf-dev musl-dev zlib-static

COPY . .

RUN cd /app/server && cargo build --release

FROM alpine:latest

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/config /etc/ourchat

CMD ["server", "-c", "/etc/ourchat/ourchat.toml"]
