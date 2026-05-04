# PUT /admin/games/:game_uuid/units/:location

# DELETE /admin/games/:game_uuid/units/:location

## 概要

- 卓主が自分の卓のユニットを操作する API
- `PUT`: 指定地点のユニットを配置または置換
- `DELETE`: 指定地点のユニットを削除
- 既存ユニットと配置先が重複する場合は既存ユニットおよびその命令を削除してから新ユニットを配置する

## HTTP

- `PUT /admin/games/:game_uuid/units/:location`
- `DELETE /admin/games/:game_uuid/units/:location`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`（`PUT` のみ）

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）
- `location` (`string`): 地域コード（例: `par`, `lon`, `spa_nc`）

## PUT リクエストボディ

```json
{
  "unit": {
    "power": "f",
    "kind": "a"
  }
}
```

### `unit.power` (`string`)

- 担当国1文字コード: `a | e | f | g | i | r | t`

### `unit.kind` (`string`)

- `a` または `army`（陸軍）
- `f` または `fleet`（海軍）

## 成功レスポンス

- Status: `200 OK`

### PUT（配置・置換）

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "location": "par",
  "unit": {
    "power": "f",
    "kind": "A"
  }
}
```

### DELETE（削除）

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "location": "par",
  "unit": null
}
```

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | not_found | forbidden | repository_error",
  "message": "人間向け説明"
}
```

## 全エラーケース

### `400 Bad Request` — `code: invalid_request`

| 条件 | `message` |
|---|---|
| Authorization ヘッダ欠落 | `"authorization header is required"` |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"` |
| アクセストークン空（Bearer 後が空文字） | `"access token is required"` |
| `game_uuid` が有効な UUID でない | `"game_uuid is invalid"` |
| `location` が無効な地域コード | `"invalid location: <location>"` |
| `unit.power` が無効な国コード（PUT） | `"invalid power: <power>"` |
| `unit.kind` が無効な値（PUT） | `"invalid unit kind: <kind>"` |
| 陸軍を海域に配置しようとした（PUT） | `"army cannot be placed in a sea province"` |
| 陸軍を海岸バリアントコードで指定した（PUT） | `"army cannot be placed on a coast variant location"` |
| 海軍を内陸に配置しようとした（PUT） | `"fleet cannot be placed in an inland province"` |
| 海軍を双海岸地域のベースコードで指定した（PUT） | `"fleet must specify a coast variant for this location"` |

### `401 Unauthorized` — `code: unauthorized`

| 条件 | `message` |
|---|---|
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件 | `message` |
|---|---|
| リクエストユーザーが当該卓の卓主でない | `"user is not the owner of this game"` |
| 卓にフェイズが存在しない | `"game has no phases"` |
| 最新フェイズがメインフェイズ以外 | `"units can only be set during a main phase"` |

### `404 Not Found` — `code: not_found`

| 条件 | `message` |
|---|---|
| 指定した UUID の卓が存在しない | `"game not found"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件 | `message` |
|---|---|
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- メインフェイズ制限: 最新フェイズが `SpringMain` または `FallMain` のときのみ操作可能
- 既存ユニット置換: 同じベースコードに既存ユニットがある場合、既存ユニットと関連命令を削除してから新ユニットを配置
- Hold 命令自動生成: PUT で新ユニット配置時に Hold 命令を自動生成
- システムメッセージ: ユニット状態が変化した場合のみメッセージ DB に追記（同じ状態への再適用では追記しない）

| 操作 | 追記されるメッセージ |
|---|---|
| 新規配置（旧なし → 新あり） | `UnitPlaced` |
| 置換（旧あり → 新あり、内容が異なる） | `UnitReplaced` |
| 削除（旧あり → 新なし） | `UnitRemoved` |
| 変化なし（旧新が同一） | なし |

## curl

### 陸軍配置（PUT）

```bash
curl -X PUT '{base_path}/admin/games/{game_uuid}/units/par' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"unit":{"power":"f","kind":"a"}}'
```

### ユニット削除（DELETE）

```bash
curl -X DELETE '{base_path}/admin/games/{game_uuid}/units/par' \
  -H 'Authorization: Bearer <access_token>'
```
