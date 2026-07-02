# Visual verification scripts

Playwright + Chrome で Gravitium の WebGPU 起動とスクリーンショット取得を行います。

## 前提

- Node.js 18+
- Rust 1.89+、`trunk`、`wasm32-unknown-unknown` ターゲット
- **Linux (CI / クラウドエージェント)**: `mesa-vulkan-drivers`, `xvfb`, Google Chrome

```bash
# Linux の追加パッケージ例
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

Rust 1.89+（edition 2024）が必要です。[`rust-toolchain.toml`](../rust-toolchain.toml) で toolchain がピン留めされます。

2. 検証を実行:

```bash
# Linux: xvfb + lavapipe で headless WebGPU
./scripts/run-visual-verification.sh verify:app

# 機能別（outer radius カメラズーム）
./scripts/run-visual-verification.sh verify:outer-radius
```

macOS では `run-visual-verification.sh` なしで `cd scripts && npm run verify:app` でも可。

## 出力

スクリーンショットは既定で `artifacts/screenshots/` に保存されます。

```bash
export GRAVITIUM_SCREENSHOT_DIR=/path/to/output   # 変更可
export GRAVITIUM_BASE_URL=http://127.0.0.1:8080   # 変更可
```

PR に添付するときは、必要な PNG を `planning/<feature>/screenshots/` などにコピーしてコミットし、本文から参照してください。

## スクリプト一覧

| コマンド | 用途 |
|----------|------|
| `npm run verify:app` | 起動・WebGPU・キャンバス描画のスモークテスト + 1 枚スクショ |
| `npm run verify:outer-radius` | outer radius スライダー + Restart でズーム差分（3 枚スクショ） |

## 共有ライブラリ (`lib/`)

- `config.mjs` — URL・タイムアウト・出力先
- `browser.mjs` — WebGPU 対応 Chrome 起動
- `simulation.mjs` — ローディング待ち・状態取得
- `screenshot.mjs` — PNG 保存・簡易ピクセル計測

新しい UI / 3D 変更向けの検証は、`verify-outer-radius-zoom.mjs` をテンプレートに `verify-<feature>.mjs` を追加してください。

## レガシー (macOS 向け)

- `verify-chrome.mjs` — 単体 Chrome スモーク（Metal ANGLE）
- `verify-browsers.mjs` — Chrome / WebKit / Safari アプリ
