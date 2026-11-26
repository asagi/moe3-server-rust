# PUT /admin/games/:game_uuid/units/:location

## 概要

- 卓主または管理者（`is_admin` のアカウント）が卓の指定地点にユニットを配置または置換する API
- 既存ユニットと配置先が重複する場合は既存ユニットおよびその自身の命令を削除してから新ユニットを配置する

## HTTP

- Method: `PUT`
- Path: `/admin/games/:game_uuid/units/:location`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）
- `location` (`string`): 地域コード（例: `par`, `lon`, `spa_nc`）

## リクエストボディ

```json
{
  "unit": {
    "power": "f",
    "kind": "a"
  },
  "season": "1901s"
}
```

### `unit.power` (`string`)

- 担当国1文字コード: `a | e | f | g | i | r | t`

### `unit.kind` (`string`)

- `a` または `army`（陸軍）
- `f` または `fleet`（海軍）

### `season` (`string`)

- 対象フェイズを指定するシーズン文字列（例: `"ready"`, `"1901s"`, `"1901f"`）
- 形式: `"ready"`（開戦前夜）または `<年4桁><s|f>`（大文字小文字不問）
- 現在の最新フェイズのシーズンと一致しない場合は `409 Conflict`

## 成功レスポンス

- Status: `200 OK`

```json
{
  "game_uuid": "019715e1-b123-7abc-8def-000000000001",
  "location": "par",
  "unit": {
    "power": "f",
    "kind": "a"
  }
}
```

## エラー形式

```json
{
  "code": "invalid_request | unauthorized | not_found | forbidden | phase_conflict | repository_error",
  "message": "人間向け説明"
}
```

## エラーケース

### `400 Bad Request` — `code: invalid_request`

| 条件                                                | `message`                                                           |
| --------------------------------------------------- | ------------------------------------------------------------------- |
| Authorization ヘッダ欠落                            | `"authorization header is required"`                                |
| Authorization 形式不正（Bearer プレフィックスなし） | `"authorization must start with 'Bearer <token>'"`                  |
| アクセストークン空（Bearer 後が空文字）             | `"access token is required"`                                        |
| `game_uuid` が有効な UUID でない                    | `"game_uuid is invalid"`                                            |
| `location` が無効な地域コード                       | `"invalid location: <location>"`                                    |
| `unit.power` が無効な国コード                       | `"invalid power: <power>"`                                          |
| `unit.kind` が無効な値                              | `"invalid unit kind: <kind>"`                                       |
| `season` が無効な形式                               | `"season must be 'ready' or in the format like '1901s' or '1901f'"` |
| 陸軍を海域に配置しようとした                        | `"army cannot be placed in a sea province"`                         |
| 陸軍を海岸バリアントコードで指定した                | `"army cannot be placed on a coast variant location"`               |
| 海軍を内陸に配置しようとした                        | `"fleet cannot be placed in an inland province"`                    |
| 海軍を双海岸地域のベースコードで指定した            | `"fleet must specify a coast variant for this location"`            |

### `401 Unauthorized` — `code: unauthorized`

| 条件                                                        | `message`        |
| ----------------------------------------------------------- | ---------------- |
| アクセストークン未登録（DB にユーザーレコードが存在しない） | `"unauthorized"` |

### `403 Forbidden` — `code: forbidden`

| 条件                                                             | `message`                                                       |
| ---------------------------------------------------------------- | --------------------------------------------------------------- |
| リクエストユーザーが当該卓の卓主でも管理者でもない               | `"user is neither the owner of this game nor an administrator"` |
| 卓が終了している（`closed` / `aborted`）                         | `"units cannot be set in a finished game"`                      |
| 卓にフェイズが存在しない                                         | `"game has no phases"`                                          |
| 最新フェイズが開戦前夜・メインフェイズ以外（撤退・調整・感想戦） | `"units can only be set during a ready or main phase"`          |

### `404 Not Found` — `code: not_found`

| 条件                           | `message`          |
| ------------------------------ | ------------------ |
| 指定した UUID の卓が存在しない | `"game not found"` |

### `409 Conflict` — `code: phase_conflict`

| 条件                                               | `message`          |
| -------------------------------------------------- | ------------------ |
| 指定した `season` が現在の最新フェイズと一致しない | `"phase conflict"` |

### `500 Internal Server Error` — `code: repository_error`

| 条件             | `message`                 |
| ---------------- | ------------------------- |
| データベース障害 | `"repository error: ..."` |

## ビジネスルール

- 権限: 卓主または管理者（`users.is_admin`）のみ操作可能。`X-Impersonate-User-Name` 付きのリクエストは成り代わり先のアカウントとして判定する
- フェイズ制限: 最新フェイズが開戦前夜（`Ready`）またはメインフェイズ（`SpringMain` / `FallMain`）のときのみ操作可能。撤退フェイズ・調整フェイズ・感想戦では操作できない
- 終了した卓（`closed` / `aborted`）では操作できない
- 編集対象は最新フェイズのみ（`season` で指定し、最新フェイズと一致しない場合は `409`）
- 既存ユニット置換: 同じベースコードに既存ユニットがある場合、既存ユニットとその自身の命令を削除してから新ユニットを配置（既存ユニットを支援・輸送する他ユニットの命令は残す。一致する命令がなければ解決時に失敗となる）
- Hold 命令自動生成: メインフェイズで新ユニットを配置したときに Hold 命令を自動生成する。開戦前夜では命令を生成しない（1901 年春のメインフェイズ生成時に全ユニットの維持命令が作られる）
- システムメッセージ: ユニット状態が変化した場合のみメッセージ DB に追記（同じ状態への再適用では追記しない）
