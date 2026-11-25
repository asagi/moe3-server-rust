# Ghost 発言（滅亡した国の発言）の仕様変更の依頼

滅亡した国の発言の扱いを、次の仕様に変える。フロント（moe3-front-rr）は対応済みで、サーバー側の変更を待っている。

項目ごとに「必須」「任意」を付けた。フィールド名は案なので、サーバー側の命名規則に合わせて変えてよい。

---

## 仕様

- 滅亡した国は、通常の発言（`public`）の代わりに Ghost 発言（`ghost`）を使う。
- Ghost 発言はオープンチャンネルに、通常の発言と同じように**全員に**表示する。フロントは、発言者のアイコンを白黒の画像にし、吹き出しを薄いグレーにして区別する。
- 滅亡した国は**密談（`confidential`）を書けない**。**独り言（`personal`）は書ける**。
- 「滅亡」の判定は、今の `is_power_eliminated`（最新フェイズにその国のユニットがない）のままでよい。

---

## 1. 発言 API（`POST /games/:game_uuid/messages`）【必須】

今は、滅亡した国の発言をすべて `power_eliminated`（403）で弾いている。これを次のように変える。

| リクエストの `kind` | 滅亡していない国   | 滅亡した国                                 |
| ------------------- | ------------------ | ------------------------------------------ |
| `public`            | `public` で保存    | **`ghost` で保存**（送信者は担当国のまま） |
| `confidential`      | `confidential`     | `power_eliminated`（403）のまま            |
| `personal`          | `personal`         | **`personal` で保存（書けるようにする）**  |

- フロントは、滅亡した国でも `kind: "public"` を送る。ghost にするかどうかはサーバーが判定する（シーズンの更新直後に滅亡が決まっても、取りこぼさないように）。
- リクエストで `kind: "ghost"` を直接指定するのは、今と同じく `invalid_kind`（400）でよい。
- レスポンスで返すメッセージは、保存した kind（`ghost`）にする。
- 管理人発言（`as_admin: true`）と、開戦前夜・感想戦のユーザーとしての発言は関係しない（滅亡の判定は進行中だけ）。

## 2. ログ取得 API と SSE の公開範囲【必須】

`apply_log_access_control` で、今は ghost を**滅亡した国にだけ**返している。これを **`public` と同じく全員**（第三者・ログインしていない人を含む）に返すように変える。

- `GET /games/:game_uuid/logs/:season`
- `GET /games/:game_uuid/messages/stream`（同じ関数を使っているので、1 か所直せば両方変わるはず）

進行中でない卓（感想戦や終了後）は、今もすべて返しているので変更はない。

## 3. 卓情報 API に滅亡したかどうかを追加【必須】

フロントは、滅亡した国の参加者に対して次のことをしたい。そのために、各参加者が滅亡しているかを知りたい。

- 発言フォームのアイコンを白黒にする。
- 密談の入力欄を、書き込めない理由付きで無効にする。

`GET /games/:game_uuid` の `players[]` に次の項目を追加してほしい。

```json
{
  "user": { "...": "..." },
  "is_owner": false,
  "requested_power": null,
  "power": "i",
  "is_eliminated": true
}
```

| フィールド      | 型        | 説明                                                                                                                              |
| --------------- | --------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `is_eliminated` | `boolean` | 担当国が滅亡しているか（`is_power_eliminated` と同じ判定）。担当国がない・開戦前夜・感想戦（最新フェイズが Ready / Debrief）は `false` |

- フロントは、この項目がまだ届かない間は `false` として扱っている。

## 4. テスト

次のテストの期待値が変わるので、書き換えてほしい。

- `access_control_in_progress_unauthenticated_ghost_hidden`
- `access_control_in_progress_participant_ghost_hidden_when_not_eliminated`
- 滅亡した国の発言が `power_eliminated` になるテスト

あわせて、次のテストを追加してほしい。

- 滅亡した国の `public` が `ghost` で保存される。
- 滅亡した国の `personal` は保存できる。
- 滅亡した国の `confidential` は `power_eliminated` になる。
- 第三者のログ取得と SSE に ghost が含まれる。
