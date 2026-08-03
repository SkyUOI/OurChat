#!/bin/sh
set -eu

# Runtime directories are bind-mounted from the host, so their ownership is
# whatever the host gives us. As root we can fix ownership before dropping
# privileges, which lets `docker compose up` work out of the box on any host
# without any manual chown.
chown -R ourchat:ourchat /etc/ourchat /app/log /app/files_storage

# Drop to the unprivileged `ourchat` user and exec the real command.
if command -v su-exec >/dev/null 2>&1; then
    exec su-exec ourchat "$@"
elif command -v gosu >/dev/null 2>&1; then
    exec gosu ourchat "$@"
else
    echo "error: neither su-exec nor gosu is available" >&2
    exit 1
fi
