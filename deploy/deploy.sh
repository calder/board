#!/usr/bin/env bash

set -euo pipefail

host="${BOARD_HOST:-pi}"
target=aarch64-unknown-linux-gnu
cd "$(dirname "$0")/.."

# Runs on the Pi. Single-quoted so nothing expands on the laptop.
# shellcheck disable=SC2016
remote_common='
set -euo pipefail
cd ~/board

wait_for_board() {
  for _ in $(seq 20); do
    if curl -fs -o /dev/null http://127.0.0.1:8001/; then
      systemctl status board --no-pager
      return 0
    fi
    sleep 0.5
  done
  echo "board did not come back up" >&2
  systemctl status board --no-pager >&2
  return 1
}

restore_prev() {
  test -f board.prev || { echo "no previous binary at ~/board/board.prev" >&2; return 1; }
  tmp=$(mktemp ./board.rollback.XXXXXX)
  mv board "$tmp"
  mv board.prev board
  mv "$tmp" board.prev
  if [[ -f state/state.json.prev ]]; then
    mv state/state.json state/state.json.tmp 2>/dev/null || true
    mv state/state.json.prev state/state.json
    if [[ -e state/state.json.tmp ]]; then
      mv state/state.json.tmp state/state.json.prev
    fi
  fi
  sudo systemctl restart board
  wait_for_board
}
'

# shellcheck disable=SC2016
remote_deploy='
if [[ -e board ]]; then
  cp -a board board.prev
fi
if [[ -e state/state.json ]]; then
  cp -a state/state.json state/state.json.prev
fi
mv board.new board
sudo cp ~/board/board.service /etc/systemd/system/board.service
sudo systemctl daemon-reload
sudo systemctl restart board
if wait_for_board; then
  exit 0
fi
if [[ ! -f board.prev ]]; then
  exit 1
fi
echo "Rolling back to the previous binary and state" >&2
if restore_prev; then
  echo "Rolled back after a failed health check" >&2
else
  echo "board did not come back up after rollback" >&2
fi
exit 1
'

rollback() {
  # The helper snippets are expanded on the laptop and sent as the remote script.
  # shellcheck disable=SC2029
  ssh "$host" "${remote_common}
restore_prev
"
  echo "Rolled back on $host"
}

case "${1:-}" in
  --rollback)
    rollback
    exit 0
    ;;
  "")
    ;;
  *)
    echo "Usage: deploy/deploy.sh [--rollback]" >&2
    exit 1
    ;;
esac

cargo zigbuild --release --target "$target"
scp -q "target/$target/release/board" "$host:board/board.new"
scp -q deploy/board.service "$host:board/board.service"

# shellcheck disable=SC2029
ssh "$host" "${remote_common}${remote_deploy}"

echo "Deployed to $host"
