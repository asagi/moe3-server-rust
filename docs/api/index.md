# API 一覧

## 認証・ユーザー管理

| メソッド | パス | 概要 |
| --- | --- | --- |
| `POST` | [`/auth/login` 🔗](./auth-login__post.md) | ログイン（自動ユーザー生成） |
| `PUT` | `/auth/token` | トークンリセット |
| `GET` | `/users/me` | 自身のユーザー情報取得 |

## 募集と応募

| メソッド | パス | 概要 |
| --- | --- | --- | --- |
| `GET` | `/games` | 卓一覧取得 |
| `POST` | [`/games` 🔗](./games__post.md) | 新規卓作成 |
| `GET` | `/games/:game_uuid` | 卓情報取得 |
| `GET` | `/games/:game_uuid/logs/:turn` | 外交履歴取得 |
| `GET` | `/games/:game_uuid/result/:turn` | 命令解決履歴取得 |
| `POST` | [`/games/:game_uuid/players` 🔗](./games-players__post.md) | ゲームへの参加 |

## ゲーム進行・命令

| メソッド | パス | 概要 |
| --- | --- | --- | --- |
| `GET` | `/games/:game_uuid/state` | 盤面情報の取得 |
| `GET` | `/games/:game_uuid/orders` | 自身の提出済み命令確認（他国軍に対する仮定命令も含む） |
| `GET` | `/games/:game_uuid/orders/:order_id` | 命令詳細取得 |
| `PUT` | `/games/:game_uuid/orders/:order_id` | 命令変更 |
| `POST` | `/games/:game_uuid/orders` | 仮想命令・建造命令登録 |
| `DELETE` | `/games/:game_uuid/orders/:order_id` | 仮想命令・建造命令削除 |
| `PUT` | `/games/:game_uuid/progress-consensus` | 即時進行への合意を表明/撤回 |

## 卓主権限

| メソッド | パス | 概要 |
| --- | --- | --- | --- |
| `PUT` | [`/admin/games/:game_uuid/units/:location` 🔗](./admin-games-units__put.md) | ユニットの登録、変更 |
| `DELETE` | [`/admin/games/:game_uuid/units/:location` 🔗](./admin-games-units__delete.md) | ユニットの削除 |
| `PUT` | [`/admin/games/:game_uuid/territories/:code` 🔗](./admin-games-territories__put.md) | 占領情報の登録、変更 |
| `DELETE` | [`/admin/games/:game_uuid/territories/:code` 🔗](./admin-games-territories__delete.md) | 占領情報の削除 |
| `PUT` | [`/admin/games/:game_uuid/progress-mode` 🔗](./admin-games-progress-mode__put.md) | 進行モードを合意進行に切り替える |
| `PUT` | `/admin/games/:game_uuid/next-update-at` | 次回更新時刻を変更する |
| `PUT` | [`/admin/games/:game_uuid/draw-proposal` 🔗](./admin-games-draw-proposal__put.md) | 和平終了フラグを変更する |

---

### 仕様詳細
* **認証**: HTTPヘッダー `Authorization: Bearer <token>` を使用。
* **ログアウト**: フロントエンド側で保持しているトークンを破棄することで完結（サーバー側エンドポイントは不要）。
* **ユーザー管理**:
    * `/auth/login` 実行時、Discord IDがDBになければ自動生成。
    * 既存ユーザーの場合は、Discord側の最新の表示名（global_name）でDBを更新。
* **トークン**:
    * 意味を持たない一意なランダム文字列（Opaque Token）。
    * フロントエンド側で `localStorage` 等に保存して利用。
* **アバター（アイコン）**:
    * `/auth/login` 時に Discord から `avatar` ハッシュを取得しDBに保存。
