#!/usr/bin/env bash
# Validate every docker compose file renders (issue #336/#337/#342).
#
# Catches: broken include chains, YAML errors, invalid volume/port
# definitions and services that lost their healthcheck/depends_on during a
# refactor. Runs `docker compose config` which merges the full graph without
# starting anything.

set -euo pipefail
cd "$(dirname "$0")/.."

status=0
for file in \
    docker/compose.yml \
    docker/compose.debian.yml \
    docker/compose.test-alpine.yml \
    docker/compose.test-debian.yml \
    docker/compose.devenv.yml; do
    if docker compose -f "$file" config --quiet; then
        echo "OK      $file"
    else
        echo "FAILED  $file"
        status=1
    fi
done

exit "$status"
