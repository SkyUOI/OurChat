# 服务器参数

| name                   |                                               usage |
| :--------------------- | --------------------------------------------------: |
| --cfg=config file path | choose the database config.[example](#配置文件示例) |

## 配置文件示例

```toml
dbcfg = "../config/database.json"
rediscfg = "../config/redis_connect.json"
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
