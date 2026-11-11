#!/usr/bin/env bash
# Run the Playwright tests against browsers in the pinned image CI uses so screenshots match the
# baselines. Arguments go to `playwright test`, e.g. `web/tests/container.sh --update-snapshots`.
# Uses Podman if it's installed, otherwise Docker.
set -euo pipefail

image=mcr.microsoft.com/playwright:v1.64.1-noble
port=3077
cd "$(dirname "$0")"

engine=docker
if command -v podman >/dev/null; then
  engine=podman
fi

network=(--publish "127.0.0.1:$port:$port")
if [ "$(uname)" = Linux ]; then
  network=(--network host)
fi
container=$("$engine" run --detach --rm --init --ipc=host --platform linux/amd64 "${network[@]}" \
  --user pwuser --workdir /home/pwuser "$image" \
  npx -y playwright@1.64.1 run-server --host 0.0.0.0 --port "$port")
trap '"$engine" stop "$container" >/dev/null' EXIT

for _ in $(seq 120); do
  if curl -s "http://127.0.0.1:$port" >/dev/null; then
    break
  fi
  sleep 1
done

# The browsers reach the test server on this machine's loopback through the connection.
PW_TEST_CONNECT_WS_ENDPOINT="ws://127.0.0.1:$port/" PW_TEST_CONNECT_EXPOSE_NETWORK="<loopback>" \
  npx playwright test "$@"
