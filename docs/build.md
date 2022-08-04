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
启动单元测试的方法
```
cmake ../src/server -DCMAKE_BUILD_TYPE=Release -DOURCHAT_BUILD_TYPE=Test
make
./unittest
```
记得提前安装编译安装googletest单元测试框架

## client
client部分由python编写，无需编译，要求是python3以上,通过以下命令进行安装和运行
```
pip install -r requirement.txt
python src/client/main.py
```