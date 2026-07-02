# Agent instructions

## Visual verification (required before finishing UI / 3D work)

When you change simulation rendering, camera, controls, or initial conditions:

1. Run `trunk serve` on `http://127.0.0.1:8080` (or use an already-running instance).
2. Run `./scripts/run-visual-verification.sh verify:app` and ensure it exits 0.
3. For visual features, add or run a targeted script under `scripts/verify-*.mjs`.
4. Attach screenshots to the PR when behavior is user-visible.

See [docs/verification.md](docs/verification.md) and [scripts/README.md](scripts/README.md).

## Rust

- Edition 2024, Rust 1.89+
- WASM build: `rustup run nightly cargo build --lib --target wasm32-unknown-unknown`

## Branches

Create feature branches from `main` using `cursor/<descriptive-name>-addc`.
