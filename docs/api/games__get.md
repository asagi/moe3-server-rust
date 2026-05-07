# GET /games

## 概要

- 卓の一覧を取得する API
- ステータス指定検索・ユーザー指定検索の 2 種類を提供する
- ステータス指定検索は認証不要、ユーザー指定検索は認証必須

## HTTP

- Method: `GET`
- Path: `/games`

## ヘッダ

- Authorization: ユーザー指定検索時のみ必須（`Bearer <access_token>`）

## クエリパラメータ

| パラメータ | 型       | 必須 | デフォルト | 説明                                   |
| ---------- | -------- | ---- | ---------- | -------------------------------------- |
| `status`   | `string` | 任意 | —          | ステータスフィルタ（`user` と排他）    |
| `user`     | `string` | 任意 | —          | Discord ユーザー ID（`status` と排他） |
| `page`     | `number` | 任意 | `1`        | ページ番号（1 以上）                   |
| `per_page` | `number` | 任意 | `20`       | 1 ページあたりの件数（1〜100）         |

### `status` の受理値

| 値        | 意味                                     |
| --------- | ---------------------------------------- |
| `active`  | Aborted・Closed 以外のすべてのステータス |
| `closed`  | 終了済み                                 |
| `aborted` | 中止済み                                 |

### 検索モードの優先順位

- `status` と `user` は同時指定不可（`400 Bad Request`）
- どちらも未指定の場合は `status=active` として扱う

## リクエスト例

### アクティブな卓一覧（デフォルト）

```
GET /games
```

### ステータス指定

```
GET /games?status=closed&page=1&per_page=20
```

### ユーザー指定（認証必須）

```
GET /games?user=123456789012345678
Authorization: Bearer <access_token>
```

## 成功レスポンス

- Status: `200 OK`

```json
{
  "games": [
    {
      "game_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "game_number": 42,
      "status": "recruiting",
      "season": "1901春",
      "next_update_at": "2026-05-10 20:00",
      "regulation": {
        "face_type": "girls",
        "progress_mode": "scheduled",
        "duration_type": "normal"
      },
      "player_count": 3
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 20
}
```

### フィールド詳細

#### ゲームオブジェクト

| フィールド       | 型               | 説明                                              |
| ---------------- | ---------------- | ------------------------------------------------- |
| `game_uuid`      | `string`         | 卓の UUID                                         |
| `game_number`    | `number \| null` | 卓番号（採番前は `null`）                         |
| `status`         | `string`         | 卓のステータス（下表参照）                        |
| `season`         | `string \| null` | 現在のシーズン（例: `"1901春"`）。開始前は `null` |
| `next_update_at` | `string \| null` | 次回更新日時（JST、`"YYYY-MM-DD HH:mm"` 形式）    |
| `regulation`     | `object`         | レギュレーション情報                              |
| `player_count`   | `number`         | 現在の参加人数                                    |

#### `status` の値

| 値            | 意味         |
| ------------- | ------------ |
| `recruiting`  | 募集中       |
| `in_progress` | 進行中       |
| `finished`    | 終了         |
| `closed`      | クローズ済み |
| `aborted`     | 中止済み     |

#### `regulation` フィールド

| フィールド      | 値                            | 意味           |
| --------------- | ----------------------------- | -------------- |
| `face_type`     | `"girls"` / `"flags"`         | 駒の種類       |
| `progress_mode` | `"scheduled"` / `"consensus"` | 進行モード     |
| `duration_type` | `"short"` / `"normal"`        | 対局期間の種類 |

#### ページネーション

| フィールド | 型       | 説明                 |
| ---------- | -------- | -------------------- |
| `total`    | `number` | 条件に合致する総件数 |
| `page`     | `number` | 現在のページ番号     |
| `per_page` | `number` | 1 ページあたりの件数 |

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | forbidden | repository_error",
  "message": "人間向け説明"
}
```

## 全エラーケース

### `400 Bad Request` — `code: invalid_request`

#### クエリパラメータ

| 条件                              | message                                                  |
| --------------------------------- | -------------------------------------------------------- |
| `status` と `user` を同時指定     | `"status and user cannot be specified at the same time"` |
| `status` に無効な値を指定         | `"status must be 'active', 'closed', or 'aborted'"`      |
| `page` が 1 未満                  | `"page must be 1 or greater"`                            |
| `per_page` が 1 未満または 100 超 | `"per_page must be between 1 and 100"`                   |

#### `Authorization` ヘッダ関連（`user` 指定時のみ検証）

| 条件                                                | message                                            |
| --------------------------------------------------- | -------------------------------------------------- |
| Authorization ヘッダ欠落                            | `"authorization header is required"`               |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"` |
| アクセストークン空（Bearer 後が空文字）             | `"access token is required"`                       |

### `401 Unauthorized` — `code: unauthorized`

| 条件                                                        | message          |
| ----------------------------------------------------------- | ---------------- |
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件                                            | message               |
| ----------------------------------------------- | --------------------- |
| 他ユーザーの `discord_user_id` を `user` に指定 | `"forbidden: <詳細>"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件             | message                                              |
| ---------------- | ---------------------------------------------------- |
| データベース障害 | `"repository error: repository unavailable: <詳細>"` |

## curl

### アクティブな卓一覧

```sh
curl '{base_path}/games'
```

### ステータス指定（Closed）

```sh
curl '{base_path}/games?status=closed&page=1&per_page=20'
```

### ユーザー指定

```sh
curl '{base_path}/games?user=123456789012345678' \
  -H 'Authorization: Bearer <access_token>'
```
