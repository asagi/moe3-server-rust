# 卓一覧取得 API の実装

## エンドポイント

- GET /games

## クエリパラメータ

- ステータス
  - アクティブ（Aborted, Closed 以外）、Closed、Aborted の三種類のいずれかを指定
- ユーザー（discord_user_id）を指定
  - 指定ユーザーが参加している／した卓をすべて返却する（ステータスは問わない）
  - 卓主としての参加かどうかはレスポンスに明示
  - リクエストユーザー本人以外の discord_user_id 指定は 403 Forbidden
- ステータスと参加ユーザーは排他とする
  - ステータスもユーザーも未指定の場合はアクティブステータス指定検索として扱う
- ページ制御パラメータも必要

## 機能

- 指定された条件に合致する卓の一覧を返す。
- ステータス指定検索は認証不要
- ユーザー指定検索は認証必須

## 成功レスポンス

- Status: `200 OK`
- 各卓の概要情報の配列。
  - `game_uuid`
  - `game_number`
  - `status`
  - `season`
  - `next_update_at`： yyyy-MM-dd HH:mm 形式（JST）に変換して返却。
  - `regulation`
    - `face_type`
    - `progress_mode`
    - `duration_type`
  - `player_count`： 参加している `Player` の数。

## その他

- 未決
