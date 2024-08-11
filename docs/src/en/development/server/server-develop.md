# Server Development Guide

- [Project Build Dependencies](#project-build-dependencies)
- [Container Development](#container-development)
- [Database](#database)
- [Server Configuration Files](#server-configuration-files)

## Project Build Dependencies

The Server part is written in the Rust language. You should first install Rust.
Development and deployment are completed in Docker, so you should also install Docker.
Docker-buildx and docker-compose are also required.

## Server Configuration Files

Examples of all configuration files are stored in the `config` directory. During the development phase, please do not change these configuration files if possible.

## Container Development

We provide a Dockerfile for the development environment.

You can run:

```bash
docker-compose up -d
```

to set up the development environment.

If the Dockerfile has changed, you can run `script/rebuild_dev_container.py` to rebuild the image.

We directly map the local folder to the `/app` folder in the container, which allows you to safely reset the container without worrying about data loss.

The recommended development method is to use an editor locally for editing, and at the same time, use `docker exec -it OurChatServer bash` to enter the container to run and observe the results.

First, switch to the `server` directory where all development will take place.

Start with:

```bash
cargo run -- --cfg=cfg.toml
```

Start tests with:

```bash
cargo test
```

## Database

This project uses Redis and MySQL as databases, and uses sea-orm as the ORM framework. To better use this ORM framework, after modifying the database tables, you can run `script/regenerate_entity.py` to regenerate the files needed by the ORM framework.

To run this script, you first need to run `cargo install sea-orm-cli`.

Note: If possible, please make sure that `sea-orm-cli` is up to date.

### Database Migration

The `migration` folder contains the database migration module. Unexecuted database migrations will automatically run when the server starts. To define a new database migration, please refer to the [Sea ORM documentation](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/).
