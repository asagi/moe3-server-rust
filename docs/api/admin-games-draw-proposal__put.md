# PUT /admin/games/:game_uuid/draw-proposal

## 概要

- 卓主が自分の卓に対して講和終了フラグを宣言・撤回する API
- フラグが変化した場合のみ DB を更新し、システムメッセージを追記する
- フラグ値が変化しない場合はべき等として `200 OK` を返す（DB 更新・メッセージ追記なし）

## HTTP

- Method: `PUT`
- Path: `/admin/games/:game_uuid/draw-proposal`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）

## リクエストボディ

### 型

- `object`

### 必須パラメータ

- `draw_proposal` (`boolean`)

### 値定義

#### `draw_proposal`

- `true` = 講和宣言
- `false` = 講和撤回

## リクエスト例

### 講和宣言

```json
{
  "draw_proposal": true
}
```

### 講和撤回

```json
{
  "draw_proposal": false
}
```

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "draw_proposal": true
}
```

注意:

- `draw_proposal` はリクエストで指定した値をそのまま返す
- 既にフラグが同じ値の場合も `200 OK` を返す（べき等）

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | not_found | forbidden | repository_error",
  "message": "人間向け説明"
}
```

## 全エラーケース

### `400 Bad Request` — `code: invalid_request`

#### `Authorization` ヘッダ関連

| 条件 | `message` |
|---|---|
| Authorization ヘッダ欠落 | `"authorization header is required"` |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"` |
| アクセストークン空（Bearer 後が空文字） | `"access token is required"` |

#### パスパラメータ検証

| 条件 | `message` |
|---|---|
| `game_uuid` が有効な UUID でない | `"game_uuid is invalid"` |

### `401 Unauthorized` — `code: unauthorized`

| 条件 | `message` |
|---|---|
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件 | `message` |
|---|---|
| リクエストユーザーが当該卓の卓主でない | `"user is not the owner of this game"` |
| 現在の最新フェイズがメインフェイズ（春命令・秋命令）以外 | `"draw proposal can only be set during a main phase"` |

### `404 Not Found` — `code: not_found`

| 条件 | `message` |
|---|---|
| 指定した UUID の卓が存在しない | `"game not found"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件 | `message` |
|---|---|
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- メインフェイズ制限: 最新フェイズが `SpringMain`（春命令）または `FallMain`（秋命令）のときのみ操作可能。`Ready`・各退却フェイズ・調整フェイズ中は `403 Forbidden`
- システムメッセージ: フラグ値が変化した場合のみ、当該卓のメッセージ DB に追記する

| 操作 | カタログ値 | メッセージ本文 |
|---|---|---|
| 宣言（`true`） | `draw_proposed` | 卓主によって講和が宣言されました。 |
| 撤回（`false`） | `draw_rescinded` | 卓主によって講和が撤回されました。 |

- 排他制御: サーバー全体の `game_update_lock` を取得した上で実行される

## curl

### 講和宣言

```
curl -X PUT '{base_path}/admin/games/{game_uuid}/draw-proposal' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"draw_proposal":true}'
```

### 講和撤回

```
curl -X PUT '{base_path}/admin/games/{game_uuid}/draw-proposal' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"draw_proposal":false}'
```
