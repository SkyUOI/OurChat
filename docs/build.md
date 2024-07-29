# How to build this project

## Server

Server is developed in rust.You should install rust first.
The development and deployment are in the docker,so you should also install docker.
docker-buildx and docker-compose are required,too.
Then you can run

```bash
docker-compose up -d
```

to set up the development environment.

If the Dockerfile changed,you can run `script/rebuild_dev_container.py` to rebuild the image.

Build and Run:

```bash
cargo run
```

But you should start server with some config.You can see examples in `config` folder.

Use this to run with config:

```bash
cargo run -- --cfg=cfg.toml
```

Run unittest:

```bash
cargo test
```

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
