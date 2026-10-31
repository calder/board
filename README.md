# 📋 Board

Home Vestaboard controller.

## 📖 Usage

```sh
cargo run --release -- show "Hello World"   # Show text and exit
cargo run --release -- serve                 # Run the web page and API
tailscale serve --bg 8001                    # Share it on the tailnet
```

Open https://air.tail2e200c.ts.net (or http://localhost:8001) to preview and send messages.

Anyone on the tailnet can also use the API directly:

```sh
POST /message   {"text": "Hi", "pad": true, "background": "rainbow"}
```

## 🛠️ Setup

Building the server also builds the web page it embeds, which needs:

```sh
rustup target add wasm32-unknown-unknown && brew install trunk
```

While working on the page, `trunk serve` in `web/` reloads on save and forwards sends to `board serve`.

Add keys to `.env`:

```sh
VESTABOARD_HOST=...     # Vestaboard local hostname
VESTABOARD_KEY=...      # Vestaboard local API key
VESTABOARD_TEST=...     # 1 to enable dry-run mode
```
