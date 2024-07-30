# Deployment Guide

For this project, we provide a Dockerfile for use in a production environment, which can be built and run directly.

Here are the specific steps:

```bash
docker-compose -f docker-compose.prod.yml up -d
```

This step only creates the most basic environment, but the security is far from sufficient. To ensure security, you need to change the passwords for MySQL and Redis.

The specific steps are as follows:

- Modify the `password` `123456` in `docker-compose.prod.yml` to your own strong password.
- Change the MySQL password in `config/database.json` and the Redis password in `config/redis_connection.json` to the updated passwords.
- Run `docker-compose -f docker-compose.prod.yml up -d` again.

After completing these steps, you will have successfully deployed the project.

For the data in the container, it is mapped in `mysql-data` and `redis-data`, and you can save the data at any time.
