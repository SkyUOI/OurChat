# Server Parameters

| Name                   |                                                                 Usage |
| :--------------------- | --------------------------------------------------------------------: |
| --cfg=config file path | Select the configuration file. [Example](#configuration-file-example) |
| --test-mode            |     Start the server in test mode, for development and debugging only |
| --clear                |                              Clear server cache and logs upon startup |

## Configuration File Example

```toml
dbcfg = "config/database.json"
rediscfg = "config/redis_connect.json"
```

Database Example(dbcfg):

```json
{
  "host": "db",
  "user": "root",
  "passwd": "123456",
  "db": "OurChat",
  "port": 3306
}
```

Redis Example(rediscfg):

```json
{
  "host": "127.0.0.1",
  "port": 6379,
  "passwd": "123456",
  "user": "root"
}
```
