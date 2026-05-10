# 外交履歴取得 API の実装

## エンドポイント

- GET /games/:game_uuid/logs/:season

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）
- `season` (`string`): `"ready" | "1901s" | "1901f" | ... | "debrief"`

## クエリストリング

- `after` (`string | null`): 最後に取得したメッセージの UUID（UUID v7）。指定した UUID より後のメッセージのみ返す。未指定の場合は全件返す。

## 機能

- 指定したシーズンの外交メッセージ履歴を返す。
- 卓のステータスが InProgress の場合は、アクセス権に応じてメッセージの内容が制限される。
  - 未認証でも取得可能（内容をそのまま返す）
    - Public
    - System
  - プレイヤーのみ取得可能（内容をそのまま返す）
    - sender が自分の担当 Power の Personal
    - sender か recipients に自分の担当 Power が含まれている Confidential
  - 担当 Power が滅亡したプレイヤーのみ取得可能（内容をそのまま返す）
    - Ghost
  - Confidential のうち、sender にも recipients にも自分の担当 Power が含まれていない場合は、`kind` を `"masked"`、`context` を `null` として返す。
    - いつ・どのプレイヤー間で会話があったかは全員に伝わるが、内容は秘匿される。
- 卓のステータスが InProgress 以外であれば誰でも全てのメッセージを取得できる。
- 注意： プレイヤーとはあくまでその卓に参加しているユーザーを指す。

## 成功レスポンス

- Status: `200 OK`

```json
{
  "messages": [
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": "f",
      "turn": "1901s",
      "context": "フランスからの公式声明",
      "kind": "public",
      "recipients": null,
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": "f",
      "turn": "1901s",
      "context": "秘密の提案",
      "kind": "confidential",
      "recipients": ["e", "g"],
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": "a",
      "turn": "1901s",
      "context": null,
      "kind": "masked",
      "recipients": ["e", "g"],
      "created_at": "2026-05-10 20:00"
    },
    {
      "message_uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "sender": null,
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

| フィールド     | 型                 | 説明                                                                                       |
| -------------- | ------------------ | ------------------------------------------------------------------------------------------ |
| `message_uuid` | `string`           | メッセージの UUID（UUID v7）                                                               |
| `sender`       | `string \| null`   | 送信者の国コード（1文字）。システムメッセージは `null`                                     |
| `turn`         | `string`           | 帰属ターン（例: `"1901s"`）                                                                |
| `context`      | `string \| null`   | メッセージ本文。`masked` の場合は `null`                                                   |
| `kind`         | `string`           | 種別（`"public"` / `"confidential"` / `"masked"` / `"personal"` / `"ghost"` / `"system"`） |
| `recipients`   | `string[] \| null` | 宛先の国コード一覧。`confidential` / `masked` のみ設定され、それ以外は `null`              |
| `created_at`   | `string`           | 送信日時（JST、`"YYYY-MM-DD HH:mm"` 形式）                                                 |

## エラー

- 404: 指定した卓またはシーズンが存在しない場合

## その他

- メッセージ種別や宛先によるフィルタリングはフロント側で行う。パフォーマンス上の問題が生じた場合はサーバー側でのフィルタ対応を検討する。
