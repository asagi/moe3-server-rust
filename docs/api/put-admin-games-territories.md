# PUT /admin/games/:game_uuid/territories/:code

## 概要

- 卓主が自分の卓の指定プロヴィンスの保有国を設定または変更する API
- 既に保有国が設定されている場合は新しい保有国に上書きする

## HTTP

- Method: `PUT`
- Path: `/admin/games/:game_uuid/territories/:code`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）
- `code` (`string`): プロヴィンスコード（例: `par`, `lon`, `bud`）

## リクエストボディ

```json
{
  "power": "f"
}
```

### `power` (`string`)

- 担当国1文字コード: `a | e | f | g | i | r | t`

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "code": "par",
  "power": "f"
}
```

## エラー形式

```json
{
  "code": "invalid_request | unauthorized | not_found | forbidden | repository_error",
  "message": "人間向け説明"
}
```

## エラーケース

### `400 Bad Request` — `code: invalid_request`

| 条件 | `message` |
|---|---|
| Authorization ヘッダ欠落 | `"authorization header is required"` |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"` |
| アクセストークン空（Bearer 後が空文字） | `"access token is required"` |
| `game_uuid` が有効な UUID でない | `"game_uuid is invalid"` |
| `code` が無効なプロヴィンスコード | `"invalid code: <code>"` |
| `power` が無効な国コード | `"invalid power: <power>"` |

### `401 Unauthorized` — `code: unauthorized`

| 条件 | `message` |
|---|---|
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件 | `message` |
|---|---|
| リクエストユーザーが当該卓の卓主でない | `"user is not the owner of this game"` |
| 卓にフェイズが存在しない | `"game has no phases"` |
| 最新フェイズがメインフェイズ以外 | `"territories can only be set during a main phase"` |

### `404 Not Found` — `code: not_found`

| 条件 | `message` |
|---|---|
| 指定した UUID の卓が存在しない | `"game not found"` |
| 指定したコードが海洋プロヴィンス | `"territory not found"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件 | `message` |
|---|---|
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- メインフェイズ制限: 最新フェイズが `SpringMain` または `FallMain` のときのみ操作可能
- 海洋プロヴィンス不可: 海洋プロヴィンスへの保有国設定は 404 を返す
- 既存保有国置換: 同じプロヴィンスに既存の保有国がある場合は上書きする
- システムメッセージ: 保有国の状態が変化した場合のみメッセージ DB に追記（同じ状態への再適用では追記しない）
