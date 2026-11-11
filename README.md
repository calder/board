# 📋 [Board](https://pi.tail2e200c.ts.net)

Vestaboard controller.

## 🛠️ Setup

Configure `.env` in `/home/calder/board` on the Pi:

```sh
VESTABOARD_HOST=...     # Vestaboard local hostname (unset for dry-run mode)
VESTABOARD_KEY=...      # Vestaboard local API key
```

## 🧪 Test

Layout tests run against browsers in Playwright's Docker image so screenshots match CI:

```sh
(cd web/tests && npm ci)
web/tests/run.sh                     # Check screenshots
web/tests/run.sh --update-snapshots  # Regenerate screenshots
```

## 🚀 Deploy

Merges to `main` build an aarch64 Linux binary (web assets embedded) and
publish it as a GitHub Release. Deploying is a separate, manual step over
Tailscale SSH, so nothing restarts on the Pi by itself.

From a laptop on the tailnet:

```sh
deploy/deploy.sh                  # latest release
deploy/deploy.sh v20261111-abc123 # a specific release
deploy/deploy.sh --rollback       # previous binary
```

The script downloads the release on the Pi, checks the SHA-256, replaces
`/home/calder/board/board` atomically, keeps the previous binary for
rollback, installs `deploy/board.service` as the `board` systemd unit
(user `calder`, working directory `/home/calder/board`), restarts the
service, and prints its status.

`BOARD_HOST` defaults to `pi`. Set it to `pi.tail2e200c.ts.net` if that
name is what you use. The script never touches `.env`.

First-time only: after the first `deploy/deploy.sh`, the unit is enabled
at boot. Make sure `.env` already exists in `/home/calder/board`.
