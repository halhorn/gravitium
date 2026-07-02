# Contributing

GitHub が PR 作成時などにリンクする、**人間向け**の開発ガイドです（[GitHub Docs: リポジトリへの貢献](https://docs.github.com/ja/communities/setting-up-your-project-for-healthy-contributions/setting-guidelines-for-repository-contributors)）。

## ビジュアル検証

UI・3D ビューに変更がある PR では、マージ前に以下を行ってください。

1. `trunk serve` でローカル起動
2. `./scripts/run-visual-verification.sh` が成功すること
3. 変更内容を目視で確認し、必要ならスクショを PR に添付

手順の詳細: [docs/verification.md](docs/verification.md)

## 開発環境

[README.md](README.md) の「ローカル開発」を参照。
