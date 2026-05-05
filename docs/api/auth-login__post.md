# POST /auth/login

概要:

- Discord アクセストークンを受け取り、ユーザーをログインさせる API。
- 初回ログイン時はユーザーを新規作成し、既存ユーザーはプロフィールを最新化して返す。

HTTP:

- メソッド: `POST`
- パス: `/auth/login`
- コンテンツ型: `application/json`

ヘッダ:

- `Content-Type`: `application/json`

リクエストボディ（JSON スキーマ）:

- 型: object
- プロパティ:
  - `discord_access_token` (string, 必須) - Discord のアクセストークン。
- バリデーション:
  - 空文字または空白のみは不正（400）。

- 例:

```json
{
  "discord_access_token": "discord-access-token"
}
```

成功レスポンス:

- ステータス: `200 OK`
- ボディ (application/json):

```json
{
  "access_token": "2f5fa1cf-f106-49a4-b892-1f6af8ca7f59",
  "user": {
    "uuid": "0195a2f0-f66e-7df6-bd8f-d7028c6b7f45",
    "discord_user_id": "123456789012345678",
    "username": "alice",
    "global_name": "Alice",
    "avatar_hash": "a_0123456789abcdef",
    "avatar_url": "https://cdn.discordapp.com/avatars/123456789012345678/a_0123456789abcdef.png"
  }
}
```

エラーと API エラー形式:

- すべてエラーは `application/json` で次の形式を返す:

```json
{
  "code": "invalid_request | unauthorized | discord_unavailable | repository_error",
  "message": "人間向けの説明メッセージ"
}
```

- 代表的なケース:
  - 400 Bad Request
    - 条件: `discord_access_token` が欠落/空文字/空白のみ。
    - `code`: `invalid_request`
    - `message` 例: `discord_access_token is required`

  - 401 Unauthorized
    - 条件: Discord API がトークンを無効と判断。
    - `code`: `unauthorized`

  - 502 Bad Gateway
    - 条件: Discord API 呼び出し失敗、または Discord 側非成功ステータス（401 以外）。
    - `code`: `discord_unavailable`

  - 409 Conflict
    - 条件: 同時ログイン等でユーザー insert が競合。
    - `code`: `repository_error`

  - 503 Service Unavailable
    - 条件: リポジトリが一時的に利用不可。
    - `code`: `repository_error`

  - 500 Internal Server Error
    - 条件: 上記以外の予期せぬエラー。
    - `code`: `repository_error` など

例: curl:

```bash
curl -X POST '{base_path}/auth/login' \
  -H 'Content-Type: application/json' \
  -d '{"discord_access_token":"discord-access-token"}'
```

互換性 / 注意点:

- 既存ユーザーの場合、`access_token` は既存値を返し、Discord プロフィール情報のみ更新される。
- 新規ユーザーの場合、`access_token` はサーバ側で UUID v4 により発行される。
- `discord_access_token` は前後空白をトリムして処理される。

実装ノート:

- API 層は `AuthLoginRequest` を `handle_auth_login` に渡し、`AuthService::login` を実行する。
- Discord ユーザー情報取得は `/users/@me` を利用する。
- エラーは API 層で HTTP ステータスと `code` にマッピングして返却する。
