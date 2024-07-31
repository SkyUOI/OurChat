# Server Arguments

| name                   |                                                    usage |
| :--------------------- | -------------------------------------------------------: |
| --cfg=config file path | choose the server config.[example](#config-file-example) |
| --test_mode            |                Run in test mode,only for debug and test. |

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
