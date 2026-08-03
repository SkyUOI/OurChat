# Helper Scripts

Notice: If there is any script not listing below or out-of-date,please open an issue.

## Server Usage

### `db_migration.py`

As we are using `sea-orm`,you can use this to run database migrations via `sea-orm-cli`,the arguments is the same as `sea-orm-cli`

For more information,please refer to [Sea Orm](https://www.sea-ql.org/SeaORM/docs/migration/running-migration/) and [OurChat Server Development Document](https://ourchat.readthedocs.io/en/latest/docs/development/server/index.html)

### `regenerate_entity.py`

If you modified any database migrations and changed the definitions of any table,please run this script to regenerate database table entities.

**Notice:You must ensure the changes have been applied to your database before you run this script**

### `build_production_container.py`

Build production docker container

## Client Usage

### `generate.pb.dart.py`

Run this script to generate dart protobuf files.

### `generate_about_code.py`

This script is to fetch the contributors and donors to compose the about page in client.

### `run_integration_tests.py`

Automatically bring up live servers and run the **client integration tests**
(`client/integration_test/*`), then tear everything down.

Prerequisites (the dev dependencies are **user-managed** — this script does not
start or stop docker):

```bash
docker compose -f docker/compose.devenv.yml up -d db redis mq
```

Usage:

```bash
# run all client integration tests (two live servers: 7777 + 7778)
python script/run_integration_tests.py

# run a single test file
python script/run_integration_tests.py --test integration_test/multi_server_test.dart

# point at a prebuilt server binary
python script/run_integration_tests.py --server-path target/debug/server

# leave servers + temp files around for debugging
python script/run_integration_tests.py --debug
```

What it does:

1. Preflight: checks `cargo`/`flutter`/`docker`, the dependency ports
   (5432/6379/5672) and that 7777/7778 are free.
2. Builds the server binary (`cargo build --bin server`).
3. Generates isolated configs for **two** server instances — the second uses
   its own rabbitmq vhost (`oc_e2e_b`, created automatically) so the two
   servers do not fight over exclusive queues.
4. Starts both servers and waits until they answer.
5. Runs the integration tests one file at a time (avoids desktop-device
   conflicts), printing a per-file summary.
6. Cleans up: kills the servers and removes the temporary configs. Docker is
   never touched.

Options:

| Flag | Description |
|---|---|
| `--test FILE` | Run a single test file (repeatable) |
| `--port-a N` / `--port-b N` | Server ports (default 7777 / 7778) |
| `--server-path PATH` | Use a prebuilt server binary instead of `cargo build` |
| `--keep-server` | Leave servers running after the tests |
| `--debug` | Keep servers + temp files, print paths |

## Daily

### `pre-commit`

Run some checks(lint, format) locally before every commit, you can copy it to
`.git/hooks`(if you are using linux,use `chmod +x .git/hooks/pre-commit` to make it runnable)

## CI

## Other

### `basic.py`

Some helper functions for writing scripts
