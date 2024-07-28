# How to build this project

## Server

Server is developed in rust.
Build:

```bash
cargo run
```

But you should start server with database config.You should write a json file to describe it.
The format:

```json
{
  "host": "",
  "user": "",
  "passwd": "",
  // Database which is provided for OurChat
  "db": "",
  // mysql port
  "port": 0
}
```

Use this to run with config:

```bash
cargo run -- --dbcfg=jsonpath
```

Run unittest

```bash
cargo test
```

## client

client is developed in python.Require python3 or higher.Install and run:

## PC

```bash
cd ./client/pc/
pip3 install -r requirement.txt
python3 main.py
```
