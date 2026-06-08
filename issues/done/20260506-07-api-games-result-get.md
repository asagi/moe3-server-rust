# 命令解決履歴取得 API の実装

## エンドポイント

- GET /games/:game_uuid/result/:season

## パスパラメータ

- `game_uuid` (`string`): 対象の卓の UUID（UUID v7 形式）
- `season` (`string`): 年 + 末尾の `s`（春）または `f`（秋）で表現するターン文字列（例: `"1901s"`, `"1902f"`）

## 機能

指定されたシーズンに属するフェイズの命令・結果とユニット配置を返す。認証不要。

### season とフェイズの対応

`season` の末尾文字によって対象フェイズ種別が変わる。

| season 末尾 | 含まれる PhaseKind |
|---|---|
| `s` | `SpringMain`, `SpringRetreat` |
| `f` | `FallMain`, `FallRetreat`, `Adjustment` |

`game.phases` から、上記 `PhaseKind` に一致し、かつ `phase.year` が `season` の年と一致するフェイズのみを取得する。

年の取り出し方は `Game::current_turn()` の実装を参照（末尾 1 文字を除いた部分を `i32` にパース）。

### フェイズの status

`game.phases.last()` が「現在フェイズ」。

- 現在フェイズ → `status: "open"`
- それ以外のすべてのフェイズ → `status: "closed"`

フロントはこのフィールドで進行中フェイズを識別するため、返却するすべてのフェイズオブジェクトに必ず含める。

### 返却対象フェイズ

- **メインフェイズ** (`SpringMain` / `FallMain`): 常に含める
- **撤退フェイズ** (`SpringRetreat` / `FallRetreat`): 対応する `PhaseKind` のフェイズが `game.phases` に存在する場合のみ含める。存在しない（撤退フェイズ未開始）場合は `null`。
- **調整フェイズ** (`Adjustment`): `season` 末尾が `f` かつ対応フェイズが存在する場合のみ含める。存在しない場合は `null`。春シーズンは常に `null`。

### 命令リスト

各フェイズの `phase.orders` から `Order::is_assumed() == true` の命令を除外したものを返す（仮想命令は含めない）。

### ユニットリスト

対象シーズンに属するフェイズのうち最後尾（インデックスが最大のもの）の `phase.units` を返す。

## 成功レスポンス

- Status: `200 OK`

レスポンスの大まかな構造（詳細なフィールド定義は doc 作成時に確定）:

```json
{
  "units": [ /* 最後尾フェイズの phase.units から生成 */ ],
  "main_phase": {
    "status": "open" | "closed",
    "orders": [ /* is_assumed() == false の Order */ ]
  },
  "retreat_phase": null | {
    "status": "open" | "closed",
    "orders": [ /* is_assumed() == false の Order */ ]
  },
  "adjustment_phase": null | {
    "status": "open" | "closed",
    "orders": [ /* is_assumed() == false の Order */ ]
  }
}
```

## エラー

- `400 Bad Request`: `game_uuid` が UUID 形式でない場合。
- `404 Not Found`:
  - 指定した卓が存在しない場合。
  - `season` が `"ready"` または `"debrief"` の場合。
  - `season` がゲームに存在しない場合（到達していないシーズン、不正な文字列を含む）。

## 実装方針

`get_games_logs` / `handle_get_games_logs` と同様のパターンで実装する。

- Axum ハンドラ `get_games_result`: `game_uuid` の UUID パースを行い、失敗なら `400` を返す。
- 純粋関数 `handle_get_games_result`: ゲーム取得 → シーズン存在確認 → フェイズ抽出 → レスポンス組み立て。
- シーズンの存在確認は `build_seasons` で得られるシーズンリストとの照合を使用する（`handle_get_games_logs` と同じ実装）。

ルーターへの追加:

```rust
.route("/games/:game_uuid/result/:season", get(get_games_result::<U, G, D>))
```
