# PUT /games/:game_uuid/progress-consensus

## 概要

- プレイヤーが自分の勢力について、即時進行への合意を表明・撤回する API
- 合意状態が変化した場合のみ DB を更新し、システムメッセージを追記する
- 合意状態が変化しない場合は冪等として `200 OK` を返す（DB 更新・メッセージ追記なし）
- 進行モードが `consensus` の卓でのみ利用可能

## HTTP

- Method: `PUT`
- Path: `/games/:game_uuid/progress-consensus`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`

## パスパラメータ

- `game_uuid` (`string`): 対象卓の UUID（UUID v7 形式）

## リクエストボディ

### 型

- `object`

### 必須パラメータ

- `agreed` (`boolean`)

### 値定義

- `true`: 即時進行に合意する
- `false`: 即時進行への合意を撤回する

## リクエスト例

### 合意する

```json
{
  "agreed": true
}
```

### 撤回する

```json
{
  "agreed": false
}
```

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "agreed": true
}
```

注意:

- `agreed` はリクエストで指定した値を返す
- 既に同じ値だった場合も `200 OK` を返す（冪等）

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

### `401 Unauthorized` — `code: unauthorized`

| 条件                                                        | `message`        |
| ----------------------------------------------------------- | ---------------- |
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件                                         | `message`                                                                |
| -------------------------------------------- | ------------------------------------------------------------------------ |
| 進行モードが `consensus` ではない            | `"progress consensus is available only when progress mode is consensus"` |
| リクエストユーザーが当該卓のプレイヤーでない | `"user is not a player of this game"`                                    |
| プレイヤーの勢力が未割当                     | `"player has no assigned power yet"`                                     |
| 現在フェーズで操作対象勢力でない             | `"player is not an active power in the current phase"`                   |
| 卓にフェイズが存在しない                     | `"game has no phases"`                                                   |

### `404 Not Found` — `code: not_found`

| 条件                           | `message`          |
| ------------------------------ | ------------------ |
| 指定した UUID の卓が存在しない | `"game not found"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件             | `message`                 |
| ---------------- | ------------------------- |
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- この API はフェーズごとに「操作が必要な勢力」全員が `agreed=true` になると即時進行を成立させる
- 操作が必要な勢力の判定は以下の通り

| フェーズ                                   | 対象勢力                               |
| ------------------------------------------ | -------------------------------------- |
| Main（`SpringMain` / `FallMain`）          | 現在の領有勢力                         |
| Retreat（`SpringRetreat` / `FallRetreat`） | 撃退ユニットを持つ勢力                 |
| Adjustment                                 | 補給都市数とユニット数が一致しない勢力 |
| Ready / Debrief                            | 対象なし                               |

- 合意状態が変化した場合のみ、当該卓のメッセージ DB に追記する

| 操作                   | カタログ値                     | メッセージ本文                                     |
| ---------------------- | ------------------------------ | -------------------------------------------------- |
| 合意（`agreed=true`）  | `progress_consented`           | {勢力名}によって即時進行に合意しました。           |
| 撤回（`agreed=false`） | `progress_consensus_rescinded` | {勢力名}によって即時進行への合意が撤回されました。 |

- 即時進行が成立し、かつメインフェーズの場合のみシステムメッセージを追記する

| 条件                   | カタログ値                   | メッセージ本文                         |
| ---------------------- | ---------------------------- | -------------------------------------- |
| 操作対象勢力が全員合意 | `progress_consensus_reached` | 全員の合意により、即時更新されました。 |

- 即時進行成立時は `next_update_at` を現在時刻に更新する
- 即時進行成立時は、無政府国以外のプレイヤーの `progress_consented` を `false` に戻す
- この API はリクエストハンドラ内で `game_update_lock` を取得して実行される

## curl

### 合意する

```bash
curl -X PUT '{base_path}/games/{game_uuid}/progress-consensus' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"agreed":true}'
```

### 撤回する

```bash
curl -X PUT '{base_path}/games/{game_uuid}/progress-consensus' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"agreed":false}'
```
