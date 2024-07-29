# 部署指南

对于该项目，我们提供了一个生产环境中使用的 dockerfile，可以直接构建并运行项目。

以下是具体操作步骤：

```bash
docker-compose -f docker-compose.prod.yml up -d
```

这一步完成只是创建了一个最基本的环境，但是安全性还远远没有达到，为了保证安全性，你需要修改 MySQL 和 Redis 的密码。

具体有以下几步:

- 修改`docker-compose.prod.yml`中的两个`password`的`123456`为你自己的强密码
- 修改`config/database.json`为改后的 MySQL 密码，`config/redis_connection.json`为改后的 Redis 密码
- 再次运行`docker-compose -f docker-compose.prod.yml up -d`

完成这几步之后，你就成功部署了该项目。

对于容器中的数据，我们将其映射在了`mysql-data`和`redis-data`中，你可以随时保存数据
