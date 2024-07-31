# 开发基本注意事项

## 脚本

我们提供了一系列脚本，帮助您处理日常简单事物。

你可以运行`script/test_server.py`来运行服务端的测试。为了您可以自定义设置，您可以新建一个`local`文件夹(该文件夹已经被添加到`.gitignore`)，然后将脚本复制进去。

例如，我们设置了`OURCHAT_CONFIG_FILE`环境变量来读取服务器运行的配置文件，你可以将`script/test_server.py`复制到`local`文件夹，复制`ourchat.toml`等配置文件，然后添加:

```python
os.putenv("OURCHAT_CONFIG_FILE", "../local/ourchat.toml")
```

从而进行服务端测试的自定义。

## 测试

由于服务端测试具有特殊性，我们引入了一个`test_lib`模块用于辅助测试，具体可以参考已有的单元测试。

## 文档

请善用`cargo doc`，我们为你提供了详尽的文档参考！调用`cargo doc --document-private-items`来生成私有的文档
