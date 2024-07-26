# How to build this project

## Server

Server is developed in rust.
Build:

```
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

```
cargo run -- --dbcfg=jsonpath
```

Run unittest

```
cargo test
```

## client

client is developed in python.Require python3 or higher.Install and run:

```
pip3 install -r requirement.txt
cd ./src/client/pc/
python3 main.py
```
