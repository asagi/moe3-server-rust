# GET /games/:game_uuid

## 概要

- 指定した卓の情報を取得する API
- 認証不要（`Authorization` を付与すると `is_owner` / `is_joined` / 本人の `requested_power` にリクエスト者の情報が反映される）

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid`

## ヘッダ

- Authorization: 任意（`Bearer <access_token>`）
  - `is_owner` / `is_joined` の判定と `requested_power` の公開範囲の判定にのみ使用する。未登録トークン・形式不正の場合はエラーにせず未ログイン扱い

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game": {
    "game_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "game_number": 42,
    "status": "in_progress",
    "seasons": ["ready", "1901s", "1901f", "1902s"],
    "phase_kind": "main",
    "next_update_at": "2026-05-10 20:00",
    "is_private": false,
    "is_owner": false,
    "regulation": {
      "face_type": "girls",
      "progress_mode": "scheduled",
      "duration_type": "normal"
    },
    "start_at": "2026-05-01 20:00",
    "players": [
      {
        "user": {
          "discord_user_id": "123456789012345678",
          "username": "alice",
          "global_name": "Alice",
          "avatar_url": "https://cdn.discordapp.com/avatars/..."
        },
        "is_owner": true,
        "requested_power": "f",
        "power": "f"
      }
    ],
    "is_joined": false
  }
}
```

### フィールド詳細

| フィールド       | 型               | 説明                                                            |
| ---------------- | ---------------- | --------------------------------------------------------------- |
| `game_uuid`      | `string`         | 卓の UUID                                                       |
| `game_number`    | `number \| null` | 卓番号（採番前は `null`）                                       |
| `status`         | `string`         | 卓のステータス（下表参照）                                      |
| `seasons`        | `string[]`       | フェイズ履歴から生成されたシーズン配列（下表参照）              |
| `phase_kind`     | `string`         | 最新フェイズの種別（下表参照）                                  |
| `next_update_at` | `string \| null` | 次回更新日時（JST、`"YYYY-MM-DD HH:mm"` 形式）。未設定は `null` |
| `is_private`     | `boolean`        | キーワード設定済みの鍵卓かどうか                                |
| `is_owner`       | `boolean`        | リクエスト者がその卓の卓主か（未認証時は `false`）              |
| `regulation`     | `object`         | 卓のレギュレーション（下表参照）                                |
| `start_at`       | `string`         | 開始日時（JST、`"YYYY-MM-DD HH:mm"` 形式。分は常に `00`）       |
| `players`        | `object[]`       | 卓主を含む全参加者（参加申請の順）。下表参照                    |
| `is_joined`      | `boolean`        | リクエスト者がその卓に参加（申請）済みか（未認証時は `false`）  |

#### `regulation` の値

| フィールド      | 型       | 値                             |
| --------------- | -------- | ------------------------------ |
| `face_type`     | `string` | `"girls"` \| `"flags"`         |
| `progress_mode` | `string` | `"scheduled"` \| `"consensus"` |
| `duration_type` | `string` | `"short"` \| `"normal"`        |

#### `players[]` の値

| フィールド            | 型               | 説明                                                                                       |
| --------------------- | ---------------- | ------------------------------------------------------------------------------------------ |
| `user.discord_user_id` | `string`        | Discord ユーザー ID                                                                        |
| `user.username`       | `string`         | Discord ユーザー名                                                                         |
| `user.global_name`    | `string \| null` | Discord 表示名                                                                             |
| `user.avatar_url`     | `string \| null` | アバター画像 URL                                                                           |
| `is_owner`            | `boolean`        | 卓主か                                                                                     |
| `requested_power`     | `string \| null` | 希望国（`a` / `e` / `f` / `g` / `i` / `r` / `t`）。希望なしは `null`。下記の公開範囲を参照 |
| `power`               | `string \| null` | 担当国（`a` / `e` / `f` / `g` / `i` / `r` / `t`）。割り当て前は `null`                      |

`requested_power` の公開範囲:

- 卓主の希望国は全員に公開する
- 卓主以外の希望国は本人にのみ公開し、それ以外（未認証を含む）には `null` を返す

#### `status` の値

| 値            | 意味         |
| ------------- | ------------ |
| `preparing`   | 準備中       |
| `ready`       | 開始待ち     |
| `in_progress` | 進行中       |
| `solo`        | 単独勝利終了 |
| `draw`        | 引き分け終了 |
| `closed`      | クローズ済み |
| `aborted`     | 中止済み     |

#### `seasons` の値

フェイズ履歴をシーズン単位に集約した配列。各要素は以下のいずれか。

| 値           | 意味                                   |
| ------------ | -------------------------------------- |
| `"ready"`    | 開始前フェイズ                         |
| `"1901s"` 等 | 春ターン（年数 + `s`）                 |
| `"1901f"` 等 | 秋ターン / 調整フェイズ（年数 + `f`）  |
| `"debrief"`  | 終戦フェイズ（`solo` / `draw` 時のみ） |

同一シーズン内の複数フェイズ（例: SpringMain と SpringRetreat）は重複排除される。

#### `phase_kind` の値

| 値             | 意味                       |
| -------------- | -------------------------- |
| `"ready"`      | 開始前                     |
| `"main"`       | 春または秋のメインフェイズ |
| `"retreat"`    | 撤退フェイズ               |
| `"adjustment"` | 調整フェイズ               |
| `"debrief"`    | 終戦フェイズ               |

## エラー形式（共通）

```json
{
  "code": "invalid_request | not_found | repository_error",
  "message": "人間向け説明"
}
```

## 全エラーケース

### `400 Bad Request` — `code: invalid_request`

| 条件                             | message                  |
| -------------------------------- | ------------------------ |
| `game_uuid` が有効な UUID でない | `"game_uuid is invalid"` |

### `404 Not Found` — `code: not_found`

| 条件                           | message            |
| ------------------------------ | ------------------ |
| 指定した UUID の卓が存在しない | `"game not found"` |

### `503 Service Unavailable` — `code: repository_error`

| 条件             | message                   |
| ---------------- | ------------------------- |
| データベース障害 | `"repository error: ..."` |
| 参加者のユーザーレコードが存在しない | `"repository error: ..."` |

## curl

```sh
curl '{base_path}/games/{game_uuid}'

# 卓主判定付き
curl '{base_path}/games/{game_uuid}' \
  -H 'Authorization: Bearer <access_token>'
```
