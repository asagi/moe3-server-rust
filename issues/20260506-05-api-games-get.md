# 卓情報取得 API の実装

## エンドポイント

- GET /games/:game_uuid

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）

## 機能

- 指定した卓の最新の詳細情報を返す。
- 認証不要。

## 成功レスポンス

- Status: `200 OK`
- `game`
  - `game_uuid`
  - `game_number`
  - `status`: `"preparing" | "ready" | "in_progress" | "solo" | "draw" | "aborted" | "closed"`
  - `seasons`: `"ready"` から最新までのすべてのシーズンの配列（例: `["ready", "1901s", "1901f"]`）
  - `phase_kind`: "ready" | "main" | "retreat" | "adjustment" | "debrief"
  - `next_update_at`: yyyy-MM-dd HH:mm 形式（JST）に変換して返却。
  - `is_private`: ブール値。キーワードが設定されてる場合は `true`

## エラー

- 404: 指定した uuid の Game が存在しない場合

## その他

- `status` が `"preparing"` または `"ready"` の場合、`"season"` は `"ready"` とする。
- `status` が `"solo"` または `"draw"` の場合、`"season"` は `"debrief"` とする。
- 例： `["ready", "1901s", "1901f", ..., "1907f", "debrief"]`
- `status` が `"closed"` または `"aborted"` の場合、`season` は追加不要。
  - closed と aborted は卓が閉じた状態を示すだけでシーズンとして固有の情報を持たないため。
- 既存処理を修正する際には DB の後方互換は考慮しなくて良い。
