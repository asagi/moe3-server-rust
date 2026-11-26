# 和平終了フラグ（講和の宣言）の状態を返す依頼

フロント（moe3-front-rr）の「卓の管理」ページ（タブバーの歯車）に、卓主の操作として次の 3 つを入れた。

- 次回更新時刻の延長（`PUT /admin/games/:game_uuid/next-update-at`）
- 和平終了フラグ（講和）の宣言・撤回（`PUT /admin/games/:game_uuid/draw-proposal`）
- 定時進行から合意進行への切り替え（`PUT /admin/games/:game_uuid/progress-mode`）

このうち和平終了フラグは、**今立っているかどうかを取る API がない**ので、画面に「講和を宣言しています／していません」を出せない。今は宣言・撤回の両方のボタンを出してしのいでいる。

項目ごとに「必須」「任意」を付けた。フィールド名は案なので、サーバー側の命名規則に合わせて変えてよい。

---

## 前提（フロント側で決まっている仕様）

- 3 つとも、使えるのは**卓主だけ**（管理者には広げない。今のサーバーのまま）。視点変更中は、切り替え先が卓主なら使える。
- 和平終了フラグを立てても、その場では終戦しない。**次の更新で講和が成立する**（今のサーバーの動きのまま）。それまでは撤回できる。

## 1. `GET /games/:game_uuid` に和平終了フラグを足す【必須】

`game` に `draw_proposal`（`boolean`）を足してほしい。値は、卓主の `players[].is_accepting_draw`。

```json
{
  "game": {
    "game_uuid": "…",
    "status": "in_progress",
    "phase_kind": "main",
    "draw_proposal": false,
    "...": "…"
  }
}
```

- 卓主の無政府化で自動的に立った場合（`mark_owner_accepting_draw_if_idle`）も、同じく `true` になる。
- 誰に返すか：全員に返してよい（講和の宣言・撤回はシステムメッセージで全員に知らされているので、隠す情報ではない）。
- 卓主がいない・フェイズがない卓は `false`。

## 2. 和平終了フラグの変更を SSE で知らせる【任意】

宣言・撤回のシステムメッセージ（`draw_proposed` / `draw_rescinded`）が SSE で流れるので、フロントはそれを合図に読み直せる。追加のイベントは要らない。

## 3. `draw-proposal` にもシーズンの確認を入れる【任意】

`next-update-at` と `progress-mode` は、リクエストの `season` が最新フェイズと違うと `409 phase_conflict` を返す。`draw-proposal` には `season` がないので、フェイズが進んだあとに古い画面から押すと、次のメインフェイズでフラグが立ってしまう。

- `season`（任意）を受け付け、指定があって最新フェイズと違えば `409 phase_conflict` にしてほしい。
- フロントは、対応してもらえたら `season` を付けて送る。

## 4. テスト

- `GET /games/:game_uuid` の `draw_proposal` が、宣言後は `true`、撤回後は `false` になる。
- 卓主の無政府化で立ったときも `true` になる。
- （3 をやる場合）`draw-proposal` に古い `season` を付けると `409`、付けなければ今までどおり。
