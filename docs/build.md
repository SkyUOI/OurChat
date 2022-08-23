# 如何构建该项目

## server
server部分由c++语言编写，跨windows和linux平台，支持gcc,clang,msvc编译器，编译请安装cmake。对于c++编译器，要求必须支持c++20标准
```
mkdir build
cd build
cmake ../src/server -DCMAKE_BUILD_TYPE=Release
make
./OurChat_server
```
但是启动服务端需要配置相应的数据库，需要自己配置json文件
具体格式为

```json
{
  "host": "",
  "user": "",
  "passwd": "",
  "db": "",
  "port": 0
}
```
启动时使用
```
./OurChat_server --dbcfg=jsonpath
```

启动单元测试的方法
```
cmake ../src/server -DCMAKE_BUILD_TYPE=Release -DOURCHAT_BUILD_TYPE=Test
make
./unittest
```

## client
client部分由python编写，无需编译，要求是python3以上,通过以下命令进行安装和运行
```
pip install -r requirement.txt
python src/client/main.py
```