# 卓一覧取得 API の修正

## 概要

- 成功時のレスポンスに、それぞれの卓のフェイズ情報（PhaseKind）を追加する。
  - `phase_kind` (`String`): "ready" | "main" | "retreat" | "adjustment" | "debrief"
