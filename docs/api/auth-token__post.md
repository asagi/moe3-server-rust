# POST /auth/token

概要:

- 現在のアクセストークンを無効化し、新しいトークンを発行する API。

HTTP:

- メソッド: `POST`
- パス: `/auth/token`

ヘッダ:

- `Authorization`: `Bearer <access_token>` （必須）

リクエストボディ:

- なし

成功レスポンス:

- ステータス: `200 OK`
- ボディ (application/json):

```json
{
  "access_token": "3e7ab2cd-0f21-4b83-9c1d-2e5fa7d8e041"
}
```

エラーと API エラー形式:

- すべてのエラーは `application/json` で次の形式を返す:

```json
{
  "code": "invalid_request | unauthorized | repository_error",
  "message": "人間向けの説明メッセージ"
}
```

- 代表的なケース:
  - 400 Bad Request
    - 条件: `Authorization` ヘッダが欠落、または形式が不正（`Bearer <token>` でない）。
    - `code`: `invalid_request`
    - `message` 例: `authorization header is required`

  - 401 Unauthorized
    - 条件: 指定されたトークンがDBに存在しない。
    - `code`: `unauthorized`

  - 503 Service Unavailable
    - 条件: リポジトリが一時的に利用不可。
    - `code`: `repository_error`

  - 500 Internal Server Error
    - 条件: 上記以外の予期せぬエラー。

例: curl:

```bash
curl -X POST '{base_path}/auth/token' \
  -H 'Authorization: Bearer <access_token>'
```

互換性 / 注意点:

- 旧トークンはこのリクエストが成功した時点で即座に無効になる。
- 新トークンは UUID v4 で生成される。
- フロントエンドは受け取った新トークンで保存済みトークンを上書きすること。

実装ノート:

- API 層は `AuthResetTokenRequest` を `handle_auth_reset_token` に渡し、`AuthService::reset_token` を実行する。
- エラーは API 層で HTTP ステータスと `code` にマッピングして返却する。
