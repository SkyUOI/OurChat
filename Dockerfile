FROM rust:latest

WORKDIR /app

CMD cd /app/server && cargo build --release
