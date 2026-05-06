# 盤面情報取得 API の実装

## エンドポイント

- GET /games/:game_uuid/state

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）

## 機能

- 現在の盤面情報（ユニット位置・占領情報・現フェイズ等）を返す。
- 参加者のみ取得可能。

## 成功レスポンス

- Status: `200 OK`
- 未決（`season`, `units`, `territories`, `orders` 等を含む構造体）

## エラー

- 401: 未認証
- 403: 参加者でない場合
- 404: 卓が存在しない場合

## その他

- 未決
