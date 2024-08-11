# 服务器参数

| name                   |                                  usage |
| :--------------------- | -------------------------------------: |
| --cfg=config file path |  选择配置文件.[example](#配置文件示例) |
| --test-mode            | 以测试模式启动服务器，仅供开发调试使用 |
| --clear                |           在启动时清除服务器缓存和日志 |

## 配置文件示例

```toml
dbcfg = "config/database.json"
rediscfg = "config/redis_connect.json"
```

数据库示例:

```json
{
  "host": "db",
  "user": "root",
  "passwd": "123456",
  "db": "OurChat",
  "port": 3306
}
```

Redis 示例:

```json
{
  "host": "127.0.0.1",
  "port": 6379,
  "passwd": "123456",
  "user": "root"
}
```
