# 如何构建该项目

## Server

Server部分由Rust语言编写.通过以下方式运行：

```
cargo run
```

但是启动服务端需要配置相应的数据库，需要自己配置json文件
具体格式为

```json
{
  // 主机
  "host": "",
  // 用户
  "user": "",
  // 密码
  "passwd": "",
  // 为Ourchat准备的数据库
  "db": "",
  // mysql端口
  "port": 0
}
```

启动时使用

```
cargo run -- --dbcfg=jsonpath
```

启动单元测试的方法

```
cargo test
```

## client

client部分由python编写，无需编译，要求是python3以上,通过以下命令进行安装和运行

## PC
```
cd ./client/pc/
pip3 install -r requirement.txt
python3 main.py
```
