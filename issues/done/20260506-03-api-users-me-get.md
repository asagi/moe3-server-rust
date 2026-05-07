# 自身のユーザー情報取得 API の実装

## エンドポイント

- GET /users/me

## リクエスト

- なし（認証ヘッダーのみ）

## 機能

- 認証済みユーザー自身のプロフィール情報を返す。

## 制限

- 未認証リクエストはエラーとする。

## 成功レスポンス

- Status: `200 OK`
- `discord_user_id`
- `username`
- `global_name`
- `avatar_url`
- `access_token`

## その他

- 未決
