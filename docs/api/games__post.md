# POST /games

## 概要

- 新しい卓（募集）を作成する API
- リクエスト送信者は卓主権限を持つプレイヤーになる
- 新卓の初期進行タイプは常に定時更新になる（卓主権限で合意進行に変更可能）

## HTTP

- Method: `POST`
- Path: `/games`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`

## リクエストボディ

### 型

- `object`

### 必須パラメータ

- `face_type` (`integer`)
- `duration_type` (`integer`)
- `start_date` (`string`)
- `first_period_hour` (`integer`)

### 任意パラメータ

- `requested_power` (`string | null`)
- `keyword` (`string | null`)

### 値定義

#### `face_type`

- `1` = `FaceType::Girls`
- `2` = `FaceType::Flags`

#### `duration_type`

- `1` = `DurationType::Short`
- `2` = `DurationType::Normal`

#### `start_date`

- 形式: `"YYYY-MM-DD"`
- 意味: 開始日（JST）

#### `first_period_hour`

- 範囲: `0`〜`23`
- 意味: 開始時刻（JST）

#### `requested_power`

- 未指定または `null` は希望なし
- 受理値:
  - 1文字コード: `"a" | "e" | "f" | "g" | "i" | "r" | "t"`
  - 国名文字列: `"Austria" | "England" | "France" | "Germany" | "Italy" | "Russia" | "Turkey"`
- 上記以外は `400 Bad Request`

#### `keyword`

- 未指定または `null` はキーワードなし（鍵卓にならない）
- 前後の空白はトリムされる
- トリム後に空文字となる場合はキーワードなしとして扱われる
- 英数字（ASCII）のみ許容。それ以外は `400 Bad Request`

## リクエスト例

### 最小

```json
{
  "face_type": 1,
  "duration_type": 1,
  "start_date": "2026-04-19",
  "first_period_hour": 12,
  "requested_power": null
}
```

### requested_power 指定（1文字コード）

```json
{
  "face_type": 1,
  "duration_type": 1,
  "start_date": "2026-04-19",
  "first_period_hour": 12,
  "requested_power": "f"
}
```

### requested_power 指定（国名文字列）

```json
{
  "face_type": 1,
  "duration_type": 1,
  "start_date": "2026-04-19",
  "first_period_hour": 12,
  "requested_power": "France"
}
```

### keyword 指定（鍵卓）

```json
{
  "face_type": 1,
  "duration_type": 1,
  "start_date": "2026-04-19",
  "first_period_hour": 12,
  "requested_power": null,
  "keyword": "secret123"
}
```

## 成功レスポンス

- Status: `201 Created`

```json
{
  "game_uuid": "作成されたゲームの UUID",
  "owner_user_uuid": "オーナーのユーザー UUID",
  "requested_power": "France"
}
```

注意:

- `requested_power` はレスポンスで国名文字列を返す（例: `"France"`）
- 希望なしは `null`

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | repository_error",
  "message": "人間向け説明"
}
```

## 全エラーケース

### `400 Bad Request` — `code: invalid_request`

#### `Authorization` ヘッダ関連

| 条件                                                | message                                            |
| --------------------------------------------------- | -------------------------------------------------- |
| Authorization ヘッダ欠落                            | `"authorization header is required"`               |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"` |
| アクセストークン空（Bearer 後が空文字）             | `"access token is required"`                       |

#### リクエストボディ検証 — 列挙値

| 条件                     | message                      |
| ------------------------ | ---------------------------- |
| face_type が無効な値     | `"face_type is invalid"`     |
| duration_type が無効な値 | `"duration_type is invalid"` |

#### リクエストボディ検証 — 日時

| 条件                                             | message                                         |
| ------------------------------------------------ | ----------------------------------------------- |
| start_date フォーマット不正（"YYYY-MM-DD" 以外） | `"start_date is invalid (expected YYYY-MM-DD)"` |
| first_period_hour 範囲外（0〜23 以外）           | `"first_period_hour must be between 0 and 23"`  |

#### リクエストボディ検証 — 国コード

| 条件                               | message                        |
| ---------------------------------- | ------------------------------ |
| requested_power 不正（許容値以外） | `"requested_power is invalid"` |

#### リクエストボディ検証 — keyword

| 条件                                 | message                                               |
| ------------------------------------ | ----------------------------------------------------- |
| keyword に英数字以外の文字が含まれる | `"keyword must contain only alphanumeric characters"` |

### `401 Unauthorized` — `code: unauthorized`

| 条件                                                        | message          |
| ----------------------------------------------------------- | ---------------- |
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件                                               | message                                              |
| -------------------------------------------------- | ---------------------------------------------------- |
| データベース障害（ユーザー検索またはゲーム作成時） | `"repository error: repository unavailable: <詳細>"` |

## バリデーション詳細

- `start_date` は `"YYYY-MM-DD"` 形式
- `first_period_hour` は `0`〜`23`
- `start_date + first_period_hour`（JST）が現在時刻より30分後以降であること

## curl

### 希望なし

```sh
curl -X POST '{base_path}/games' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"face_type":1,"duration_type":1,"start_date":"2026-04-19","first_period_hour":12,"requested_power":null}'
```

### 希望あり

```sh
curl -X POST '{base_path}/games' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"face_type":1,"duration_type":1,"start_date":"2026-04-19","first_period_hour":12,"requested_power":"f"}'
```

### keyword 指定（鍵卓）

```sh
curl -X POST '{base_path}/games' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"face_type":1,"duration_type":1,"start_date":"2026-04-19","first_period_hour":12,"requested_power":null,"keyword":"secret123"}'
```
