# Helper Scripts

Notice: If there is any script not listing below or out-of-date,please open a issue.

## Server Usage

### `db_migration.py`

As we are using `sea-orm`,you can use this to run database migrations via `sea-orm-cli`,the arguments is the same as `sea-orm-cli`

For more information,please refer to [Sea Orm](https://www.sea-ql.org/SeaORM/docs/migration/running-migration/) and [OurChat Server Development Document](https://ourchat.readthedocs.io/en/latest/docs/development/server/index.html)

### `regenerate_entity.py`

If you modified any database migrations and changed the definitions of any table,please run this script to regenerate database table entities.

**Notice:You must ensure the changes has been applied to your database before you run this script**

### `stress_test.py`

**Warning:Your computer may be frozen**

Run this script to run stress test.The first argument is the test flag(default is `--release`).The second one is the test command(default is `cargo run --bin stress_test`)

### `build_production_container.py`

Build production docker container

### `push_docker_images.py`

Push docker images to docker hub

## Client Usage

### `generate.pb.dart.py`

Run this script to generate dart protobuf files.

## Daily

### `pre-commit`

Run some checks(lint, format) locally before every commit,your can copy it to `.git/hooks`(if your are using linux,use `chmod +x .git/hooks/pre-commit` to make it runnable)

### `pre-commit.py`

Format your code or do other works.You can run it before you commit to prevent your code from being rejected by CI.

### `merge_and_push_main.py`

Merge the changes from `dev` into `main`,only for core developers to simplify daily chores.

## CI

## Other

### `basic.py`

Some helper functions for writing scripts

### `ci_test.sh`

Test in docker container in CI, not for local usage
