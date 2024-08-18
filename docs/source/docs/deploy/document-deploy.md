# 本地文档部署指南

该文档使用Sphinx作为文档生成器

## 环境配置

- Python: python3 以上即可
- 相关依赖: 见下方说明

在`docs`目录下运行下方命令以安装相关依赖

```bash
pip install -r requirements.txt
```

## 生成文档

在`docs`目录下运行

```bash
./make
```

即可查看所有可用的生成器

```bash
# OUTPUT
html        to make standalone HTML files
#...
```

使用

```bash
./make target
```

即可生成指定类型的文档，生成完毕后请检查`build/target`目录

该文档默认语言为zh_CN，如果你想要生成其他语言的文档，请修改`source/conf.py`中的`language`配置项
目前支持语言

- zh_CN
- en

### Example

例如: 生成`en`语言的`html`文档

```python
# conf.py
language = "en"
```

```bash
./make html
```

文档将会生成在`build/html`目录中
