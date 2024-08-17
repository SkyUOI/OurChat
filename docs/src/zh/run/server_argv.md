# 服务器参数

| name                         |                                  usage |   default |
| :--------------------------- | -------------------------------------: | --------: |
| --cfg=config file path       |  选择配置文件.[example](#配置文件示例) |    不适用 |
| --test-mode                  | 以测试模式启动服务器，仅供开发调试使用 |     false |
| --clear                      |           在启动时清除服务器缓存和日志 |     false |
| --port=server listening port |                       服务器监听的端口 |      7777 |
| --ip=server listening ip     |                        服务器监听的 ip | 127.0.0.1 |

## 配置文件示例

```toml
dbcfg = "config/database.json"
rediscfg = "config/redis_connect.json"
port = 7777
```

注意，当配置文件和命令行参数冲突时，以命令行参数覆盖配置文件参数

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
