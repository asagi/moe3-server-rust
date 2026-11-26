# GET /games/:game_uuid/messages/stream

## 概要

- 卓の新しいメッセージを Server-Sent Events（SSE）でリアルタイムに配信する API
- 認証任意（認証状態によって配信内容が変わる）。公開範囲は [外交履歴取得 API](./games-game-uuid-logs-season__get.md) と同じ
- シーズンを問わず、卓のすべてのメッセージ（システムメッセージを含む）が対象

## HTTP

- Method: `GET`
- Path: `/games/:game_uuid/messages/stream`
- レスポンスの Content-Type: `text/event-stream`

## ヘッダ

| ヘッダ                    | 必須 | 説明                                                                            |
| ------------------------- | ---- | ------------------------------------------------------------------------------- |
| `Authorization`           | 任意 | `Bearer <access_token>`。無い・不正な場合は第三者として扱う                     |
| `X-Impersonate-User-Name` | 任意 | 管理者による視点切り替え                                                        |
| `Last-Event-ID`           | 任意 | 再接続時に、最後に受け取ったイベントの `id`（`message_uuid`）。`after` より優先 |

- ブラウザの `EventSource` はヘッダを付けられないため、フロントのサーバーが中継する前提。クエリにトークンを載せる方式は無い。

## パスパラメータ

- `game_uuid` (`string`): 対象卓の UUID（UUID v7 形式）

## クエリストリング

| パラメータ | 型       | 必須 | 説明                                                                                 |
| ---------- | -------- | ---- | ------------------------------------------------------------------------------------ |
| `after`    | `string` | 任意 | 指定したメッセージ UUID より後のメッセージから配信する（`Last-Event-ID` と同じ意味） |

- `Last-Event-ID` / `after` のどちらも無い場合は、接続した時点より後に保存されたメッセージだけを配信する。
- どちらかがある場合は、接続直後にその UUID より後のメッセージ（シーズンを問わない）を送り直してから、新しいメッセージの配信を続ける。

## イベント

### `message`：新しいメッセージ

- `id`: `message_uuid`
- `data`: 外交履歴取得 API の `messages[]` の 1 件と同じ JSON
- システムメッセージ（`kind: "system"`）には `notice`（種類と値）が入る。`type` と `params` の一覧は [外交履歴取得 API](./games-game-uuid-logs-season__get.md) の「`notice`（システムメッセージの種類）」を参照。それ以外は `notice: null`

```
id: 019715e1-b123-7abc-8def-000000000001
event: message
data: {"message_uuid":"019715e1-b123-7abc-8def-000000000001","sender":"f","sender_kind":"power","sender_user":null,"turn":"1901s","context":"こんにちは","kind":"public","recipients":null,"created_at":"2026-11-24 21:00","notice":null}

```

システムメッセージの例：

```
id: 019715e1-b123-7abc-8def-000000000002
event: message
data: {"message_uuid":"019715e1-b123-7abc-8def-000000000002","sender":null,"sender_kind":"system","sender_user":null,"turn":"1901s","context":"German A Mun が配置されました。","kind":"system","recipients":null,"created_at":"2026-11-24 21:01","notice":{"type":"unit_placed","params":{"unit":{"power":"g","kind":"a","location":"mun"}}}}

```

### `season`：シーズンが切り替わった

- `data`: 新しいシーズン。フロントはこれを受けて現在のシーズンを読み直す。
- 同じタイミングで届くメッセージより前に送る。

```
event: season
data: {"season":"1901f"}

```

### `phase`：フェイズ種別が切り替わった

- `data`: 切り替わったあとのシーズンとフェイズ種別。
  - `season`: [`current_turn()`](./games-game-uuid__get.md) と同じ形（`ready` / `1901s` / … / `debrief`）
  - `phase_kind`: [`GET /games/:game_uuid`](./games-game-uuid__get.md) の `phase_kind` と同じ値（`ready` / `main` / `retreat` / `adjustment` / `debrief`）
- 最新フェイズの種別が変わったときに送る。メインフェイズ → 撤退フェイズのように、シーズンが変わらない切り替えでも届く。
- シーズンも変わる切り替え（例：撤退フェイズ → 秋のメインフェイズ）では、`season` イベントのあとに `phase` イベントを送る。
- 撤退フェイズ・調整フェイズのあいだは [発言 API](./games-game-uuid-messages__post.md) が `phase_not_accepting_messages` を返すので、フロントはこのイベントで書き込み画面とログビューアーを切り替える。

```
event: phase
data: {"season":"1901s","phase_kind":"retreat"}

```

### `board`：国ごとの状況が変わった

- `data`: 空のオブジェクト。フロントはこれを受けて [盤面取得 API](./games-game-uuid-board__get.md) を読み直す。
- 最新フェイズの国ごとの補給都市数・陸軍数・海軍数・即時進行の合意のどれかが変わったときに送る（即時進行の合意・撤回、フェイズの進行、盤面の編集など）。
- フェイズの進行と同時に変わった場合は、`season` / `phase` イベントのあとに送る。
- 命令の確定では送らない（他国に、確定したかどうかも見せない）。無政府状態への移行も通知のきっかけにならないため送らない。

```
event: board
data: {}

```

### 接続維持

- 15 秒ごとにコメント行 `:ping` を送る（プロキシのタイムアウト対策）。

## 公開範囲

- 受け取る人ごとに外交履歴取得 API と同じ規則で絞り込む（ほかの国同士の密談は `masked`、他人の独り言は送らない、など）。
- 卓のステータスと担当国は配信のたびに読み直す。そのため、接続中に感想戦に入った場合、それ以降のメッセージは全員に全文で配信する。
- 公開されないメッセージは送らないが、`Last-Event-ID` の起点としては進む（再接続しても重複・取りこぼしは起きない）。

## エラー（接続時）

| Status | `code`             | 条件                                                         |
| ------ | ------------------ | ------------------------------------------------------------ |
| `400`  | `invalid_request`  | `game_uuid` / `after` / `Last-Event-ID` が有効な UUID でない |
| `404`  | `not_found`        | 卓が存在しない                                               |
| `503`  | `repository_error` | データベース障害                                             |

- 接続後にデータベース障害などで配信を続けられなくなった場合は、ストリームを閉じる。クライアントは `Last-Event-ID` 付きで再接続すれば続きを受け取れる。

## 注意

- フェイズの進行（シーズンの切り替え）は、これまでどおり何らかの API リクエストを受けたときに行われる。SSE の接続を開いているだけではフェイズは進まない。
- フェイズが進んだ卓は、システムメッセージの有無にかかわらず購読者に通知され、`season` / `phase` イベントが送られる。

## curl

```bash
curl -N '{base_path}/games/{game_uuid}/messages/stream' \
  -H 'Authorization: Bearer token-1' \
  -H 'Last-Event-ID: 019715e1-b123-7abc-8def-000000000001'
```
