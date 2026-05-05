# 卓更新フロー

## 概要

すべての HTTP リクエストは、ハンドラが実行される前に **グローバルプリハンドラ**（`GlobalPreHandler::run`）を通過する。このプリハンドラが Game の自動進行の起点となる。

---

## 全体フロー

```
HTTP リクエスト受信
    │
    ▼
run_global_pre_handler（ミドルウェア）
    │
    ├─[1] last_access_at 更新（ロック外）
    │       Authorization ヘッダのアクセストークンで users テーブルの
    │       last_access_at を現在時刻に更新する。
    │       （無政府判定の誤検知を防ぐため、ロック取得前に先行実行）
    │
    ├─[2] game_update_lock 取得
    │
    ├─[3] GlobalPreHandler::run()
    │       ├─ A. 卓主無政府化チェック（mark_idle_owners_accepting_draw）
    │       └─ B. フェイズ自動進行（progress_games）
    │
    └─[4] game_update_lock 解放
    │
    ▼
各ハンドラ実行（ロック外）
    │   ※ PUT draw-proposal / PUT units / DELETE units は再度 game_update_lock を取得して実行
    ▼
レスポンス返却
```

---

## GlobalPreHandler::run の詳細

### A. 卓主無政府化チェック（`mark_idle_owners_accepting_draw`）

1. Closed・Aborted 以外の全 Game を取得（`find_all_active`）
2. 各 Game について卓主の `last_access_at` が閾値以前かどうかを確認
   - 閾値 = `now - duration_type.idle_limit_minutes()`
3. 無政府状態かつ `is_accepting_draw = false` の場合のみ `is_accepting_draw = true` に更新し DB に反映（冪等）
4. 今回新たにフラグを立てた卓のうち、現在フェイズがメインフェイズ（春命令・秋命令）のものについて `OwnerAbsent` メッセージを追記する

### B. フェイズ自動進行（`progress_games`）

1. **進行候補取得**（`find_progress_candidates`）
   - `status NOT IN ('closed', 'aborted')` かつ `next_update IS NOT NULL` かつ `next_update <= now`
   - `next_update` の昇順（古い順）で取得

2. **卓ごとに `progress_game` を実行**

---

## progress_game の詳細

```
game = find_by_uuid(game_uuid)
    │
    ├─ next_update_at が None または未来 → スキップ
    ├─ status が Closed または Aborted    → スキップ
    │
    ▼
latest_phase = game.phases.pop()
    │
    ├─ Ready フェイズかつ参加プレイヤーが 7 人未満
    │       → status = Aborted, next_update_at = None
    │       → game_repository.update()
    │       → [Aborted メッセージ追記]
    │
    ▼
PhaseContext 生成
    │
    ├─ 無政府国の除外（remove_idle_powers）
    │       各プレイヤーの last_access_at を確認し、閾値以前なら
    │       context.remove_power() で活性国リストから除外
    │
    ├─ 卓主 is_accepting_draw かつメインフェイズ → context.set_draw()
    │
    ▼
latest_phase.close(context)  ← フェイズ締め切り・次フェイズ生成
    │
    ▼
PhaseContext の結果を Game に反映
    ├─ 生成されたフェイズ列を game.phases に追加
    ├─ game.is_draw / game.is_solo 更新
    ├─ game.status 更新（InProgress / Finished / Closed）
    └─ game.next_update_at を計算・更新
    │
    ▼
game_repository.update(game)
    │
    ▼
メッセージ追記（変化に応じて）
    ├─ StartSeason … 新フェイズがメインフェイズになった場合
    │       └─ 卓主が無政府状態なら続けて OwnerAbsent も追記
    ├─ Solo     … 制覇終了（Finished かつ is_solo）
    ├─ Draw     … 和平終了（Finished かつ is_draw）
    └─ Closed   … Debrief フェイズ終了によるクローズ
```

---

## Phase.close の概要

`Phase::close(context)` は以下を順に実行する。

1. **補給都市ゼロ国の除外** — `count_supply_centers == 0` の国を `context.remove_power` する
2. **和平判定** — `context.is_draw()` かつメインフェイズなら Draw 終了シーケンス（Retreat → Debrief を生成）
3. **命令解決** — フェイズ種別に応じたアジュディケーター（MainAdjudicator / RetreatAdjudicator / AdjustmentAdjudicator）を呼び出す
4. **占領処理** — ユニットの位置から領土を更新
5. **制覇判定** — 補給都市 18 以上の国があれば Solo 終了シーケンス（Retreat → Adjustment → Debrief を生成）
6. **次フェイズ生成** — フェイズ種別ごとに次フェイズを決定し、スキップ可能なフェイズ（撤退ユニット不在の退却フェイズなど）は再帰的に `close` する

### フェイズ遷移

```
Ready
  └─→ SpringMain（1901年）
        ├─→ SpringRetreat（撤退ユニットがいる場合）
        │       └─→ FallMain
        └─→ FallMain（スキップ）
                ├─→ FallRetreat（撤退ユニットがいる場合）
                │       └─→ Adjustment（ユニット過不足がある国がいる場合）
                │               └─→ SpringMain（次年）
                └─→ Adjustment / SpringMain（スキップ組み合わせ）

Solo/Draw 確定時:
  メインフェイズ または 退却フェイズ
    └─→ [Retreat →] Debrief

Debrief.close:
  └─→ status = Closed, next_update_at = None
```

---

## DB 更新のタイミング

| 操作 | 更新箇所 |
|---|---|
| 卓主無政府化フラグ立て | `games` テーブルの `game_players.is_accepting_draw` |
| フェイズ自動進行 | `games` / `game_phases` / `game_phase_orders` テーブル（`game_repository.update`） |
| プレイヤー操作（draw-proposal / units） | `game_repository.update` 経由で同上 |

`game_repository.update` は `games` テーブルのメタ情報・`game_players` の `is_accepting_draw` を更新し、フェイズ差分（追加分）を `game_phases` / `game_phase_orders` に INSERT する。

---

## 関連ファイル

| 役割 | ファイル |
|---|---|
| ミドルウェア（プリハンドラ呼び出し） | `server/src/api/router.rs` — `run_global_pre_handler` |
| グローバルプリハンドラ | `server/src/api/middleware.rs` — `GlobalPreHandler` |
| 卓進行サービス | `server/src/services/game_progression_service.rs` — `GameProgressionService` |
| フェイズ締め切りロジック | `server/src/domain/models/phases/method.rs` — `Phase::close` |
| 進行候補クエリ | `server/src/repositories/sqlite_game_repository.rs` — `find_progress_candidates` |
