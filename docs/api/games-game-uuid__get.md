# GET /games/:game_uuid

## 概要

- 指定した卓の情報を取得する API
- 認証不要

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid`

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
    "is_private": false
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

## curl

```sh
curl '{base_path}/games/{game_uuid}'
```
