#!/usr/bin/env bash

set -euo pipefail

host="${BOARD_HOST:-pi}"
target=aarch64-unknown-linux-gnu
cd "$(dirname "$0")/.."

cargo zigbuild --release --target "$target"
scp -q "target/$target/release/board" "$host:board/board.new"
scp -q deploy/board.service "$host:board/board.service"

ssh "$host" '
  set -e
  mv ~/board/board.new ~/board/board
  sudo cp ~/board/board.service /etc/systemd/system/board.service
  sudo systemctl daemon-reload
  sudo systemctl restart board
  for _ in $(seq 20); do
    curl -fs -o /dev/null http://127.0.0.1:8001/ && exit 0
    sleep 0.5
  done
  echo "board did not come back up" >&2
  systemctl status board --no-pager >&2
  exit 1
'

echo "Deployed to $host"
