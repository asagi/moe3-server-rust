# POST /games/:game_uuid/messages

## 概要

- 卓に発言（公式声明・機密書簡・独り言・管理人発言）を投稿する API
- 発言は現在のシーズン（`ready` / `1901s` / … / `debrief`）に属するものとして記録する（過去のシーズンには投稿できない）
- 投稿したメッセージは、[`GET /games/:game_uuid/messages/stream`](./games-game-uuid-messages-stream__get.md) の購読者にも配信される

## HTTP

- Method: `POST`
- Path: `/games/:game_uuid/messages`
- Content-Type: `application/json`

## ヘッダ

- Authorization: 必須（`Bearer <access_token>`）
- X-Impersonate-User-Name: 任意（管理者による視点切り替え。切り替え先ユーザーの発言として記録する）

## パスパラメータ

- `game_uuid` (`string`): 対象卓の UUID（UUID v7 形式）

## リクエストボディ

| フィールド   | 型                 | 必須 | 説明                                                                         |
| ------------ | ------------------ | ---- | ---------------------------------------------------------------------------- |
| `kind`       | `string`           | 必須 | `"public"`（公式声明）/ `"confidential"`（機密書簡）/ `"personal"`（独り言） |
| `recipients` | `string[] \| null` | 任意 | 宛先の国コード。`confidential` のときだけ必須で、それ以外は省略または `null` |
| `context`    | `string`           | 必須 | 本文。空白だけは不可。最大 2000 文字                                         |
| `as_admin`   | `boolean`          | 任意 | 省略時 `false`。`true` なら管理人発言                                        |

- `recipients` は自国を含めず、重複なし、1 か国以上。国コードの大文字・小文字は区別しない。
- 保存時、`recipients` は国の並び順（`a` `e` `f` `g` `i` `r` `t`）に並べ替える。

## 送信者と権限

| 卓のステータス                    | 発言できる人                 | 使える `kind`                          | 記録される送信者                  |
| --------------------------------- | ---------------------------- | -------------------------------------- | --------------------------------- |
| `preparing` / `ready`（開戦前夜） | 参加者                       | `public` のみ                          | ユーザー（`sender_kind: "user"`） |
| `in_progress`                     | 担当国が滅亡していない参加者 | `public` / `confidential` / `personal` | 担当国（`sender_kind: "power"`）  |
| `solo` / `draw`（感想戦）         | 参加者                       | `public` のみ                          | ユーザー（`sender_kind: "user"`） |
| `closed` / `aborted`              | 投稿不可                     | -                                      | -                                 |

- 「滅亡」とは、最新フェイズに担当国のユニットが存在しない状態を指す。

### 管理人発言（`as_admin: true`）

- `is_admin` のユーザー本人だけが使える。`X-Impersonate-User-Name` 付きのリクエストでは使えない。
- 卓に参加していなくても、`closed` / `aborted` 以外のすべてのステータスで投稿できる。
- `kind` は `"public"` のみ（オープンチャンネルにだけ書ける）。
- 管理人が卓に参加していても、担当国ではなく管理人の発言（`sender_kind: "admin"`）として記録する。
- 管理人発言は `kind: "public"` なので、外交履歴取得 API で全員（第三者を含む）に見える。

## リクエスト例

```json
{
  "kind": "confidential",
  "recipients": ["e", "g"],
  "context": "ベルギーは譲ってほしい。",
  "as_admin": false
}
```

## 成功レスポンス

- Status: `201 Created`
- 作成したメッセージを、[外交履歴取得 API](./games-game-uuid-logs-season__get.md) の `messages[]` の 1 件と同じ形で返す。

```json
{
  "message_uuid": "019715e1-b123-7abc-8def-000000000001",
  "sender": "f",
  "sender_kind": "power",
  "sender_user": null,
  "turn": "1901s",
  "context": "ベルギーは譲ってほしい。",
  "kind": "confidential",
  "recipients": ["e", "g"],
  "created_at": "2026-11-24 21:00"
}
```

## エラー

| Status | `code`                        | 条件                                                                             |
| ------ | ----------------------------- | -------------------------------------------------------------------------------- |
| `400`  | `invalid_request`             | `game_uuid` が有効な UUID でない                                                 |
| `400`  | `invalid_kind`                | `kind` が不正 / 開戦前夜・感想戦に `public` 以外 / 管理人発言に `public` 以外    |
| `400`  | `invalid_recipients`          | `confidential` で宛先なし・空・重複・不明な国・自国を含む / それ以外で宛先を指定 |
| `400`  | `empty_context`               | 本文が空（空白だけを含む）                                                       |
| `400`  | `context_too_long`            | 本文が 2000 文字を超える                                                         |
| `401`  | `unauthorized`                | Authorization ヘッダが無い・不正、またはアクセストークンが未登録                 |
| `403`  | `not_a_player`                | 通常の発言で、リクエストユーザーが卓の参加者でない                               |
| `403`  | `power_eliminated`            | 進行中に、担当国が滅亡しているプレイヤーが発言しようとした                       |
| `403`  | `admin_post_not_allowed`      | 管理人発言で、管理者でない / `X-Impersonate-User-Name` 付き                      |
| `403`  | `game_not_accepting_messages` | 卓が `closed` / `aborted`                                                        |
| `404`  | `not_found`                   | 卓が存在しない                                                                   |
| `503`  | `repository_error`            | データベース障害                                                                 |

- チェックの順序は「認証 → リクエストボディの形式 → 卓の存在 → 卓のステータス → 権限 → シーズンと種別の組み合わせ」。

## ビジネスルール

- この API はリクエストハンドラ内で `game_update_lock` を取得して実行される（投稿先シーズンの判定と保存の間にフェイズが進まないようにするため）。
- 送信ユーザーの UUID はすべての発言で記録するが、レスポンスで送信ユーザーを返すのは `sender_kind: "user"` のときだけ。

## curl

```bash
curl -X POST '{base_path}/games/{game_uuid}/messages' \
  -H 'Authorization: Bearer token-1' \
  -H 'Content-Type: application/json' \
  -d '{"kind":"public","context":"よろしくお願いします。"}'
```
