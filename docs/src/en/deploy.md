# Deployment Guide

## Docker (Recommended)

For this project, we provide a Dockerfile suitable for production environments, which can be directly built and run.

Here are the specific steps:

```bash
docker-compose -f docker-compose.prod.yml up -d
```

Completing this step only creates the most basic environment, but the security is far from adequate. To ensure security, you need to change the passwords for MySQL and Redis.

Here are the specific steps:

- Change the `password` `123456` in the `docker-compose.prod.yml` to your own strong passwords.
- Modify `config/database.json` to the new MySQL password, and `config/redis_connection.json` to the new Redis password.
- Run `docker-compose -f docker-compose.prod.yml up -d` again.

After completing these steps, you have successfully deployed the project.

For data in the container, we have mapped it to `mysql-data` and `redis-data`, where you can save data at any time.

## Manual Deployment

For computers with low performance and those that have not installed Docker, we also provide a manual deployment document. It is recommended to deploy in a Linux environment, as other environments have not been strictly tested.

### Install MySQL

MySQL version is 9.0.1 (if this document is not updated in time, you can check the MySQL version in the [`docker-compose.yml`](https://github.com/SkyUOI/OurChat/blob/main/docker-compose.yml)).

### Install Redis

You can directly install the latest version of Redis.

### Install OurChat Server

There are two alternative options here:

1. (Recommended) Download the latest Linux compilation result from the GitHub release. This version has been optimized by the official pgo, and the performance will be higher. If you encounter CPU architecture and other compatibility issues, you may need to compile manually, see the next section.

2. Manual compilation

- Pull the source code:

```sh
git clone https://github.com/SkyUOI/OurChat --depth=1 && cd OurChat
```

- Install the Rust toolchain

- Compile the project

```sh
cd server && cargo build --release
```

- (Optional) PGO Optimization

This step will take a lot of time to collect runtime information for program optimization. If you do not have stringent performance requirements, it is not recommended to do this.

```sh
cargo install cargo-pgo
```

- Run the project

Please refer to [Server Parameters](./run/server_argv.md) for running the project. The executable file is located at `target/server`.
