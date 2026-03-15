#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup が見つかりません。先に rustup をインストールしてください。"
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo が見つかりません。Rust のインストール状態を確認してください。"
  exit 1
fi

TOOLCHAIN="$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml | head -n1)"

if [[ -z "${TOOLCHAIN}" ]]; then
  echo "rust-toolchain.toml から channel を取得できませんでした。"
  exit 1
fi

echo "==> toolchain: ${TOOLCHAIN}"
rustup toolchain install "${TOOLCHAIN}"
rustup component add rustfmt clippy llvm-tools-preview --toolchain "${TOOLCHAIN}"

echo "==> install cargo-llvm-cov"
cargo +"${TOOLCHAIN}" install cargo-llvm-cov --locked

echo "==> done"
rustc +"${TOOLCHAIN}" --version
cargo +"${TOOLCHAIN}" llvm-cov --version
