# GET /games/:game_uuid/result/:season

## 概要

- 指定したシーズンの命令・解決結果とユニット配置を取得する API
- 認証不要

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid/result/:season`

## パスパラメータ

| パラメータ  | 型       | 説明                                                              |
| ----------- | -------- | ----------------------------------------------------------------- |
| `game_uuid` | `string` | 対象の卓の UUID（UUID v7 形式）                                   |
| `season`    | `string` | 対象シーズン（例: `"1901s"` / `"1902f"`）                        |

### season の形式

`season` は `{年}{末尾文字}` の形式。末尾文字によって対象フェイズが異なる。

| season 末尾 | 対象フェイズ                           |
| ----------- | -------------------------------------- |
| `s`         | SpringMain（春メイン）、SpringRetreat（春撤退） |
| `f`         | FallMain（秋メイン）、FallRetreat（秋撤退）、Adjustment（調整） |

`"ready"` / `"debrief"` はこの API では扱わない（`404` を返す）。

## 成功レスポンス

- Status: `200 OK`

```json
{
  "units": [
    {
      "power": "e",
      "kind": "a",
      "location": "Lon"
    }
  ],
  "main_phase": {
    "status": "closed",
    "orders": [
      {
        "power": "e",
        "unit_kind": "a",
        "location": "Lon",
        "order_kind": "hold",
        "status": "success"
      },
      {
        "power": "f",
        "unit_kind": "a",
        "location": "Par",
        "order_kind": "move",
        "status": "success",
        "dest": "Bur",
        "via_convoy": false
      },
      {
        "power": "g",
        "unit_kind": "a",
        "location": "Mun",
        "order_kind": "support",
        "status": "success",
        "target_location": "Boh",
        "target_dest": "Tyr"
      },
      {
        "power": "e",
        "unit_kind": "f",
        "location": "Nth",
        "order_kind": "convoy",
        "status": "success",
        "target_location": "Lon",
        "target_dest": "Hol"
      }
    ]
  },
  "retreat_phase": null,
  "adjustment_phase": null
}
```

### トップレベルフィールド

| フィールド         | 型                             | 説明                                                                    |
| ------------------ | ------------------------------ | ----------------------------------------------------------------------- |
| `units`            | `UnitEntry[]`                  | シーズン最後尾フェイズのユニット配置                                    |
| `main_phase`       | `PhaseResult`                  | メインフェイズ（SpringMain / FallMain）の命令結果                       |
| `retreat_phase`    | `PhaseResult \| null`          | 撤退フェイズ（SpringRetreat / FallRetreat）の命令結果。存在しない場合は `null` |
| `adjustment_phase` | `PhaseResult \| null`          | 調整フェイズ（Adjustment）の命令結果。秋シーズンのみ、存在しない場合は `null`。春シーズンは常に `null` |

### `UnitEntry` フィールド

| フィールド  | 型       | 説明                               |
| ----------- | -------- | ---------------------------------- |
| `power`     | `string` | 国コード（1文字小文字）            |
| `kind`      | `string` | ユニット種別（`"a"` 陸軍 / `"f"` 海軍） |
| `location`  | `string` | 位置（省略形、例: `"Lon"` / `"Nth"`） |

### `PhaseResult` フィールド

| フィールド | 型            | 説明                                          |
| ---------- | ------------- | --------------------------------------------- |
| `status`   | `string`      | フェイズ状態（`"open"` 現在進行中 / `"closed"` 解決済み） |
| `orders`   | `OrderEntry[]` | 命令一覧（仮定命令を除く）                   |

### `OrderEntry` フィールド

| フィールド        | 型               | 説明                                                                          |
| ----------------- | ---------------- | ----------------------------------------------------------------------------- |
| `power`           | `string`         | 命令を出した国コード                                                          |
| `unit_kind`       | `string`         | 命令対象ユニット種別（`"a"` / `"f"`）                                         |
| `location`        | `string`         | 命令対象ユニットの現在位置                                                    |
| `order_kind`      | `string`         | 命令種別（下表参照）                                                          |
| `status`          | `string`         | 解決結果（下表参照）                                                          |
| `dest`            | `string \| 省略` | 移動先（`"move"` / `"retreat"` のみ）                                         |
| `target_location` | `string \| 省略` | 支援・輸送対象ユニットの現在位置（`"support"` / `"convoy"` のみ）             |
| `target_dest`     | `string \| 省略` | 支援・輸送対象ユニットの移動先（`"support"` は移動支援時のみ、`"convoy"` は常に） |
| `via_convoy`      | `bool \| 省略`   | 海路移動フラグ（`"move"` のみ）                                               |

省略フィールドは、不要な場合はレスポンスに含まれない（`null` ではなくフィールドごと省略）。

#### `order_kind` の値

| 値         | 説明                 |
| ---------- | -------------------- |
| `"hold"`   | 維持                 |
| `"move"`   | 移動                 |
| `"support"`| 支援                 |
| `"convoy"` | 輸送                 |
| `"retreat"`| 撤退                 |
| `"build"`  | 建造（調整フェイズ） |
| `"disband"`| 解体（調整フェイズ） |

#### `status` の値

| 値              | 説明                                       |
| --------------- | ------------------------------------------ |
| `"unresolved"`  | 未解決（メインフェイズ進行中）             |
| `"success"`     | 成功                                       |
| `"failure"`     | 失敗                                       |
| `"dislodged"`   | 撃退（ユニットが退却を余儀なくされた）     |
| `"cut"`         | 支援カット                                 |
| `"valid"`       | 有効（調整フェイズ）                       |
| `"invalid"`     | 無効（調整フェイズ）                       |
| `"unreachable"` | 到達不能                                   |

## エラー

| Status | 説明                                                                      |
| ------ | ------------------------------------------------------------------------- |
| `400`  | `game_uuid` が UUID 形式でない場合                                        |
| `404`  | 指定した卓が存在しない場合、`season` が `"ready"` / `"debrief"` の場合、`season` がゲームに存在しない場合（未到達シーズン・不正な文字列を含む） |
