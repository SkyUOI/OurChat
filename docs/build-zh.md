# 如何构建该项目

## Server

Server 部分由 Rust 语言编写.你首先应当安装 Rust.
开发和部署都在 Docker 中完成，所以你也应当安装 Docker.
docker-buildx 和 docker-compose 同样也需要安装.

然后你可以运行:

```bash
docker-compose up -d
```

来配置开发环境

如果 Dockerfile 改变了，你可以运行`script/rebuild_dev_container.py`来重新构建镜像。

构建并运行:

```bash
cargo run
```

但是启动服务端需要配置相应参数，可以见`config`目录下的示例

启动时使用

```bash
cargo run -- --cfg=cfg.toml
```

启动单元测试的方法

```bash
cargo test
```

## Web

首先，你需要运行:

```bash
rustup target add wasm32-unknown-unknown
```

来添加 wasm 支持.

然后你应该运行:

```bash
cargo install trunk
```

来安装用于构建和运行 web 应用的[trunk](https://github.com/trunk-rs/trunk).

接着运行`trunk serve`和`trunk build` 来构建和运行 web 应用.

## Tauri-client

Tauri-client 使用 Rust 开发.使用与 web 端相同的前端.

首先你应当运行:

```bash
cargo install tauri-cli
```

来安装一个管理 tauri 项目的工具.

然后运行:

```bash
cd src-tauri && cargo tauri dev
```

来构建和运行 Tauri 应用.

## client

client 部分由 python 编写，无需编译，要求是 python3 以上,通过以下命令进行安装和运行

## PC

```bash
cd ./client/pc/
pip3 install -r requirement.txt
python3 main.py
```
