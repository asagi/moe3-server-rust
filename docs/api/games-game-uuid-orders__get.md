# GET /games/:game_uuid/orders

## 概要

- 担当国のある参加者が、最新フェイズで自分が出している命令と、命令の候補を取得する API
- 最新フェイズがメイン・撤退・調整のときだけ使える
- 候補（移動先・支援先・輸送先・撤退先・増設先）はサーバーが作る。フロントは隣接情報を持たなくてよい
- 管理者の視点変更（`X-Impersonate-User-Name`）中は、切り替え先のアカウントとして扱う

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid/orders`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）

## パスパラメータ

- `game_uuid` (`string`): 対象卓の UUID（UUID v7 形式）

## 成功レスポンス

- Status: `200 OK`

### メインフェイズの例

```json
{
  "season": "1901s",
  "phase_kind": "main",
  "power": "f",
  "needs_action": true,
  "orders": [
    { "unit_power": "f", "unit_kind": "a", "location": "par", "order_kind": "move", "dest": "bur", "via_convoy": false },
    { "unit_power": "f", "unit_kind": "f", "location": "bre", "order_kind": "hold" },
    { "unit_power": "g", "unit_kind": "a", "location": "mun", "order_kind": "move", "dest": "bur", "via_convoy": false }
  ],
  "options": [
    {
      "unit_power": "e",
      "unit_kind": "a",
      "location": "lon",
      "move_dests": ["wal", "yor"],
      "convoy_dests": ["bel", "den", "edi", "hol", "nwy", "yor"],
      "support_targets": [{ "target_location": "nth", "target_dests": ["yor"] }],
      "convoy_targets": []
    },
    {
      "unit_power": "e",
      "unit_kind": "f",
      "location": "nth",
      "move_dests": ["bel", "den", "edi", "eng", "hel", "hol", "lon", "nrg", "nwy", "ska", "yor"],
      "convoy_dests": [],
      "support_targets": [{ "target_location": "lon", "target_dests": [null, "yor", "bel", "den", "edi", "hol", "nwy"] }],
      "convoy_targets": [{ "target_location": "lon", "target_dests": ["bel", "den", "edi", "hol", "nwy", "yor"] }]
    }
  ],
  "adjustment": null
}
```

### 撤退フェイズの例

```json
{
  "season": "1901s",
  "phase_kind": "retreat",
  "power": "f",
  "needs_action": true,
  "orders": [{ "unit_power": "f", "unit_kind": "a", "location": "bur", "order_kind": "disband" }],
  "options": [{ "unit_power": "f", "unit_kind": "a", "location": "bur", "retreat_dests": ["bel", "gas", "mar", "ruh"] }],
  "adjustment": null
}
```

### 調整フェイズの例

```json
{
  "season": "1901f",
  "phase_kind": "adjustment",
  "power": "r",
  "needs_action": true,
  "orders": [],
  "options": [],
  "adjustment": {
    "build_count": 2,
    "disband_count": 0,
    "buildable": [
      { "location": "mos", "unit_kinds": ["a"] },
      { "location": "sev", "unit_kinds": ["a", "f"] },
      { "location": "stp", "unit_kinds": ["a"] },
      { "location": "stp_nc", "unit_kinds": ["f"] },
      { "location": "stp_sc", "unit_kinds": ["f"] }
    ]
  }
}
```

### トップレベルフィールド

| フィールド     | 型                         | 説明                                                                                       |
| -------------- | -------------------------- | ------------------------------------------------------------------------------------------ |
| `season`       | `string`                   | 最新フェイズのシーズン（例: `"1901s"`）                                                    |
| `phase_kind`   | `string`                   | 最新フェイズの種別（`"main"` / `"retreat"` / `"adjustment"`）                              |
| `power`        | `string`                   | リクエスト者の担当国コード                                                                 |
| `needs_action` | `boolean`                  | このフェイズで操作が必要か（下表）。`false` なら命令は確定できない（`PUT` は `403`）       |
| `orders`       | `OrderEntry[]`             | リクエスト者が今出している命令（自国ユニットへの命令と、メインフェイズでは自分の仮定命令） |
| `options`      | `UnitOptions[]`            | 命令を出せるユニットごとの候補（メイン：盤面の全ユニット、撤退：自国の撃退されたユニット、調整：空） |
| `adjustment`   | `AdjustmentOptions \| null` | 調整フェイズの候補。ほかのフェイズは `null`                                               |

#### `needs_action` の判定（即時進行の「操作が必要な勢力」と同じ）

| フェイズ | 操作が必要な国                                                             |
| -------- | -------------------------------------------------------------------------- |
| メイン   | 補給都市かユニットが残っている国                                           |
| 撤退     | 撃退されたユニットがある国（海路で攻められて撃退されたユニットを含む）     |
| 調整     | 補給都市数とユニット数が異なる国                                           |

### `OrderEntry`

| フィールド        | 型               | 説明                                                                                   |
| ----------------- | ---------------- | -------------------------------------------------------------------------------------- |
| `unit_power`      | `string`         | 命令の対象ユニットの国。担当国と違えば仮定命令                                         |
| `unit_kind`       | `string`         | `"a"`（陸軍）/ `"f"`（海軍）                                                           |
| `location`        | `string`         | 対象ユニットの位置（増設は増設先）。地域コードは `GET /board` と同じ形（海岸は `spa_nc` など） |
| `order_kind`      | `string`         | `hold` / `move` / `support` / `convoy` / `retreat` / `build` / `disband`               |
| `dest`            | `string \| 省略` | 移動先・撤退先（`move` / `retreat` のみ）                                              |
| `target_location` | `string \| 省略` | 支援・輸送の対象ユニットの位置（`support` / `convoy` のみ）                            |
| `target_dest`     | `string \| 省略` | 支援・輸送の対象ユニットの移動先（`support` は移動の支援のときだけ、`convoy` は常に）  |
| `via_convoy`      | `bool \| 省略`   | 海路移動か（`move` のみ）                                                              |

- 命令解決履歴取得 API の `OrderEntry` から `status` を除き、`power` を `unit_power`（対象ユニットの国）にしたもの。
- 自国ユニットには、フェイズ開始時にサーバーが初期値を入れておく（メイン：維持、撤退：解体、調整：解体が必要な国は自動で選んだ解体）。
- 仮定命令を出していない他国ユニットは含めない。フロントは「維持」として表示する。
- 他国の命令・ほかの人の仮定命令は返さない。

### `UnitOptions`

| フィールド        | 型                     | フェイズ | 説明                                                                                                 |
| ----------------- | ---------------------- | -------- | ---------------------------------------------------------------------------------------------------- |
| `unit_power`      | `string`               | 共通     | ユニットの国                                                                                         |
| `unit_kind`       | `string`               | 共通     | `"a"` / `"f"`                                                                                        |
| `location`        | `string`               | 共通     | ユニットの位置                                                                                       |
| `move_dests`      | `string[]`             | メイン   | 陸路（海軍は海路）で移動できる地域。海軍の移動先は海岸まで区別する（`spa_nc` など）                  |
| `convoy_dests`    | `string[]`             | メイン   | 海路移動できる海岸。海軍と内陸の陸軍は空配列                                                         |
| `support_targets` | `SupportTarget[]`      | メイン   | 支援できるユニットと、その支援先                                                                     |
| `convoy_targets`  | `ConvoyTarget[]`       | メイン   | 輸送できる陸軍と、その移動先。海にいる海軍のみ（ほかは空配列）                                       |
| `retreat_dests`   | `string[]`             | 撤退     | 撤退できる地域                                                                                       |

- フェイズに関係しないフィールドは省略する。

#### 候補の作り方

- `convoy_dests`：その時点で盤面にあるすべての海軍（国を問わない）が輸送に同意した場合に、海域を連ねてたどり着ける海岸すべて（自分の地域を除く）。陸路で隣接している海岸も含む（陸路と海路は `via_convoy` で区別する）。
- `support_targets`：盤面のほかのユニットについて、
  - 支援するユニットがそのユニットの地域に移動できるなら、維持の支援（`target_dests` の `null`）
  - 支援されるユニットの移動先（陸路・海路の両方）のうち、支援するユニットが移動できる地域（支援するユニット自身の地域を除く）
  - 支援先が 1 つも無いユニットは含めない
- `convoy_targets`：`convoy_dests` と同じく「すべての海軍が輸送に同意した場合」の経路のうち、その海軍を通るもの。
- `retreat_dests`：陸路（海軍は海路）で移動できる地域から、攻撃元（海路で攻められた場合は除外しない）・スタンドオフが起きた地域・ほかのユニットがいる地域を除いたもの。

### `SupportTarget`

| フィールド        | 型                   | 説明                                            |
| ----------------- | -------------------- | ----------------------------------------------- |
| `target_location` | `string`             | 支援されるユニットの位置                        |
| `target_dests`    | `(string \| null)[]` | 支援先。`null` は維持の支援（あれば先頭に置く） |

### `ConvoyTarget`

| フィールド        | 型         | 説明                   |
| ----------------- | ---------- | ---------------------- |
| `target_location` | `string`   | 輸送される陸軍の位置   |
| `target_dests`    | `string[]` | 輸送できる移動先       |

### `AdjustmentOptions`

| フィールド      | 型                | 説明                                                                                           |
| --------------- | ----------------- | ---------------------------------------------------------------------------------------------- |
| `build_count`   | `number`          | 増設できる最大数（補給都市数 − ユニット数。負なら 0）                                          |
| `disband_count` | `number`          | 解体しなければならない数（ユニット数 − 補給都市数。負なら 0）。解体は自国ユニットならどれでも選べる |
| `buildable`     | `Buildable[]`     | 今空いている本国の補給都市（自国が占領しているもの）と、置けるユニットの種類。`build_count` が 0 なら空 |

#### `Buildable`

| フィールド   | 型         | 説明                                                                       |
| ------------ | ---------- | -------------------------------------------------------------------------- |
| `location`   | `string`   | 増設先。海岸が複数ある地域は、陸軍用の地域（`stp`）と海岸ごと（`stp_nc` / `stp_sc`）に分ける |
| `unit_kinds` | `string[]` | 置けるユニットの種類（`"a"` / `"f"`）                                      |

## エラー形式（共通）

```json
{
  "code": "invalid_request | unauthorized | forbidden | not_found | repository_error",
  "message": "人間向け説明"
}
```

## 全エラーケース

| Status | `code`             | 条件                                                                                 | `message`                                                                |
| ------ | ------------------ | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| `400`  | `invalid_request`  | `game_uuid` が有効な UUID でない                                                     | `"game_uuid is invalid"`                                                 |
| `401`  | `unauthorized`     | Authorization ヘッダが無い・形式不正・アクセストークン未登録                         | `"unauthorized"`                                                         |
| `403`  | `forbidden`        | リクエスト者が卓の参加者でない、または担当国が無い                                   | `"user is not a player with an assigned power"`                          |
| `403`  | `forbidden`        | 卓が進行中（`in_progress`）でない                                                    | `"game is not in progress"`                                              |
| `403`  | `forbidden`        | 最新フェイズがメイン・撤退・調整でない（開戦前夜・感想戦）                           | `"orders are available only in a main, retreat or adjustment phase"`     |
| `404`  | `not_found`        | 卓が存在しない                                                                       | `"game not found"`                                                       |
| `500`  | `repository_error` | データベース障害                                                                     | `"repository error: ..."`                                                |

## curl

```bash
curl '{base_path}/games/{game_uuid}/orders' \
  -H 'Authorization: Bearer token-1'
```
