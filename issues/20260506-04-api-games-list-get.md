# 卓一覧取得 API の実装

## エンドポイント

- GET /games

## クエリパラメータ

- 未決（ステータスによるフィルタ等）

## 機能

- アクティブな卓（Closed・Aborted 以外）の一覧を返す。
- 未決（認証不要かどうか）

## 成功レスポンス

- Status: `200 OK`
- 未決（各卓の概要情報の配列。`game_uuid`, `status`, `season`, `next_update_at` 等）

## その他

- 未決
