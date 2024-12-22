FROM rustlang/rust:nightly-alpine as builder

WORKDIR /app

COPY . /app

RUN cargo build --release

FROM alpine:latest

COPY --from=builder /app/target/release/load_balancer /usr/local/bin/load_balancer

CMD ["/usr/local/bin/load_balancer"]