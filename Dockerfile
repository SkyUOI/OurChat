FROM mysql:9.0.1
FROM rust:latest

# 设置工作目录
WORKDIR /app

# 将本地代码复制到容器中
COPY . /app

CMD ["rustup", "component", "add", "rust-analyzer"]
