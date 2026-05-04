# 排他制御

## 概要

サーバーには 2 段階の排他制御機構がある。

1. **インスタンスロック** — 複数プロセスの同時起動防止
2. **`game_update_lock`** — プロセス内のゲーム状態更新の直列化

---

## 1. インスタンスロック

### 目的

同一マシン上で複数の `moe3-server` プロセスが同時に起動することを防ぐ。

### 仕組み

- サーバー起動時（`serve()`）に `<MAIN_DATABASE_PATH>.lock` というファイルを作成し、OS の排他ファイルロック（`flock(2)` 相当）を取得する。
- ロック取得に失敗した場合（`WouldBlock`）は起動を中断し、エラーメッセージを返す。
- ロックファイルは `INSTANCE_LOCK` グローバル変数（`Mutex<Option<Arc<File>>>`）が保持するため、プロセスが終了するまで解放されない。

### 実装箇所

- `server/src/api/router.rs` — `acquire_instance_lock()` / `INSTANCE_LOCK`

---

## 2. `game_update_lock`

### 目的

複数の HTTP リクエストが同時にゲーム状態を読み書きした場合の競合を防ぐ。
特に「定時進行（グローバルプリハンドラによる自動進行）」とプレイヤー操作が衝突しないよう直列化する。

### 実装

`AppState` が保持する `Arc<TokioMutex<()>>` フィールド `game_update_lock` を各ハンドラで `.lock().await` することで実現する。

```rust
let _game_update_guard = state.game_update_lock.lock().await;
// ゲーム状態を更新する処理 ...
drop(_game_update_guard);
```

ガードを `drop()` するまで、他のロック取得者はブロックされる。

### ロックを取得するタイミング

| 場所 | ロック取得 | ロック解放 | 備考 |
|---|---|---|---|
| グローバルプリハンドラ（`run_global_pre_handler`） | `pre_handler.run()` 呼び出し前 | `pre_handler.run()` 完了後（ハンドラ実行前） | 自動進行（定時更新）を直列化 |
| `PUT /admin/games/:game_uuid/draw-proposal` | ハンドラ本体の先頭 | ハンドラ本体の末尾 | 講和フラグ更新 |
| `PUT /admin/games/:game_uuid/units` | ハンドラ本体の先頭 | ハンドラ本体の末尾 | ユニット配置・削除 |

### ロックを取得しないエンドポイント

以下のエンドポイントはゲーム状態を直接変更しないか、競合の影響が軽微なため `game_update_lock` を取得しない。

| エンドポイント | 理由 |
|---|---|
| `POST /auth/login` | ユーザー情報のみ更新。ゲーム状態に触れない |
| `POST /games` | 新規卓作成のみ。既存ゲーム状態を変更しない |
| `POST /games/:game_uuid/players` | プレイヤー追加。競合時は DB の Conflict エラーで安全に処理される |

### グローバルプリハンドラと通常ハンドラの関係

```
リクエスト受信
    │
    ▼
run_global_pre_handler
    │ [1] last_access_at 更新（ロック外）
    │ [2] game_update_lock 取得
    │ [3] GlobalPreHandler::run()（定時進行）
    │ [4] game_update_lock 解放
    │
    ▼
各ハンドラ実行（ロック外）
    │ ※ draw-proposal / units は再度 game_update_lock を取得
    ▼
レスポンス返却
```

プリハンドラはロックを解放してからハンドラに制御を渡す。draw-proposal・units ハンドラは再度ロックを取得するため、同一リクエスト内で定時進行と操作が衝突することはない。

---

## 注意事項

- `game_update_lock` はプロセス内のスレッド間排他であり、マルチプロセス構成には対応していない。マルチプロセス構成が必要になった場合は DB レベルの排他制御（行ロック等）に置き換える必要がある。
- インスタンスロックと組み合わせることで、現状のシングルプロセス構成では競合が発生しないことを保証している。
