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

if [[ ! -f rust-toolchain.toml ]]; then
  echo "rust-toolchain.toml が見つかりません。"
  exit 1
fi

TOOLCHAIN="$(grep -m1 '^channel' rust-toolchain.toml | sed -E 's/.*\"([^\"]+)\".*/\1/' || true)"

if [[ -z "${TOOLCHAIN:-}" ]]; then
  echo "rust-toolchain.toml から channel を取得できませんでした。"
  exit 1
fi

echo "==> toolchain: ${TOOLCHAIN}"

if rustup toolchain list | grep -q -E \"^${TOOLCHAIN}\(\\s|$\)\"; then
  echo "toolchain ${TOOLCHAIN} は既にインストールされています。"
else
  echo "==> installing toolchain ${TOOLCHAIN}"
  rustup toolchain install "${TOOLCHAIN}"
fi

echo "==> adding components (rustfmt, clippy, llvm-tools-preview)"
rustup component add rustfmt clippy llvm-tools-preview --toolchain "${TOOLCHAIN}"

if command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov は既にインストールされています。"
else
  echo "==> install cargo-llvm-cov"
  cargo +"${TOOLCHAIN}" install cargo-llvm-cov --locked || {
    echo "cargo-llvm-cov のインストールに失敗しました（手動で再試行してください）。"
  }
fi

echo "==> done"
rustc +"${TOOLCHAIN}" --version || true
cargo +"${TOOLCHAIN}" llvm-cov --version || true
