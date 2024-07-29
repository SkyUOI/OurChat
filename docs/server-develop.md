# Server Development Guide

- [Project Build Dependencies](#project-build-dependencies)
- [Container Development](#container-development)
- [Database](#database)
- [Server Configuration Files](#server-configuration-files)

## Project Build Dependencies

The Server part is written in Rust language. You should first install Rust.
Development and deployment are both completed in Docker, so you should also install Docker.
Docker-buildx and docker-compose are also needed to be installed.

## Server Configuration Files

All configuration file examples are stored in the `config` directory. Please do not change these configuration files during the development phase.

## Container Development

We provide a Dockerfile for the development environment.

You can run:

```bash
docker-compose up -d
```

to configure the development environment.

If the Dockerfile has changed, you can run `script/rebuild_dev_container.py` to rebuild the image.

We directly map the local folder to the `/app` folder in the container, which allows you to reset the container without worrying about data loss.

The recommended development method is to edit locally with an editor, and at the same time, use `docker exec -it OurChatServer bash` to enter the container to run and observe the results.

First, switch to the `server` directory where all development will take place.

Start with:

```bash
cargo run -- --cfg=cfg.toml
```

Start testing:

```bash
cargo test
```

## Database

This project uses Redis and MySQL as databases, and uses sea-orm as the ORM framework. To better use this ORM framework, after modifying tables in databases, you can run `script/regenerate_entity.py` to regenerate the files needed by the ORM framework.

To run this script, you first need to run `cargo install sea-orm-cli` to install the corresponding tool.
