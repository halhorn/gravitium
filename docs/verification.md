# ビジュアル検証

Gravitium は WebGPU + WASM のため、ユニットテストだけでは UI の regressions を検知しにくいです。  
`scripts/verify-view.mjs` で **起動・描画・スクリーンショット** を自動確認します。

## クイックスタート

```bash
# ターミナル 1
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve --address 127.0.0.1 --port 8080

# ターミナル 2
./scripts/run-visual-verification.sh
```

成功すると `artifacts/screenshots/verify-view.png` が生成されます。

## Linux (CI / クラウドエージェント)

headless Chrome で WebGPU を使うには **lavapipe** と **xvfb** が必要です。

```bash
sudo apt-get install -y mesa-vulkan-drivers xvfb
./scripts/run-visual-verification.sh
```

`run-visual-verification.sh` が xvfb + lavapipe をまとめて使います。

## 環境変数

| 変数 | 既定 | 説明 |
|------|------|------|
| `GRAVITIUM_BASE_URL` | `http://127.0.0.1:8080` | trunk の URL |
| `GRAVITIUM_SCREENSHOT_DIR` | `<repo>/artifacts/screenshots` | PNG 出力先 |
| `GRAVITIUM_TIMEOUT_MS` | `240000` | ロード待ち ms |

## 実装完了時

UI / 3D ビューに触れた PR では、マージ前に `./scripts/run-visual-verification.sh` を実行し、必要ならスクショを PR に添付してください。
