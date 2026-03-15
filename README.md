# moe3-server-rust

Diplomacy MOE 3 Server の Rust 実装。

## 概要

このリポジトリは Rust ワークスペースで、現在は `server` クレートを含みます。
`server` はゲームのドメインモデル（注文、ユニット、国家、州など）を中心に実装されています。

## 構成

- `Cargo.toml`（ルート）: workspace 定義
- `server/`
  - `Cargo.toml`: `server` クレート定義
  - `src/lib.rs`: `domain` モジュールを公開
  - `src/domain/`: ドメインモデル
    - `order.rs`, `unit.rs`, `province.rs`, `power.rs`, `player.rs`, `phase.rs`, `path.rs`, `table.rs`, `user.rs`

<!-- ...existing code... -->

## 使い方

### 前提

- rustup
- `rust-toolchain.toml` により指定ツールチェーン（`1.94.0`）が適用されます
- 初回セットアップ（推奨）

```bash
bash scripts/bootstrap.sh
```

> `bootstrap.sh` は以下を実行します:
> - ツールチェーン `1.94.0` のインストール
> - `rustfmt` / `clippy` / `llvm-tools-preview` の導入
> - `cargo-llvm-cov` のインストール

### ビルド

```bash
cargo build
```

### テスト

```bash
cargo test
```

### カバレッジ（要約のみ）

```bash
cargo llvm-cov --summary-only
```

## 主要ドメイン型（`server::domain`）

- `Order`
- `Path`
- `Player`
- `Power`
- `Province`
- `Table`
- `Unit`（`Army`, `Fleet`）
- `User`

## 開発メモ

- ID 型は `domain.rs` で type alias として定義されています（`OrderId`, `PhaseId` など）。
- `Province` は州コード（文字列）から安全に生成する API（`from_code`）を持ちます。
