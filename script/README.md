# Helper Scripts

Notice: If there is any script not listing below or out-of-date,please open a issue.

## Server Usage

### `test_server.py`

Help to run set up test environment before `cargo test`.The first argument is the test command(default is `cargo test`).The second argument is the server config file(default is `config/gh_test/ourchat.toml`)

### `db_migration.py`

As we are using `sea-orm`,you can use this to run database migrations via `sea-orm-cli`,the arguments is the same as `sea-orm-cli`

For more information,please refer to [Sea Orm](https://www.sea-ql.org/SeaORM/docs/migration/running-migration/) and [OurChat Server Development Document](https://ourchat.readthedocs.io/en/latest/docs/development/server/index.html)

### `regenerate_entity.py`

If you modified any database migrations and changed the definitions of any table,please run this script to regenerate database table entities.

**Notice:You must ensure the changes has been applied to your database before you run this script**

### `delete_all_database.py`

**Warning:Never run in production environment!**

As every failed integration test will leave a database for debug usage,it will leave hundreds of useless db in your hard disk,run this to delete all databases to clean your disk.

### `stress_test.py`

**Warning:Your computer may be frozen**

Run this script to run stress test.The first argument is the test flag(default is `--release`).The second one is the test command(default is `cargo run --bin stress_test`)

### `rebuild_dev_container.py`

If you modified the `Dockerfile`,use it to rebuild docker image for development usage

## Daily

### `pre-commit`

Run some checks(lint, format) locally before every commit,your can copy it to `.git/hooks`(if your are using linux,use `chmod +x .git/hooks/pre-commit` to make it runnable)

### `merge_and_push_main.py`

Merge the changes from `dev` into `main`,only for core developers to simplify daily chores.

## CI

### `action_test_server.py`

Just for Github Action workflow,**should not run directly on your local machine**

### `init_valgrind_rust.py`

Helper script to [action_test_server.py](#action_test_serverpy),initialize `valgrind`(A memory error detector on Linux) to detect memory bug.**should not run directly on your local machine**

## Other

### `basic.py`

Some helper functions for writing scripts
