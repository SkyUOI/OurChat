# 如何构建该项目

## server
server部分由c++语言编写，跨windows和linux平台，支持gcc,clang,msvc编译器，xmake。对于c++编译器，要求必须支持c++20标准
```
xmake
xmake run ourchat_server
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
bin/ourchat_server --dbcfg=jsonpath
```

启动单元测试的方法
```
xmake build unittest
xmake run unittest
```

## client
client部分由python编写，无需编译，要求是python3以上,通过以下命令进行安装和运行
```
pip3 install -r requirement.txt
cd ./src/client/pc/
python3 main.py
```