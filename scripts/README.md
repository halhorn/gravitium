# Visual verification scripts

Playwright + Chrome で Gravitium の WebGPU 起動とスクリーンショット取得を行います。

## 前提

- Node.js 18+
- Rust 1.89+、`trunk`、`wasm32-unknown-unknown` ターゲット
- **Linux (CI / クラウドエージェント)**: `mesa-vulkan-drivers`, `xvfb`, Google Chrome

```bash
sudo apt-get install -y mesa-vulkan-drivers xvfb
```

## セットアップ

```bash
cd scripts
npm install
npx playwright install chromium
```

## 使い方

1. 別ターミナルで dev サーバーを起動:

```bash
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve --address 127.0.0.1 --port 8080
```

[`rust-toolchain.toml`](../rust-toolchain.toml) で toolchain がピン留めされます。

2. 検証を実行:

```bash
./scripts/run-visual-verification.sh
```

macOS では `cd scripts && npm run verify:view` でも可。

## 出力

スクリーンショットは `artifacts/screenshots/verify-view.png` に保存されます。

```bash
export GRAVITIUM_SCREENSHOT_DIR=/path/to/output   # 変更可
export GRAVITIUM_BASE_URL=http://127.0.0.1:8080   # 変更可
```

PR には **動作確認結果（生成したスクショ）を本文に添付**してください。

## 共有ライブラリ (`lib/`)

- `config.mjs` — URL・タイムアウト・出力先
- `browser.mjs` — WebGPU 対応 Chrome 起動
- `simulation.mjs` — ローディング待ち・状態取得
- `screenshot.mjs` — PNG 保存

## レガシー (macOS 向け)

- `verify-chrome.mjs` — 単体 Chrome スモーク（Metal ANGLE）
- `verify-browsers.mjs` — Chrome / WebKit / Safari アプリ
