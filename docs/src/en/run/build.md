# How to Build the Project

## Server

For development and building, see [Backend Development](../development/server/server-develop.md).

For actual deployment to a production environment, see [Deployment Guide](../deploy.md).

## Web

Firstly, you need to run:

```bash
rustup target add wasm32-unknown-unknown
```

to add support for WebAssembly.

Then you should run:

```bash
cargo install trunk
```

to install [trunk](https://github.com/trunk-rs/trunk) for building and running web applications.

Next, run `trunk serve` and `trunk build` to build and run the web application.

## Tauri-client

The Tauri-client is developed using Rust. It uses the same frontend as the web version.

First, you should run:

```bash
cargo install tauri-cli
```

to install a tool for managing Tauri projects.

Then run:

```bash
cd src-tauri && cargo tauri dev
```

to build and run the Tauri application.

## client-pc

The client-pc part is written in Python and does not require compilation. It requires Python 3 or above. Install and run with the following commands:

### Running

```bash
python -m pip install -r client/pc/requirement.txt # Install the dependency libraries
python script/export_themes.py # Export themes to client/pc/src/theme
cd ./client/pc/src
python main.py # Run
```

### Packaging into an Executable

```bash
python script/build_pc.py
```

After the script finishes running, the executable file and its dependencies will be in the `client/pc/src/out/main.dist` directory.
