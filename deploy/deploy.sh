#!/usr/bin/env bash
# Deploy a GitHub Release of board to the Pi over SSH.
set -euo pipefail

host="${BOARD_HOST:-pi}"
repo="${BOARD_REPO:-calder/board}"
asset="board-aarch64-unknown-linux-gnu"
# Installed path on the Pi, matching deploy/board.service.
remote_dir="board"

usage() {
  cat <<USAGE
Usage: deploy/deploy.sh [version]
       deploy/deploy.sh --rollback

Install a board release on ${host} over SSH.
With no version, uses the latest GitHub Release.

Environment:
  BOARD_HOST   SSH host (default: pi)
  BOARD_REPO   GitHub repo (default: calder/board)
USAGE
}

die() {
  echo "$*" >&2
  exit 1
}

latest_tag() {
  local json tag
  json=$(curl -fsSL -H "Accept: application/vnd.github+json" \
    "https://api.github.com/repos/${repo}/releases/latest") \
    || die "couldn't fetch the latest release for ${repo}"
  tag=$(printf '%s\n' "${json}" | grep -m1 '"tag_name"' \
    | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
  [[ -n ${tag} && ${tag} != *tag_name* ]] || die "couldn't parse tag_name from the GitHub API"
  printf '%s\n' "${tag}"
}

# Run a script on the Pi, passing selected variables as the remote environment.
# The printf is expanded on the laptop so the values are quoted for the remote shell.
# shellcheck disable=SC2029
remote() {
  ssh "${host}" "env $(printf 'REPO=%q ASSET=%q VERSION=%q REMOTE_DIR=%q' \
    "${repo}" "${asset}" "${version:-}" "${remote_dir}") bash -s"
}

rollback() {
  echo "Rolling back on ${host}"
  version="" remote <<'REMOTE'
set -euo pipefail
cd "${HOME}/${REMOTE_DIR}"
if [[ ! -f board.prev ]]; then
  echo "no previous binary at ${HOME}/${REMOTE_DIR}/board.prev" >&2
  exit 1
fi
tmp=$(mktemp ./board.rollback.XXXXXX)
mv board "${tmp}"
mv board.prev board
mv "${tmp}" board.prev
sudo systemctl restart board
for _ in {1..20}; do
  if curl -fs -o /dev/null http://127.0.0.1:8001/; then
    systemctl status board --no-pager
    exit 0
  fi
  sleep 0.5
done
echo "board did not come back up" >&2
systemctl status board --no-pager >&2 || true
exit 1
REMOTE
  echo "Rolled back on ${host}"
}

deploy() {
  version=$1
  echo "Deploying ${version} to ${host}"
  remote <<'REMOTE'
set -euo pipefail
dir="${HOME}/${REMOTE_DIR}"
mkdir -p "${dir}"
work=$(mktemp -d)
trap 'rm -rf "${work}"' EXIT
base="https://github.com/${REPO}/releases/download/${VERSION}"
curl -fsSL -o "${work}/${ASSET}" "${base}/${ASSET}"
curl -fsSL -o "${work}/${ASSET}.sha256" "${base}/${ASSET}.sha256"
curl -fsSL -o "${work}/board.service" "${base}/board.service"
(cd "${work}" && sha256sum -c "${ASSET}.sha256")
install -m 755 "${work}/${ASSET}" "${dir}/board.new"
if [[ -e ${dir}/board ]]; then
  cp -a "${dir}/board" "${dir}/board.prev"
fi
mv -f "${dir}/board.new" "${dir}/board"
sudo cp "${work}/board.service" /etc/systemd/system/board.service
sudo systemctl daemon-reload
sudo systemctl enable board
sudo systemctl restart board
for _ in {1..20}; do
  if curl -fs -o /dev/null http://127.0.0.1:8001/; then
    systemctl status board --no-pager
    exit 0
  fi
  sleep 0.5
done
echo "board did not come back up" >&2
systemctl status board --no-pager >&2 || true
exit 1
REMOTE
  echo "Deployed ${version} to ${host}"
}

case "${1:-}" in
  -h | --help)
    usage
    exit 0
    ;;
  --rollback)
    rollback
    ;;
  -*)
    die "unknown option: $1"
    ;;
  "")
    tag=$(latest_tag)
    deploy "${tag}"
    ;;
  *)
    deploy "$1"
    ;;
esac
