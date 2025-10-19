# Helper Scripts

Notice: If there is any script not listing below or out-of-date,please open an issue.

## Server Usage

### `db_migration.py`

As we are using `sea-orm`,you can use this to run database migrations via `sea-orm-cli`,the arguments is the same as `sea-orm-cli`

For more information,please refer to [Sea Orm](https://www.sea-ql.org/SeaORM/docs/migration/running-migration/) and [OurChat Server Development Document](https://ourchat.readthedocs.io/en/latest/docs/development/server/index.html)

### `regenerate_entity.py`

If you modified any database migrations and changed the definitions of any table,please run this script to regenerate database table entities.

**Notice:You must ensure the changes have been applied to your database before you run this script**

### `stress_test.py`

**Warning:Your computer may be frozen**

Run this script to run stress test.
The first argument is the test flag(default is `--release`).The second one is the test command(default is `cargo run --bin stress_test`)

### `build_production_container.py`

Build production docker container

## Client Usage

### `generate.pb.dart.py`

Run this script to generate dart protobuf files.

### `generate_about_code.py`

This script is to fetch the contributors and donors to compose the about page in client.

## Daily

### `pre-commit`

Run some checks(lint, format) locally before every commit, you can copy it to
`.git/hooks`(if you are using linux,use `chmod +x .git/hooks/pre-commit` to make it runnable)

### `pre-commit.py`

Format your code or do other works. You can run it before you commit to prevent your code from being rejected by CI.

### `merge_and_push_main.py`

Merge the changes from `dev` into `main`,only for core developers to simplify daily chores.

## CI

## Other

### `basic.py`

Some helper functions for writing scripts
