# Server Parameters

| Name                         |                                                                 Usage |   default |
| :--------------------------- | --------------------------------------------------------------------: | --------: |
| --cfg=config file path       | Select the configuration file. [Example](#configuration-file-example) |    不适用 |
| --test-mode                  |     Start the server in test mode, for development and debugging only |     false |
| --clear                      |                              Clear server cache and logs upon startup |     false |
| --port=server listening port |                                                 server listening port |      7777 |
| --ip=server listening ip     |                                                   server listening ip | 127.0.0.1 |

## Configuration File Example

```toml
dbcfg = "config/database.json"
rediscfg = "config/redis_connect.json"
port = 7777
```

Warning,if config file is conflicted with command argument, command argument will be used.

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
