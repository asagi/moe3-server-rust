# API 一覧

## 認証・ユーザー管理

| メソッド | パス                                      | 概要                         |
| -------- | ----------------------------------------- | ---------------------------- |
| `POST`   | [`/auth/login` 🔗](./auth-login__post.md) | ログイン（自動ユーザー生成） |
| `POST`   | [`/auth/token` 🔗](./auth-token__post.md) | トークンリセット             |
| `GET`    | [`/users/me` 🔗](./users-me__get.md)      | 自身のユーザー情報取得       |

## 募集と応募

| メソッド | パス                                                                                | 概要                          |
| -------- | ----------------------------------------------------------------------------------- | ----------------------------- |
| `GET`    | [`/games` 🔗](./games__get.md)                                                      | 卓一覧取得                    |
| `POST`   | [`/games` 🔗](./games__post.md)                                                     | 新規卓作成                    |
| `GET`    | [`/games/:game_uuid` 🔗](./games-game-uuid__get.md)                                 | 卓情報取得                    |
| `GET`    | [`/games/:game_uuid/logs/:season` 🔗](./games-game-uuid-logs-season__get.md)        | 外交履歴取得                  |
| `GET`    | [`/games/:game_uuid/result/:season` 🔗](./games-game-uuid-result-season__get.md)    | 命令解決履歴取得              |
| `GET`    | [`/games/:game_uuid/board` 🔗](./games-game-uuid-board__get.md)                     | 盤面と国ごとの状況の取得      |
| `POST`   | [`/games/:game_uuid/messages` 🔗](./games-game-uuid-messages__post.md)              | 発言の投稿                    |
| `GET`    | [`/games/:game_uuid/messages/stream` 🔗](./games-game-uuid-messages-stream__get.md) | 発言のリアルタイム配信（SSE） |
| `POST`   | [`/games/:game_uuid/players` 🔗](./games-players__post.md)                          | ゲームへの参加                |

## ゲーム進行・命令

| メソッド | パス                                                                            | 概要                                                   |
| -------- | ------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `GET`    | `/games/:game_uuid/state`                                                       | 盤面情報の取得                                         |
| `GET`    | [`/games/:game_uuid/orders` 🔗](./games-game-uuid-orders__get.md)                | 自分の命令と命令の候補の取得（他国軍に対する仮定命令も含む） |
| `PUT`    | [`/games/:game_uuid/orders` 🔗](./games-game-uuid-orders__put.md)                | 自分の命令をまとめて確定（丸ごと置き換え）             |
| `PUT`    | [`/games/:game_uuid/progress-consensus` 🔗](./games-progress-consensus__put.md) | 即時進行への合意を表明/撤回                            |

## 卓主権限

| メソッド | パス                                                                                   | 概要                               |
| -------- | -------------------------------------------------------------------------------------- | ---------------------------------- |
| `PUT`    | [`/admin/games/:game_uuid/units/:location` 🔗](./admin-games-units__put.md)            | ユニットの登録、変更（管理者も可） |
| `DELETE` | [`/admin/games/:game_uuid/units/:location` 🔗](./admin-games-units__delete.md)         | ユニットの削除（管理者も可）       |
| `PUT`    | [`/admin/games/:game_uuid/territories/:code` 🔗](./admin-games-territories__put.md)    | 占領情報の登録、変更（管理者も可） |
| `DELETE` | [`/admin/games/:game_uuid/territories/:code` 🔗](./admin-games-territories__delete.md) | 占領情報の削除（管理者も可）       |
| `PUT`    | [`/admin/games/:game_uuid/progress-mode` 🔗](./admin-games-progress-mode__put.md)      | 進行モードを合意進行に切り替える   |
| `PUT`    | [`/admin/games/:game_uuid/next-update-at` 🔗](./admin-games-next-update-at__put.md)    | 次回更新時刻を延長する             |
| `PUT`    | [`/admin/games/:game_uuid/draw-proposal` 🔗](./admin-games-draw-proposal__put.md)      | 和平終了フラグを変更する           |

---

### 仕様詳細

- **認証**: HTTPヘッダー `Authorization: Bearer <token>` を使用。
- **管理者による成り代わり**:
  - ユーザーは管理者フラグ（`users.is_admin`、デフォルト `false`）を持つ。設定は DB を直接更新して行う。
  - 管理者ユーザーのリクエストに HTTPヘッダー `X-Impersonate-User-Name: <username>` が付与されている場合、`users.username`（Discord ユーザー名）が一致するユーザーの権限で API を実行する。未設定（または空）の場合は管理者自身の権限で実行する。
  - 管理者以外のユーザーが同ヘッダーを付与した場合、ヘッダーは無視される。
  - `POST /auth/token`（トークンリセット）は成り代わりの対象外で、常にリクエストユーザー本人のトークンをリセットする。
  - 最終アクセス時刻（アイドル判定）は成り代わり先ではなく管理者本人のものが更新される。
  - エラー:
    - `404 impersonate_user_not_found`: 指定されたユーザーが存在しない
- **ログアウト**: フロントエンド側で保持しているトークンを破棄することで完結（サーバー側エンドポイントは不要）。
- **ユーザー管理**:
  - `/auth/login` 実行時、Discord IDがDBになければ自動生成。
  - 既存ユーザーの場合は、Discord側の最新の表示名（global_name）でDBを更新。
- **トークン**:
  - 意味を持たない一意なランダム文字列（Opaque Token）。
  - フロントエンド側で `localStorage` 等に保存して利用。
- **アバター（アイコン）**:
  - `/auth/login` 時に Discord から `avatar` ハッシュを取得しDBに保存。
