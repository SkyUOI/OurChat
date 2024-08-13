# 服务端开发指南

- [项目构建依赖](#项目构建依赖)
- [容器开发](#容器开发)
- [数据库](#数据库)
- [服务器配置文件](#服务器配置文件)

## 项目构建依赖

Server 部分由 Rust 语言编写.你首先应当安装 Rust.
开发和部署都在 Docker 中完成，所以你也应当安装 Docker.
docker-buildx 和 docker-compose 同样也需要安装.

## 服务器配置文件

于`config`目录下存放了所有配置文件的示例，在开发阶段请最好不要更改这些配置文件。

## 容器开发

我们提供了一个用于开发环境的 Dockerfile

你可以运行:

```bash
docker compose up -d
```

来配置开发环境

如果 Dockerfile 改变了，你可以运行`script/rebuild_dev_container.py`来重新构建镜像。

我们直接将本地文件夹映射到了容器中的`/app`文件夹，这使得你可以放心地重置容器而不用担心数据丢失。

推荐的开发方式是在本地使用编辑器编辑，同时使用`docker exec -it OurChatServer bash`进入容器运行并观察结果

首先，切换进`server`目录中，开发都将在这里进行。

启动时使用

```bash
cargo run -- --cfg=cfg.toml
```

启动测试:

```bash
cargo test
```

## 数据库

本项目采用 Redis 和 MySQL 作为数据库，同时采用 sea-orm 作为 ORM 框架。为了更好地使用该 ORM 框架，在修改数据库表后，您可以运行`script/regenerate_entity.py`来重新生成 ORM 框架需要的文件

为了运行这个脚本，你首先需要运行`cargo install sea-orm-cli`

注意：如果可以，请最好保证`sea-orm-cli`是最新的

### 数据库迁移

`migration`中是数据库迁移模块，在server启动时会自动运行未运行的数据库迁移，为了定义一个新的数据库迁移，请参考[sea orm](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
