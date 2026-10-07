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

# Smoke-test chromedriver directly: create and delete a headless session.
# Isolates "chromedriver/chrome is broken on this runner" (hangs below 60s)
# from "the flutter-side webdriver session is broken".
SESSION=$(curl -s -m 60 -X POST http://127.0.0.1:4444/session \
  -H 'Content-Type: application/json' \
  -d '{"capabilities":{"alwaysMatch":{"browserName":"chrome","goog:chromeOptions":{"args":["--headless","--no-sandbox","--disable-dev-shm-usage","--disable-gpu"]}}}}')
echo "chromedriver session smoke: ${SESSION:0:200}"
SID=$(echo "$SESSION" | python3 -c 'import json,sys; print(json.load(sys.stdin)["value"].get("sessionId",""))' 2>/dev/null || true)
if [ -n "$SID" ]; then
  curl -s -m 30 -X DELETE "http://127.0.0.1:4444/session/$SID" >/dev/null
  echo "chromedriver session smoke OK"
else
  echo "WARNING: chromedriver session smoke failed (exit $?); drives below will likely hang"
fi

for target in web_int64_precision_test web_keygen_test web_grpcweb_error_test; do
  echo "==> driving $target"
  timeout --kill-after=30s 15m flutter drive \
    --verbose \
    --profile \
    --headless \
    --driver=test_driver/integration_test.dart \
    --target="integration_test/$target.dart" \
    -d chrome \
    --web-browser-flag=--no-sandbox \
    --web-browser-flag=--disable-dev-shm-usage \
    --web-browser-flag=--disable-gpu
done
