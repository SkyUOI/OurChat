# 如何构建该项目

## Server

开发构建，请见[服务端开发](../development/server/server-develop.md)

对于真正部署到生产环境，参见[部署指南](../deploy.md)

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

## client-pc

client-pc 部分由 python 编写，无需编译，要求是 python3 以上,通过以下命令进行安装和运行

### 运行

```bash
python -m pip install -r client/pc/requirement.txt # 安装依赖库
python script/export_themes.py # 导出主题到client/pc/src/theme中
cd ./client/pc/src
python main.py # 运行
```

### 打包为可执行文件

```bash
python script/build_pc.py
```

等待脚本运行完毕后，`client/pc/src/out/main.dist`目录中即为可执行文件及其依赖文件
