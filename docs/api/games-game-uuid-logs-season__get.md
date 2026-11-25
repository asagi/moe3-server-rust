# GET /games/:game_uuid/logs/:season

## 概要

- 指定したシーズンの外交メッセージ履歴を取得する API
- 認証任意（認証状態によって返却内容が変わる）

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid/logs/:season`

## パスパラメータ

| パラメータ  | 型       | 説明                                                       |
| ----------- | -------- | ---------------------------------------------------------- |
| `game_uuid` | `string` | 対象の卓の UUID（UUID v7 形式）                            |
| `season`    | `string` | 対象シーズン（例: `"ready"` / `"1901s"` / `"debrief"` 等） |

## クエリストリング

| パラメータ | 型               | 必須 | 説明                                                                                                           |
| ---------- | ---------------- | ---- | -------------------------------------------------------------------------------------------------------------- |
| `after`    | `string \| null` | 任意 | 最後に取得したメッセージの UUID（UUID v7）。指定した UUID より後のメッセージのみ返す。未指定の場合は全件返す。 |

## アクセス制御

卓のステータスが `in_progress` の場合、リクエスト者の認証状態と担当 Power に応じてメッセージの可視性が変わる。

| kind           | 未認証 / 非参加者 | 参加者（担当 Power あり）                                                |
| -------------- | ----------------- | ------------------------------------------------------------------------ |
| `public`       | ✅ 全文           | ✅ 全文                                                                  |
| `ghost`        | ✅ 全文           | ✅ 全文                                                                  |
| `system`       | ✅ 全文           | ✅ 全文                                                                  |
| `personal`     | ❌ 非表示         | ✅ 自分が sender の場合のみ全文                                          |
| `confidential` | ⚠️ masked         | ✅ 自分が sender か recipients に含まれる場合のみ全文、それ以外は masked |

- `masked`: `kind` が `"masked"`、`context` が `null` となる。`sender` / `recipients` / `created_at` は返却される。
- 卓のステータスが `in_progress` 以外の場合は、全員が全メッセージを取得できる（感想戦に入ったらすべての情報を公開する）。
- 管理人発言（`sender_kind: "admin"`）は `kind: "public"` なので、全員が取得できる。
- `ghost`（亡国会話）は、滅亡した国の公式声明。`public` と同じく全員が取得できる。

## 成功レスポンス

- Status: `200 OK`

```json
{
  "messages": [
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": null,
      "sender_kind": "user",
      "sender_user": {
        "username": "asagix",
        "global_name": "あさぎ",
        "avatar_url": "https://cdn.discordapp.com/avatars/..."
      },
      "turn": "ready",
      "context": "よろしくお願いします。",
      "kind": "public",
      "recipients": null,
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": "f",
      "sender_kind": "power",
      "sender_user": null,
      "turn": "1901s",
      "context": "フランスからの公式声明",
      "kind": "public",
      "recipients": null,
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": "f",
      "sender_kind": "power",
      "sender_user": null,
      "turn": "1901s",
      "context": "秘密の提案",
      "kind": "confidential",
      "recipients": ["e", "g"],
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": "a",
      "sender_kind": "power",
      "sender_user": null,
      "turn": "1901s",
      "context": null,
      "kind": "masked",
      "recipients": ["e", "g"],
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": null,
      "sender_kind": "system",
      "sender_user": null,
      "turn": "1901s",
      "context": "1901 年春 のメインフェイズが開始されました。",
      "kind": "system",
      "recipients": null,
      "created_at": "2026-05-10 20:00"
    }
  ]
}
```

### フィールド詳細

| フィールド     | 型                 | 説明                                                                                                                                         |
| -------------- | ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `message_uuid` | `string`           | メッセージの UUID（UUID v7）                                                                                                                 |
| `sender`       | `string \| null`   | 送信者の国コード（1文字）。`sender_kind` が `"power"` 以外は `null`                                                                          |
| `sender_kind`  | `string`           | 送信者種別。`"power"`（国の発言）/ `"user"`（開戦前夜・感想戦のユーザーの発言）/ `"admin"`（管理人発言）/ `"system"`（システムメッセージ）   |
| `sender_user`  | `object \| null`   | `sender_kind` が `"user"` のときの送信ユーザー（`username` / `global_name` / `avatar_url`）。それ以外は `null`。Discord の数値 ID は含めない |
| `turn`         | `string`           | 帰属ターン（例: `"1901s"`）                                                                                                                  |
| `context`      | `string \| null`   | メッセージ本文。`masked` の場合は `null`                                                                                                     |
| `kind`         | `string`           | 種別（`"public"` / `"confidential"` / `"masked"` / `"personal"` / `"ghost"` / `"system"`）                                                   |
| `recipients`   | `string[] \| null` | 宛先の国コード一覧。`confidential` / `masked` のみ設定され、それ以外は `null`                                                                |
| `created_at`   | `string`           | 送信日時（JST、`"YYYY-MM-DD HH:mm"` 形式）                                                                                                   |

## エラー

| Status | 説明                                     |
| ------ | ---------------------------------------- |
| `404`  | 指定した卓またはシーズンが存在しない場合 |
