# 📋 [Board](https://pi.tail2e200c.ts.net)

Vestaboard controller.

## 🛠️ Setup

Configure `.env`:

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

Build and install on Raspberri Pi:

```sh
deploy/deploy.sh
```
