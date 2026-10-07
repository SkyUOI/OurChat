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
# NOTES:
# - The `-d web-server` device is deliberate: `-d chrome` deadlocks on
#   flutter 3.4x (drive hardcodes no-launch-chrome while the resident web
#   runner's attach waits for the chromium instance it therefore never
#   launches). With web-server, chromedriver launches and drives Chrome
#   itself, headless — no display/xvfb needed.
# - Override the Chrome binary locally via CHROME_BINARY (CI's chromedriver
#   finds the runner's Chrome on its own).
# - Each target is capped with `timeout` so a stuck session fails the job
#   instead of hanging it.
set -euo pipefail

cd "$(dirname "$0")/../client"

chrome_binary_args=()
if [ -n "${CHROME_BINARY:-}" ]; then
  chrome_binary_args=(--chrome-binary="$CHROME_BINARY")
fi

for target in web_int64_precision_test web_keygen_test web_grpcweb_error_test; do
  echo "==> driving $target"
  timeout --kill-after=30s 15m flutter drive \
    --profile \
    -d web-server \
    --driver=test_driver/integration_test.dart \
    --target="integration_test/$target.dart" \
    "${chrome_binary_args[@]}" \
    --web-browser-flag=--no-sandbox \
    --web-browser-flag=--disable-dev-shm-usage \
    --web-browser-flag=--disable-gpu
done
