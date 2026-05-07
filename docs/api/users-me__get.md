# GET /users/me

概要:

- 認証済みユーザー自身のプロフィール情報を返す API。

HTTP:

- メソッド: `GET`
- パス: `/users/me`

ヘッダ:

- `Authorization`: `Bearer <access_token>` （必須）

リクエストボディ:

- なし

成功レスポンス:

- ステータス: `200 OK`
- ボディ (application/json):

```json
{
  "discord_user_id": "123456789012345678",
  "username": "alice",
  "global_name": "Alice",
  "avatar_url": "https://cdn.discordapp.com/avatars/123456789012345678/a_0123456789abcdef.png"
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
    - 条件: 指定されたトークンが DB に存在しない。
    - `code`: `unauthorized`

  - 503 Service Unavailable
    - 条件: リポジトリが一時的に利用不可。
    - `code`: `repository_error`

  - 500 Internal Server Error
    - 条件: 上記以外の予期せぬエラー。

例: curl:

```bash
curl -X GET '{base_path}/users/me' \
  -H 'Authorization: Bearer <access_token>'
```

互換性 / 注意点:

- `global_name` は Discord 側で未設定の場合 `null` になる。
- `avatar_url` は Discord 側でアバター未設定の場合 `null` になる。

実装ノート:

- API 層は `AuthResetTokenRequest`（Authorization ヘッダ抽出・バリデーション共通）を `handle_get_users_me` に渡し、`AuthService::get_me` を実行する。
- エラーは API 層で HTTP ステータスと `code` にマッピングして返却する。
