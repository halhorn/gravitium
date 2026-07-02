# Agent instructions

## Visual verification (required before finishing UI / 3D work)

1. Run `trunk serve` on `http://127.0.0.1:8080` (or reuse a running instance).
2. Run `./scripts/run-visual-verification.sh` and ensure it exits 0.
3. Attach the screenshot to the PR when behavior is user-visible.

See [docs/verification.md](docs/verification.md) and [scripts/README.md](scripts/README.md).

## Rust

- Edition 2024 via [`rust-toolchain.toml`](rust-toolchain.toml)
- WASM build: `cargo build --lib --target wasm32-unknown-unknown`

## Branches

Create feature branches from `main` using `cursor/<descriptive-name>-addc`.
