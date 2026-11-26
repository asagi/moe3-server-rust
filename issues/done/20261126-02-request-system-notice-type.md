# システムメッセージに種類を付ける依頼

フロント（moe3-front-rr）では、今はシステムメッセージをすべて同じ見た目（点線の枠で中央）で出している。今後、種類ごとに出し分けたい。例：

- 盤面編集の通知（ユニットの配置・除去など）が続いたら畳む
- シーズン開始・単独勝利・講和成立などの区切りを目立たせる
- 国名の代わりに国の画像を出す

今のサーバーは、システムメッセージを保存するときに `SystemNoticeCatalog` を文字列（`context`）にしてしまい、種類を DB に残していない（`MessageKind::System(SystemNotice {})` の中身は空）。そのため、フロントからは `kind: "system"` と本文しか見えない。本文の文字列から種類を推測するのは、文言が変わると壊れるので避けたい。

項目ごとに「必須」「任意」を付けた。フィールド名は案なので、サーバー側の命名規則に合わせて変えてよい。

---

## 1. システムメッセージに種類を付けて返す【必須】

システムメッセージ（`kind: "system"`）に、`notice` を足してほしい。システムメッセージ以外は `null`（またはフィールドごと省略）。

対象の API：

- `GET /games/:game_uuid/logs/:season`
- SSE（`GET /games/:game_uuid/messages/stream`）の `message` イベント

### 例

```json
{
  "message_uuid": "…",
  "sender": null,
  "sender_kind": "system",
  "sender_user": null,
  "turn": "1901s",
  "context": "German A mun が配置されました。",
  "kind": "system",
  "recipients": null,
  "created_at": "2026-11-26 18:29",
  "notice": {
    "type": "unit_placed",
    "params": { "unit": { "power": "g", "kind": "a", "location": "mun" } }
  }
}
```

- `context`（本文）は今までどおり返す。フロントは、知らない `type` や `notice` が無いメッセージには本文を出す。
- `type` は `SystemNoticeCatalog` のバリアント名を snake_case にしたもの。
- `params` には、本文を組み立てるのに使った値を、**表示名ではなくコード**で入れてほしい（国は `"g"`、地域は `"mun"` / `"stp_sc"`、ユニット種別は `"a"` / `"f"`）。フロントで画像やリンクに変えるため。

### `type` と `params` の一覧（案）

| `type`                          | `params`                                                                 |
| ------------------------------- | ------------------------------------------------------------------------ |
| `game_created`                  | `{ "username": "alice" }`                                                |
| `player_joined`                 | `{ "username": "alice" }`                                                |
| `ready`                         | `{}`                                                                     |
| `aborted`                       | `{}`                                                                     |
| `start_season`                  | `{ "season": "1901s" }`（`current_turn()` と同じ形）                     |
| `draw_proposed`                 | `{}`                                                                     |
| `owner_absent`                  | `{}`                                                                     |
| `draw_rescinded`                | `{}`                                                                     |
| `solo`                          | `{ "power": "f" }`                                                       |
| `draw`                          | `{}`                                                                     |
| `closed`                        | `{}`                                                                     |
| `unit_placed`                   | `{ "unit": Unit }`                                                       |
| `unit_replaced`                 | `{ "old_unit": Unit, "new_unit": Unit }`                                 |
| `unit_removed`                  | `{ "unit": Unit }`                                                       |
| `territory_set`                 | `{ "province": "mun", "power": "g" }`                                    |
| `territory_replaced`            | `{ "province": "mun", "old_power": "a", "new_power": "g" }`              |
| `territory_released`            | `{ "province": "mun", "old_power": "g" }`                                |
| `progress_mode_changed`         | `{}`                                                                     |
| `progress_consented`            | `{ "power": "f" }`                                                       |
| `progress_consensus_rescinded`  | `{ "power": "f" }`                                                       |
| `progress_consensus_reached`    | `{}`                                                                     |
| `next_update_at_changed`        | `{ "next_update_at": "2026-11-27 21:00" }`（JST、`"YYYY-MM-DD HH:mm"`）  |

- `Unit` は `GET /board` の `units[]` と同じ形（`{ "power": "g", "kind": "a", "location": "mun" }`）。
- `username` は Discord のユーザー名（`@` なし）。数値の Discord ユーザー ID は入れないでほしい（フロントでは出さない方針）。
- 今後システムメッセージを増やすとき（`issues/20260508-02` の無政府からの復帰など）も、同じように `type` を付けてほしい。

## 2. 種類を DB に保存する【必須】

1 を実現するため、システムメッセージを保存するときに、種類と値も残してほしい（例：`messages` テーブルに `notice_type` と `notice_params`（JSON）の列を足す）。保存のしかたはサーバー側で決めてよい。

## 3. 保存済みのメッセージにも種類を付ける【任意】

すでに保存されているシステムメッセージには種類が残っていない。サーバーは本文のテンプレート（`SystemNoticeCatalog` の `Display`）を持っているので、本文から種類と値を逆算して埋める移行処理を、可能なら入れてほしい。

- 逆算できないメッセージは `notice: null` のままでよい（フロントは本文を出す）。
- 手間に見合わなければやらなくてよい。その場合、過去の卓のシステムメッセージは今までどおりの見た目で出す。

## 4. ドキュメント

- `games-game-uuid-logs-season__get.md` と `games-game-uuid-messages-stream__get.md` に `notice` と `type` の一覧を追記してほしい。

## 5. テスト

- システムメッセージに `notice.type` と `notice.params` が入って返る（いくつかの種類で）。
- システムメッセージ以外は `notice` が `null`（または省略）。
- `params` の国・地域・ユニットがコードで入っている（表示名ではない）。
- SSE で届くシステムメッセージにも `notice` が入っている。
- （3 をやる場合）移行後、既存の本文から種類と値が正しく復元される。復元できない本文は `null` のまま。
