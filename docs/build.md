# How to build this project

## Server

For development building, please refer to [Server Development](./server-develop-zh.md).

For actual deployment to a production environment, refer to the [Deployment Guide](./deploy.md)

## Web

First,you should run:

```bash
rustup target add wasm32-unknown-unknown
```

to add wasm support.

then you should run:

```bash
cargo install trunk
```

to install [trunk](https://github.com/trunk-rs/trunk) to build and run web application.

Then run `trunk serve` and `trunk build` to build and run web application.

## Tauri-client

Tauri-client is developed in rust.Use the same frontend of web client.

First you should run:

```bash
cargo install tauri-cli
```

to install a tool to manage the tauri application.

Then run:

```bash
cd src-tauri && cargo tauri dev
```

to build and run tauri application.

## client

client is developed in python.Require python3 or higher.Install and run:

## PC

```bash
cd ./client/pc/
pip3 install -r requirement.txt
python3 main.py
```
