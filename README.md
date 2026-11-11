# 📋 [Board](https://pi.tail2e200c.ts.net)

Vestaboard controller.

## 🛠️ Setup

Configure `.env` or the environment:

```sh
VESTABOARD_HOST=...     # Vestaboard local hostname
VESTABOARD_KEY=...      # Vestaboard local API key
VESTABOARD_TEST=...     # 1 to enable dry-run mode
```

## 🧪 Test

Layout tests run against browsers in Playwright's Docker image so screenshots match CI:

```sh
(cd web/tests && npm ci)
web/tests/run.sh                     # Check screenshots
web/tests/run.sh --update-snapshots  # Regenerate screenshots
```

## 🚀 Deploy

Build and install on Raspberri Pi:

```sh
deploy/deploy.sh
```
