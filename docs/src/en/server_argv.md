# Server Arguments

| name                   |                                                      usage |
| :--------------------- | ---------------------------------------------------------: |
| --cfg=config file path | choose the database config.[example](#config-file-example) |

## config file example

```toml
dbcfg = "../config/database.json"
rediscfg = "../config/redis_connect.json"
```

dbcfg example:

```json
{
  "host": "db",
  "user": "root",
  "passwd": "123456",
  "db": "OurChat",
  "port": 3306
}
```

redis example:

```json
{
  "host": "127.0.0.1",
  "port": 6379,
  "passwd": "123456",
  "user": "root"
}
```
