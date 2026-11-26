# PUT /games/:game_uuid/orders

## 概要

- 担当国のある参加者が、最新フェイズの自分の命令を**丸ごと置き換えて**確定する API
- フロントはブラウザの中で命令を編集し、「更新」ボタンでこの API を 1 回呼ぶ
- 候補（[命令取得 API](./games-game-uuid-orders__get.md) の `options` / `adjustment`）にない命令が 1 つでもあれば、何も変えずに `400` を返す
- 成功したら、命令取得 API と同じ形で確定後の状態を返す
- 管理者の視点変更（`X-Impersonate-User-Name`）中は、切り替え先のアカウントとして扱う
- `game_update_lock` を取って実行するので、定時進行とぶつからない
- 命令の確定は他国に知らせない（システムメッセージ・SSE イベントは出さない）

## HTTP

- Method: `PUT`
- Path: `/games/:game_uuid/orders`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- Content-Type: `application/json`

## パスパラメータ

- `game_uuid` (`string`): 対象卓の UUID（UUID v7 形式）

## リクエストボディ

```json
{
  "season": "1901s",
  "phase_kind": "main",
  "orders": [
    { "unit_power": "f", "unit_kind": "a", "location": "par", "order_kind": "move", "dest": "bur", "via_convoy": false },
    { "unit_power": "f", "unit_kind": "f", "location": "bre", "order_kind": "support", "target_location": "par", "target_dest": "pic" },
    { "unit_power": "g", "unit_kind": "a", "location": "mun", "order_kind": "move", "dest": "bur" }
  ]
}
```

| フィールド   | 型        | 必須 | 説明                                                                                       |
| ------------ | --------- | ---- | ------------------------------------------------------------------------------------------ |
| `season`     | `string`  | 必須 | フロントが見ていたフェイズのシーズン。最新フェイズと違えば `409`                           |
| `phase_kind` | `string`  | 必須 | フロントが見ていたフェイズの種別（`main` / `retreat` / `adjustment`）。違えば `409`        |
| `orders`     | `Order[]` | 任意 | 確定する命令（省略・空配列なら、すべて初期値に戻す）                                       |

### `Order`

| フィールド        | 型        | 必須                                              | 説明                                                   |
| ----------------- | --------- | ------------------------------------------------- | ------------------------------------------------------ |
| `unit_power`      | `string`  | 必須                                              | 対象ユニットの国。担当国と違えば仮定命令（メインのみ） |
| `unit_kind`       | `string`  | 必須                                              | `"a"` / `"f"`                                          |
| `location`        | `string`  | 必須                                              | 対象ユニットの位置（増設は増設先）                     |
| `order_kind`      | `string`  | 必須                                              | `hold` / `move` / `support` / `convoy` / `retreat` / `build` / `disband` |
| `dest`            | `string`  | `move` / `retreat` で必須                         | 移動先・撤退先                                         |
| `target_location` | `string`  | `support` / `convoy` で必須                       | 対象ユニットの位置                                     |
| `target_dest`     | `string`  | `convoy` で必須。`support` は移動の支援のときだけ | 対象ユニットの移動先                                   |
| `via_convoy`      | `boolean` | 任意（既定 `false`）                              | 海路移動か（`move` のみ）                              |

- 地域コードは `GET /board` と同じ形（小文字、海岸は `spa_nc` など）。

## 置き換えの規則

| フェイズ | 使える命令                                    | 含まれていないもの                                                                               |
| -------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| メイン   | `hold` / `move` / `support` / `convoy`        | 自国ユニットは維持。自分の仮定命令は消える（維持扱い）                                           |
| 撤退     | `retreat` / `disband`（自国の撃退されたユニットのみ） | 解体                                                                                     |
| 調整     | `build` / `disband`（自国のみ）               | 自国の命令はリクエストのものだけになる                                                           |

- 1 つのユニットに 2 つ以上の命令を出すと `400`。
- メインフェイズでは、他国ユニットへの命令は仮定命令として保存する（本人にしか見せない。フェイズが変われば引き継がない）。維持の仮定命令は保存しない。
- 調整フェイズでは、増設は `build_count` 以下、解体はちょうど `disband_count` 個でなければ `400`。同じ地域への増設は 1 つまで。
- ほかの国の命令・ほかの人の仮定命令は変えない。

## 成功レスポンス

- Status: `200 OK`
- [命令取得 API](./games-game-uuid-orders__get.md) と同じ形（確定後の状態）

## エラー形式（共通）

```json
{
  "code": "invalid_request | invalid_order | unauthorized | forbidden | not_found | phase_conflict | repository_error",
  "message": "人間向け説明",
  "location": "par"
}
```

- `location` は `invalid_order` で、だめだった命令の対象地域が分かる場合だけ付く（命令数の過不足のように特定の命令によらない場合は省略）。

## 全エラーケース

| Status | `code`             | 条件                                                                           | `message`（例）                                                       |
| ------ | ------------------ | ------------------------------------------------------------------------------ | --------------------------------------------------------------------- |
| `400`  | `invalid_request`  | `game_uuid` が有効な UUID でない                                               | `"game_uuid is invalid"`                                              |
| `400`  | `invalid_order`    | 地域コード・国コード・ユニット種別・命令種別が不正、必須フィールドが無い       | `"invalid province code: xxx"` / `"dest is required"`                 |
| `400`  | `invalid_order`    | 対象ユニットが盤面に無い                                                       | `"unit not found at the location"`                                    |
| `400`  | `invalid_order`    | 候補にない移動先・支援先・輸送先・撤退先・増設先                               | `"destination is not reachable"` など                                 |
| `400`  | `invalid_order`    | フェイズで使えない命令種別                                                     | `"order kind is not available in a main phase"` など                  |
| `400`  | `invalid_order`    | 同じユニットへの命令の重複                                                     | `"duplicate orders for the same unit"`                                |
| `400`  | `invalid_order`    | 増設が多すぎる、解体の数が合わない（`location` なし）                          | `"too many builds: at most 2 allowed"` / `"exactly 1 disbands required"` |
| `401`  | `unauthorized`     | Authorization ヘッダが無い・形式不正・アクセストークン未登録                   | `"unauthorized"`                                                      |
| `403`  | `forbidden`        | 参加者でない・担当国が無い・卓が進行中でない・フェイズがメイン／撤退／調整でない | 命令取得 API と同じ                                                 |
| `403`  | `forbidden`        | `needs_action` が `false` の国                                                 | `"power does not need to act in the current phase"`                   |
| `404`  | `not_found`        | 卓が存在しない                                                                 | `"game not found"`                                                    |
| `409`  | `phase_conflict`   | `season` / `phase_kind` が最新フェイズと違う                                   | `"phase has changed since this request was issued"`                   |
| `422`  | —                  | リクエストボディが JSON として不正（axum の既定のエラー）                      | —                                                                     |
| `500`  | `repository_error` | データベース障害                                                               | `"repository error: ..."`                                             |

- `409` の判定は、参加者かどうかの判定のあと、フェイズの種別による `403` の判定より先に行う（古い画面から確定したときに `409` を返すため）。

## curl

```bash
curl -X PUT '{base_path}/games/{game_uuid}/orders' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"season":"1901s","phase_kind":"main","orders":[{"unit_power":"f","unit_kind":"a","location":"par","order_kind":"move","dest":"bur"}]}'
```
