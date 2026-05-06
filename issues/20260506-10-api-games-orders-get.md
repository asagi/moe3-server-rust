# 命令詳細取得 API の実装

## エンドポイント

- GET /games/:game_uuid/orders/:order_id

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）
- `order_id` (`integer` or `string`): 命令 ID（未決：型・形式）

## 機能

- 指定した命令の詳細情報を返す。
- 参加者のみ取得可能。他国軍に対する仮定命令も取得可能。

## 成功レスポンス

- Status: `200 OK`
- 未決（`order_id`, `unit`, `order_kind`, `status`, `target` 等）

## エラー

- 401: 未認証
- 403: 参加者でない場合
- 404: 卓または命令が存在しない場合

## その他

- 未決
