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
  - `status`
  - `season`： 季節（例: `"ready"`, `"1901s"`, `"1901f"`, ..., `"debrief"`）
  - `phase_kind`： "ready" | "main" | "retreat" | "adjustment" | "debrief"
  - `next_update_at`： yyyy-MM-dd HH:mm 形式（JST）に変換して返却。
  - `is_private`： ブール値。キーワードが設定されてる場合は `true`

## エラー

- 404: 指定した uuid の Game が存在しない場合

## その他

- 未決
