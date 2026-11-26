# 命令解決履歴取得 API の仕様変更

- レスポンスに territories（占領地情報）を追加する。
- レスポンスの units に地域の内部コード（スネークケース）を追加する。
- 検討： リクエストパラメータの game_uuid と season もレスポンスに加えた方が良いかもしれない。

```json
{
  "main_phase": {
    // 最新 phase は直前 phase のユニット配置情報のリスト
    "units": [
      { "power": "e", "kind": "f", "location": "nth" },
      { "power": "e", "kind": "f", "location": "lon" }
    ],
    // 最新 phase は直前 phase の占領地情報のリスト
    "territories": [
      { "power": "a", "code": "vie" },
      { "power": "e", "code": "lon" },
      { "power": "e", "code": "nwy" },
      { "power": "f", "code": "par" },
      { "power": "f", "code": "mun" },
      { "power": "f", "code": "bur" },
      { "power": "g", "code": "ber" },
      { "power": "g", "code": "den" },
      { "power": "g", "code": "boh" }
    ],
    // 最新 phase は未解決命令のリスト
    "orders": [
      {
        "power": "e",
        "unit_kind": "a",
        "location": "Lon",
        "order_kind": "hold",
        "status": "unresolved"
      },
      {
        "power": "f",
        "unit_kind": "a",
        "location": "Par",
        "order_kind": "move",
        "status": "unresolved",
        "dest": "Bur",
        "via_convoy": false
      }
    ]
  },
  "retreat_phase": null, // 未来の phase は null
  "adjustment_phase": null // 未来または存在しない phase は null
}
```
