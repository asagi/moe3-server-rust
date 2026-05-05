# PUT /admin/games/:game_uuid/progress-mode

## 概要

- 卓主が自分の卓の進行モードを合意進行に変更する API
- 既に合意進行の場合はべき等として `200 OK` を返す（DB 更新・メッセージ追記なし）
- 変更が発生した場合のみシステムメッセージ「進行モードが 定時進行 から 合意進行 に変更されました。」をメッセージ DB に追記する

## HTTP

- Method: `PUT`
- Path: `/admin/games/:game_uuid/progress-mode`
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

- `season` (`string`): 現在のシーズン（例: `"1901s"`, `"1903f"`）

## リクエスト例

```json
{
  "season": "1901s"
}
```

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "progress_mode": "consensus"
}
```

注意:

- `progress_mode` は常に `"consensus"` を返す
- 既に合意進行の場合も `200 OK` を返す（べき等）

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | not_found | forbidden | phase_conflict | repository_error",
  "message": "人間向け説明"
}
```

## エラー一覧

| HTTP ステータス | code | 説明 |
|---|---|---|
| 400 | `invalid_request` | リクエストの形式が不正（ヘッダ欠落、season 形式不正など） |
| 401 | `unauthorized` | アクセストークンが無効または未指定 |
| 403 | `forbidden` | 卓主以外からのリクエスト、またはメインフェイズ以外でのリクエスト |
| 404 | `not_found` | 指定した `game_uuid` の卓が存在しない |
| 409 | `phase_conflict` | リクエスト時と処理時でシーズンが異なる |
| 500 | `repository_error` | DB 操作エラー |

## curl 例

```bash
curl -X PUT https://example.com/admin/games/019715e1-b123-7abc-8def-000000000001/progress-mode \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"season": "1901s"}'
```
