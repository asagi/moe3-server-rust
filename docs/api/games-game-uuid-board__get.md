# GET /games/:game_uuid/board

## 概要

- 卓の盤面（ユニット配置・占領情報）と、国ごとの状況を取得する API
- シーズンを指定しなければ最新フェイズ、指定すればそのシーズンの最後のフェイズの盤面を返す
- 命令は含めない（命令は [命令取得 API](./games-game-uuid-orders__get.md) と [命令解決履歴取得 API](./games-game-uuid-result-season__get.md) で取る）
- 認証不要

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid/board`

## パスパラメータ

| パラメータ  | 型       | 説明                            |
| ----------- | -------- | ------------------------------- |
| `game_uuid` | `string` | 対象の卓の UUID（UUID v7 形式） |

## クエリストリング

| パラメータ | 型       | 必須 | 説明                                                                                                |
| ---------- | -------- | ---- | --------------------------------------------------------------------------------------------------- |
| `season`   | `string` | 任意 | 対象シーズン（`ready` / `1901s` / `1901f` / … / `debrief`）。省略時は最新フェイズ。大文字小文字不問 |

- `season` を指定した場合は、そのシーズンの最後のフェイズ（例：`1901s` なら春の撤退フェイズがあればそれ、なければ春のメインフェイズ）を返す。

## 成功レスポンス

- Status: `200 OK`

```json
{
  "season": "1901s",
  "phase_kind": "retreat",
  "units": [
    { "power": "f", "kind": "a", "location": "bur", "dislodged": true, "dislodged_from": "mun" },
    { "power": "g", "kind": "a", "location": "bur", "dislodged": false, "dislodged_from": null },
    { "power": "r", "kind": "f", "location": "stp_sc", "dislodged": false, "dislodged_from": null }
  ],
  "territories": [{ "power": "f", "code": "par" }],
  "powers": [
    {
      "power": "f",
      "supply_centers": 3,
      "armies": 2,
      "fleets": 1,
      "needs_action": true,
      "progress_consented": false,
      "is_idle": false,
      "is_eliminated": false
    }
  ]
}
```

### トップレベルフィールド

| フィールド    | 型                  | 説明                                                                                    |
| ------------- | ------------------- | --------------------------------------------------------------------------------------- |
| `season`      | `string`            | 返したフェイズのシーズン（`ready` / `1901s` / … / `debrief`）                           |
| `phase_kind`  | `string`            | 返したフェイズの種別（`ready` / `main` / `retreat` / `adjustment` / `debrief`）         |
| `units`       | `UnitEntry[]`       | ユニット配置                                                                            |
| `territories` | `TerritoryEntry[]`  | 占領情報                                                                                |
| `powers`      | `PowerStatus[]`     | 国ごとの状況（7 か国、オーストリアから国コード順）。フェイズが無い卓では空配列          |

### `UnitEntry`

| フィールド       | 型               | 説明                                                                                                   |
| ---------------- | ---------------- | ------------------------------------------------------------------------------------------------------ |
| `power`          | `string`         | 国コード（1 文字小文字）                                                                               |
| `kind`           | `string`         | ユニット種別（`"a"` 陸軍 / `"f"` 海軍）                                                                |
| `location`       | `string`         | 位置（地域コード。海岸は `spa_nc` など）                                                               |
| `dislodged`      | `boolean`        | 撃退されたユニットか（撤退フェイズでのみ `true` になりうる）                                           |
| `dislodged_from` | `string \| null` | 攻撃元の地域。撃退されていない場合と、海路で攻められた場合は `null`                                    |

- 撤退フェイズでは、撃退されたユニットと攻めてきたユニットが同じ地域に並ぶ。
- 攻めてきたユニットの移動前の位置は、命令解決履歴取得 API のメインフェイズの命令（成功した移動）から求める。

### `TerritoryEntry`

| フィールド | 型       | 説明                       |
| ---------- | -------- | -------------------------- |
| `power`    | `string` | 占領している国のコード     |
| `code`     | `string` | 地域コード（海岸を含まない） |

### `PowerStatus`

| フィールド           | 型                | 説明                                                                                                                        |
| -------------------- | ----------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `power`              | `string`          | 国コード                                                                                                                    |
| `supply_centers`     | `number`          | 占領している補給都市の数                                                                                                    |
| `armies`             | `number`          | 陸軍の数（撤退フェイズでは撃退されたユニットも数える）                                                                      |
| `fleets`             | `number`          | 海軍の数（同上）                                                                                                            |
| `needs_action`       | `boolean`         | このフェイズで操作が必要か（[命令取得 API](./games-game-uuid-orders__get.md) の `needs_action` と同じ判定）。過去のフェイズでは `false` |
| `progress_consented` | `boolean \| null` | 即時進行に合意しているか。担当者がいない国は `false`。過去のフェイズでは `null`                                             |
| `is_idle`            | `boolean \| null` | 無政府状態か（担当者の最終アクセスが卓の無政府判定時間より前）。担当者がいない国も `true`。過去のフェイズでは `null`         |
| `is_eliminated`      | `boolean`         | 滅亡しているか（補給都市もユニットも無い）。開戦前夜・感想戦では常に `false`                                                |

- 和平合意は卓主だけの権限なので、ここには含めない。

## エラー

| Status | `code`             | `message`            | 条件                                                                  |
| ------ | ------------------ | -------------------- | --------------------------------------------------------------------- |
| `400`  | `invalid_request`  | `"game_uuid is invalid"` | `game_uuid` が UUID 形式でない                                    |
| `404`  | `not_found`        | `"game not found"`   | 指定した卓が存在しない                                                |
| `404`  | `not_found`        | `"season not found"` | `season` がこの卓に存在しない（未到達のシーズン・不正な文字列を含む） |
| `503`  | `repository_error` | `"..."`              | データベース障害                                                      |

## 更新の通知

- 国ごとの状況（補給都市・ユニット数・即時進行の合意）が変わると、[SSE](./games-game-uuid-messages-stream__get.md) の `board` イベントが届く。フロントはこれを受けてこの API を読み直す。
- 命令の確定は他国に知らせないため、`board` イベントは送らない。

## curl

```bash
curl '{base_path}/games/{game_uuid}/board'
curl '{base_path}/games/{game_uuid}/board?season=1901s'
```
