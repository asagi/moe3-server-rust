# 卓情報取得 API の実装

## エンドポイント

- GET /games/:game_uuid

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）

## 機能

- 指定した卓の詳細情報を返す。
- 未決（認証不要かどうか）

## 成功レスポンス

- Status: `200 OK`
- 未決（`game_uuid`, `status`, `regulation`, `players`, `next_update_at` 等。フェーズ情報の範囲も未決）

## エラー

- 404: 指定した卓が存在しない場合

## その他

- 未決
