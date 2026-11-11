# 📋 [Board](https://pi.tail2e200c.ts.net)

Vestaboard controller.

## 🛠️ Setup

Configure `.env`:

```sh
VESTABOARD_HOST=...     # Vestaboard local hostname
VESTABOARD_KEY=...      # Vestaboard local API key
VESTABOARD_TEST=...     # 1 to enable dry-run mode
```

## 🔤 Full grid

`POST /board` shows a full grid of [character codes](https://docs.vestaboard.com/docs/charactercodes),
in the same `grid` shape `GET /board` returns: 6 rows of 22 codes. Errors name
the bad row or column, counting from 0.

A rainbow, one color per row:

```sh
grid=$(for code in 63 64 65 66 67 68; do
  printf '[%s]' "$(yes "$code" | head -22 | paste -sd,)"
done | sed 's/\]\[/],[/g')
curl -X POST http://localhost:8001/board \
  -H 'Content-Type: application/json' \
  -d "{\"grid\": [$grid]}"
```

## 🚀 Deploy

Build and install on Raspberri Pi:

```sh
deploy/deploy.sh
```
