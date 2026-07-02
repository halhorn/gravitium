# ビジュアル検証

Gravitium は WebGPU + WASM のため、ユニットテストだけでは UI  regressions を検知しにくいです。  
`scripts/` 配下の Playwright 検証で **起動・描画・スクリーンショット** を自動化します。

## クイックスタート

```bash
# ターミナル 1
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve --address 127.0.0.1 --port 8080

# ターミナル 2
./scripts/run-visual-verification.sh verify:app
```

成功すると `artifacts/screenshots/verify-app.png` が生成されます。

## Linux (CI / クラウドエージェント)

headless Chrome で WebGPU を使うには **lavapipe** と **xvfb** が必要です。

```bash
sudo apt-get install -y mesa-vulkan-drivers xvfb
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json
./scripts/run-visual-verification.sh verify:app
```

`run-visual-verification.sh` が上記をまとめて実行します。

## 環境変数

| 変数 | 既定 | 説明 |
|------|------|------|
| `GRAVITIUM_BASE_URL` | `http://127.0.0.1:8080` | trunk の URL |
| `GRAVITIUM_SCREENSHOT_DIR` | `<repo>/artifacts/screenshots` | PNG 出力先 |
| `GRAVITIUM_TIMEOUT_MS` | `240000` | ロード待ち ms |
| `VK_ICD_FILENAMES` | (Linux) lavapipe | SwiftShader Vulkan ICD |

## 機能別検証の追加

1. `scripts/lib/` のヘルパーを使う
2. `scripts/verify-<feature>.mjs` を作成
3. `scripts/package.json` の `scripts` に `"verify:<feature>"` を追加
4. 必要なら `./scripts/run-visual-verification.sh verify:<feature>` で実行

テンプレート: `verify-outer-radius-zoom.mjs`（URL 状態・複数スクショ・ピクセル assert）

## PR へのスクショ添付

1. 検証実行で `artifacts/screenshots/*.png` を生成
2. 永続化したいものは `planning/<area>/screenshots/` 等へコピーしてコミット
3. PR 本文で GitHub raw URL または `<img>` タグで参照

```markdown
![description](https://github.com/<owner>/gravitium/raw/<branch>/planning/.../shot.png)
```

## エージェント向けチェックリスト

実装ターンの末尾で:

- [ ] `cargo build --lib --target wasm32-unknown-unknown`（Rust 変更時）
- [ ] `./scripts/run-visual-verification.sh verify:app`
- [ ] 見た目に関わる変更なら feature 別 verify + スクショ
- [ ] PR 更新（スクショ・計測結果の記載）
