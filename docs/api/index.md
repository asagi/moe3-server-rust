# API 一覧

## 認証・ユーザー管理

| メソッド | パス | 概要 | 備考 |
| --- | --- | --- | --- |
| `POST` | [`/auth/login`](./post-auth-login.md) | ログイン（自動ユーザー生成） | Discord 情報で Upsert し不透明トークンを発行 |
| `PUT` | `/auth/token` | トークンリセット | 既存のトークンを無効化し、新たに払い出す |
| `GET` | `/users/me` | 自身のユーザー情報取得 | 表示名、参加卓一覧など |

## 募集と応募

| メソッド | パス | 概要 | 備考 |
| --- | --- | --- | --- |
| `GET` | `/games` | ゲーム一覧取得 | 募集中・進行中などの一覧 |
| `POST` | [`/games`](./post-games.md) | 新規卓作成 | ルール設定や開始時間の定義 |
| `GET` | `/games/{id}` | 卓情報取得 | 参加者、現フェイズ、設定など |
| `GET` | `/games/{id}/logs/{turn}` | 外交履歴取得 |  |
| `GET` | `/games/{id}/result/{turn}` | 命令解決履歴取得 |  |
| `POST` | [`/games/{id}/players`](./post-games-players.md) | ゲームへの参加 | |

## ゲーム進行・命令

| メソッド | パス | 概要 | 備考 |
| --- | --- | --- | --- |
| `GET` | `/games/{id}/state` | 盤面情報の取得 | ユニット配置、補給都市、残り時間 |
| `GET` | `/games/{id}/orders` | 自身の提出済み命令確認 | 自分が今期出している命令の再認（他国軍に対する仮定命令も含む） |
| `GET` | `/games/{id}/orders/{id}` | 命令詳細取得 | フェイズによって指定命令の対象ユニットが移動撤退可能な地域、支援や輸送が可能な対象、解体可能なユニット、建造可能な補給都市のリストが返却される |
| `PUT` | `/games/{id}/orders/{id}` | 命令変更 | GET で取得したリストに適合する命令に変更可能（建造命令は別 API を使用する） |
| `POST` | `/games/{id}/orders` | 建造命令登録 | 調整フェイズ建造命令専用 API |
| `DELETE` | `/games/{id}/orders/{id}` | 建造命令削除 | 調整フェイズ建造命令専用 API |

## 卓主権限

| メソッド | パス | 概要 | 備考 |
| --- | --- | --- | --- |
| `PUT` | [`/admin/games/{id}/units`](./admin-games-units.md) | ユニットの登録、変更、削除 | メインフェイズのみ |
| `PUT` | `/admin/games/{id}/territories` | 占領情報の登録、変更、削除 | メインフェイズのみ |
| `PUT` | `/admin/games/{id}/progress-mode` | 進行モードを合意進行に切り替える | メインフェイズのみ |
| `PUT` | `/admin/games/{id}/next-update-at` | 次回更新時刻を変更する | メインフェイズのみ |
| `PUT` | [`/admin/games/{id}/draw-proposal`](./admin-games-draw-proposal.md) | 和平終了フラグを変更する | メインフェイズのみ |

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
