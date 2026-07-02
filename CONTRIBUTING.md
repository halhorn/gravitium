# Contributing

## 実装完了時の動作確認（必須）

UI・3D ビュー・シミュレーション挙動に触れた PR では、マージ前に **ビジュアル検証** を行ってください。

### 最低限

1. `trunk serve` でローカル起動
2. `./scripts/run-visual-verification.sh verify:app` が成功すること
3. 変更内容を目視で確認すること

### UI / カメラ / 初期条件など見た目が変わる場合

1. 上記に加え、該当する `scripts/verify-*.mjs` を追加または更新する
2. スクリーンショットを PR 本文に添付する（リポジトリ内パスまたは生成 PNG）
3. 期待と異なる場合は **ピクセル計測** など客観的なチェックを入れる（`verify-outer-radius-zoom.mjs` 参照）

詳細: [docs/verification.md](docs/verification.md)

## 開発環境

[README.md](README.md) の「ローカル開発」を参照。

## ブランチ

機能ブランチは `cursor/<topic>-addc` 形式を推奨。`main` から分岐してください。
