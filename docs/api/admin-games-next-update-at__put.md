# PUT /admin/games/:game_uuid/next-update-at

## 概要

- 卓主が自分の卓の次回更新時刻 `next_update_at` を延長する API
- リクエストが成功した場合、システムメッセージをメッセージ DB に追記する
- 本 API は `ready` / メインフェイズ中のみ実行できる

## HTTP

- Method: `PUT`
- Path: `/admin/games/:game_uuid/next-update-at`
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

- `season` (`string`): 現在シーズン（例: `"ready"`, `"1901s"`, `"1903f"`）
- `next_update_at` (`string`): 変更後の次回更新時刻（`YYYY-MM-DD HH:MM`、JST）

## リクエスト例

```json
{
  "season": "1901s",
  "next_update_at": "2099-06-15 14:00"
}
```

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "next_update_at": "2099-06-15 14:00"
}
```

注意:

- `next_update_at` はリクエストで指定した値（JST文字列）をそのまま返す
- 変更が発生した場合のみ、システムメッセージ「次回更新時刻が {next_update_at} に変更されました。」を追記する

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | not_found | forbidden | phase_conflict | repository_error",
  "message": "人間向け説明"
}
```

## エラー一覧

| HTTP ステータス | code               | 説明                                                                |
| --------------- | ------------------ | ------------------------------------------------------------------- |
| 400             | `invalid_request`  | リクエスト形式不正（ヘッダ欠落、season 形式不正、日時形式不正など） |
| 401             | `unauthorized`     | アクセストークンが無効または未指定                                  |
| 403             | `forbidden`        | 卓主以外、または `ready` / メインフェイズ以外での実行               |
| 404             | `not_found`        | 指定 `game_uuid` の卓が存在しない                                   |
| 409             | `phase_conflict`   | リクエストの `season` とサーバー側の現在シーズンが不一致            |
| 500             | `repository_error` | DB 操作エラー                                                       |

## ビジネスルール

- 卓主のみ実行可能
- 実行可能フェイズ: `Ready` / `SpringMain` / `FallMain`
- `season` は `"ready"` または `"YYYYs"` / `"YYYYf"`（英字の大小文字は許容）
- `next_update_at` は `YYYY-MM-DD HH:MM`（JST）形式
- `next_update_at` の分（MM）は 5 分刻み
- 新しい `next_update_at` が現在の `next_update_at` より過去の場合はエラー（同一時刻は no-op として成功）
- 排他制御: `game_update_lock` を取得した上で実行

## curl

```bash
curl -X PUT https://example.com/admin/games/019715e1-b123-7abc-8def-000000000001/next-update-at \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"season":"1901s","next_update_at":"2099-06-15 14:00"}'
```
