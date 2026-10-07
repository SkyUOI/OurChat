#!/usr/bin/env bash
# Drive the web-only Flutter integration tests in a real Chrome via
# `flutter drive --profile` (flutter test does not support web devices).
#
# Requires a chromedriver listening on port 4444 matching the installed
# Chrome, e.g.:
#   chromedriver --port=4444 &
#   ./script/drive_web_integration_tests.sh
#
# Used by the flutter-web-integration CI job (.github/workflows/client_ci.yml);
# web_grpcweb_error_test.dart skips itself when no server is reachable.
#
# Chrome runs headless (flutter drive's default). On CI runners the sandbox
# must be disabled and /dev/shm usage limited, or Chrome hangs at launch —
# hence the --web-browser-flag entries. Each target is capped with `timeout`
# so a stuck session fails the job instead of hanging it.
set -euo pipefail

cd "$(dirname "$0")/../client"

for target in web_int64_precision_test web_keygen_test web_grpcweb_error_test; do
  echo "==> driving $target"
  timeout --kill-after=30s 15m flutter drive \
    --profile \
    --headless \
    --driver=test_driver/integration_test.dart \
    --target="integration_test/$target.dart" \
    -d chrome \
    --web-browser-flag=--no-sandbox \
    --web-browser-flag=--disable-dev-shm-usage \
    --web-browser-flag=--disable-gpu
done
