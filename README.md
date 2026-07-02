# Gravitium

WebGPU 上で **10,000 体** の N 体重力を GPU compute で動かす 3D シミュレータです。Bevy + WebGPU + Trunk でブラウザ専用に構成しています。

https://halhorn.github.io/gravitium/

## 試してみる

WebGPU 対応ブラウザ（Chrome / Edge 推奨）で上記 URL を開いてください。マウスでオービットカメラを操作できます。

WebGPU 非対応のブラウザでは CPU フォールバックはなく、起動できない旨のメッセージが表示されます。

## ローカル開発

[`rustup`](https://rustup.rs/) で Rust **1.89 以上**（edition 2024）を入れ、wasm ターゲットと Trunk を用意します。  
[`rust-toolchain.toml`](rust-toolchain.toml) で toolchain がピン留めされます。

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
git clone https://github.com/halhorn/gravitium.git
cd gravitium
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve
```

`Trunk.toml` に従い、既定では `http://127.0.0.1:8080/` で待ち受けます。  
`.cargo/config.toml` にも `web_sys_unstable_apis` の rustflags が設定されています。

本番相当のビルド:

```bash
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk build --release --public-url /gravitium/
```

GitHub Pages では `/gravitium/` サブパス配下に公開するため、`--public-url /gravitium/` は必須です。

## 動作確認（ビジュアル検証）

UI や 3D 表示に変更がある場合は、マージ前にスクリーンショット付きのビジュアル検証を行います。

```bash
# trunk serve 起動後
./scripts/run-visual-verification.sh
```

詳細は [docs/verification.md](docs/verification.md) を参照してください。

## 対応ブラウザ

| ブラウザ | 備考 |
|----------|------|
| Chrome / Edge | 推奨 |
| Firefox | WebGPU 実装状況に依存（要最新版） |
| Safari | WebGPU 実装状況に依存（要最新版） |
