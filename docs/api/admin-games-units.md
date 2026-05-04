# PUT /admin/games/:game_uuid/units

## 概要

- 卓主が自分の卓のユニットを配置・置換・削除する API
- `unit` を指定するとユニットを配置し、省略または `null` にすると `location` のユニットを削除する
- 既存ユニットと配置先が重複する場合は既存ユニットおよびその命令を削除してから新ユニットを配置する

## HTTP

- Method: `PUT`
- Path: `/admin/games/:game_uuid/units`
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

- `location` (`string`)

### 任意パラメータ

- `unit` (`object | null`)

### 値定義

#### `location`

- 地域コード文字列（例: `"par"`, `"lon"`, `"spa_nc"`）
- 存在しないコードは `400 Bad Request`
- 双海岸地域（`spa` など）に海軍を配置する場合は海岸バリアントコード（`"spa_nc"` / `"spa_sc"`）を指定すること

#### `unit`

- `null` または省略: 指定 `location` のユニットを削除する
- 存在しない場合: 削除操作

##### `unit.power` (`string`)

- 担当国を示す1文字コード: `"a" | "e" | "f" | "g" | "i" | "r" | "t"`
- 上記以外は `400 Bad Request`

##### `unit.kind` (`string`)

- ユニット種別:
  - `"a"` または `"army"` → 陸軍
  - `"f"` または `"fleet"` → 海軍
- 上記以外は `400 Bad Request`

## リクエスト例

### 陸軍配置

```json
{
  "unit": { "power": "f", "kind": "a" },
  "location": "par"
}
```

### 海軍配置（海岸バリアント指定）

```json
{
  "unit": { "power": "f", "kind": "f" },
  "location": "bre"
}
```

### 双海岸地域への海軍配置

```json
{
  "unit": { "power": "f", "kind": "f" },
  "location": "spa_nc"
}
```

### ユニット削除

```json
{
  "unit": null,
  "location": "par"
}
```

## 成功レスポンス

- Status: `200 OK`

### ユニット配置・置換時

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

### ユニット削除時

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "location": "par",
  "unit": null
}
```

注意:

- `unit.power` はレスポンスで1文字コードを返す（例: `"f"`）
- `unit.kind` はレスポンスでシンボル文字を返す（陸軍: `"A"`, 海軍: `"F"`）

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

#### リクエストボディ検証

| 条件 | `message` |
|---|---|
| `location` が無効な地域コード | `"invalid location: <location>"` |
| `unit.power` が無効な国コード | `"invalid power: <power>"` |
| `unit.kind` が無効な値 | `"invalid unit kind: <kind>"` |
| 陸軍を海域に配置しようとした | `"army cannot be placed in a sea province"` |
| 陸軍を海岸バリアントコードで指定した | `"army cannot be placed on a coast variant location"` |
| 海軍を内陸に配置しようとした | `"fleet cannot be placed in an inland province"` |
| 海軍を双海岸地域のベースコードで指定した（バリアント指定なし） | `"fleet must specify a coast variant for this location"` |

### `401 Unauthorized` — `code: unauthorized`

| 条件 | `message` |
|---|---|
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件 | `message` |
|---|---|
| リクエストユーザーが当該卓の卓主でない | `"user is not the owner of this game"` |
| 卓にフェイズが存在しない | `"game has no phases"` |
| 現在の最新フェイズがメインフェイズ（春命令・秋命令）以外 | `"units can only be set during a main phase"` |

### `404 Not Found` — `code: not_found`

| 条件 | `message` |
|---|---|
| 指定した UUID の卓が存在しない | `"game not found"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件 | `message` |
|---|---|
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- メインフェイズ制限: 最新フェイズが `SpringMain`（春命令）または `FallMain`（秋命令）のときのみ操作可能
- 既存ユニット置換: 同じベースコードにユニットが既に存在する場合、既存ユニットとその関連命令（直接命令・支援対象・輸送対象を含む）をすべて削除してから新ユニットを配置する
- Hold 命令自動生成: 新ユニット配置時、そのユニットの Hold 命令を自動で生成する
- システムメッセージ: ユニットの状態が変化した場合にメッセージ DB へ追記する

| 操作 | 追記されるメッセージ |
|---|---|
| 新規配置（旧ユニットなし → 新ユニットあり） | `UnitPlaced` |
| 置換（旧ユニットあり → 新ユニットあり） | `UnitReplaced` |
| 削除（旧ユニットあり → 新ユニットなし） | `UnitRemoved` |
| 変化なし（旧ユニットなし → 新ユニットなし） | なし |

- 排他制御: サーバー全体の `game_update_lock` を取得した上で実行される

## curl

### 陸軍配置

```bash
curl -X PUT '{base_path}/admin/games/{game_uuid}/units' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"unit":{"power":"f","kind":"a"},"location":"par"}'
```

### ユニット削除

```bash
curl -X PUT '{base_path}/admin/games/{game_uuid}/units' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"unit":null,"location":"par"}'
```
