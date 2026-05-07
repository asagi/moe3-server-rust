# POST /games/:game_uuid/players

## 概要

- 指定した卓にプレイヤーとして参加する API
- 既に同じ卓に同じ `requested_power` で参加済みの場合は冪等な成功（`200 OK`）を返す
- 7人揃った時点で担当国の割り当て・ステータス変更・卓番号採番が行われる

## HTTP

- Method: `POST`
- Path: `/games/:game_uuid/players`
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

なし（すべて任意）

### 任意パラメータ

- `requested_power` (`string | null`)
- `keyword` (`string | null`)

### 値定義

#### `requested_power`

- 未指定または `null` は希望なし
- 受理値:
  - 1文字コード: `"a" | "e" | "f" | "g" | "i" | "r" | "t"`
  - 国名文字列: `"Austria" | "England" | "France" | "Germany" | "Italy" | "Russia" | "Turkey"`
- 上記以外は `400 Bad Request`

#### `keyword`

- 未指定または `null` はキーワードなし
- 前後の空白はトリムされる
- トリム後に空文字となる場合はキーワードなしとして扱われる
- 英数字（ASCII）のみ許容。それ以外は `400 Bad Request`
- 卓のキーワードと一致しない場合は `403 Forbidden`
- 卓にキーワードがない場合にリクエストにキーワードを指定しても `403 Forbidden`

## リクエスト例

### 最小

```json
{}
```

### requested_power 指定（1文字コード）

```json
{
  "requested_power": "f"
}
```

### requested_power 指定（国名文字列）

```json
{
  "requested_power": "France"
}
```

### keyword 指定（鍵卓）

```json
{
  "requested_power": null,
  "keyword": "secret123"
}
```

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "参加した卓の UUID",
  "user_uuid": "参加したユーザーの UUID",
  "requested_power": "France"
}
```

注意:

- `requested_power` はレスポンスで国名文字列を返す（例: `"France"`）
- 希望なしは `null`

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

| 条件                                                | `message`                                          |
| --------------------------------------------------- | -------------------------------------------------- |
| Authorization ヘッダ欠落                            | `"authorization header is required"`               |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"` |
| アクセストークン空（Bearer 後が空文字）             | `"access token is required"`                       |

#### パスパラメータ検証

| 条件                             | `message`                |
| -------------------------------- | ------------------------ |
| `game_uuid` が有効な UUID でない | `"game_uuid is invalid"` |

#### リクエストボディ検証

| 条件                                          | `message`                                              |
| --------------------------------------------- | ------------------------------------------------------ |
| `requested_power` が不正な値                  | `"requested_power is invalid"`                         |
| `keyword` に英数字以外の文字が含まれる        | `"keyword must contain only alphanumeric characters"`  |
| 既に同じ卓に別の `requested_power` で参加済み | `"user already joined with different requested_power"` |

### `401 Unauthorized` — `code: unauthorized`

| 条件                                                        | `message`        |
| ----------------------------------------------------------- | ---------------- |
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件                                                                               | `message`                                                |
| ---------------------------------------------------------------------------------- | -------------------------------------------------------- |
| リクエストユーザーが別の卓に参加中                                                 | `"user is already participating in another active game"` |
| キーワードが卓のキーワードと一致しない（キーワードなし卓へのキーワード指定を含む） | `"keyword does not match"`                               |
| 卓のステータスが `Preparing` でない（募集終了済み）                                | `"game is not accepting new players"`                    |
| 卓の定員（7人）に達している                                                        | `"game is full (max players reached)"`                   |

### `404 Not Found` — `code: not_found`

| 条件                           | `message`          |
| ------------------------------ | ------------------ |
| 指定した UUID の卓が存在しない | `"game not found"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件             | `message`                 |
| ---------------- | ------------------------- |
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- 参加条件: 卓のステータスが `Preparing`（募集中）であること
- 定員: 最大 7 人（担当国数）
- 冪等性: 同じ `requested_power` で同じ卓に再リクエストした場合は `200 OK` を返す（DB 更新なし）
- 7人成立時の処理: 7人目が参加した時点で担当国をランダム割り当て・ステータスを `Ready` に変更・卓番号を採番する
- システムメッセージ: 参加成立時（新規参加）にメッセージ DB へ `PlayerJoined` メッセージを追記する。7人成立時は追加で `Ready` メッセージも追記する

## curl

### 最小

```bash
curl -X POST '{base_path}/games/{game_uuid}/players' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{}'
```

### requested_power 指定

```bash
curl -X POST '{base_path}/games/{game_uuid}/players' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"requested_power":"France"}'
```

### 鍵卓

```bash
curl -X POST '{base_path}/games/{game_uuid}/players' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"requested_power":null,"keyword":"secret123"}'
```
