FROM rustlang/rust:nightly-alpine AS builder

WORKDIR /app

RUN apk add cmake protobuf-dev musl-dev zlib-static g++

COPY . .

RUN cd /app && cargo build --release

FROM alpine:latest AS ourchat-server

COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/config /etc/ourchat

CMD ["server", "-c", "/etc/ourchat/ourchat.toml"]

FROM alpine:latest AS http-server

COPY --from=builder /app/target/release/http_server /usr/local/bin/http_server
COPY --from=builder /app/config /etc/ourchat

CMD ["http_server", "-c", "/etc/ourchat/http.toml"]