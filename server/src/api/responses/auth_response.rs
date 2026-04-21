// ============================================================================
// imports
// ============================================================================

use serde::Serialize;
use uuid::Uuid;

use super::LoginUser;

// ============================================================================
// definitions
// ============================================================================

///
/// ログインレスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuthLoginResponse {
    pub access_token: String,
    pub user: AuthLoginResponseUser,
}

///
/// ログインレスポンスのユーザー構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuthLoginResponseUser {
    pub uuid: Uuid,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

/// ログインレスポンスのユーザー構造体の実装
impl From<LoginUser> for AuthLoginResponseUser {
    fn from(user: LoginUser) -> Self {
        Self {
            uuid: user.uuid,
            discord_user_id: user.discord_user_id,
            username: user.username,
            global_name: user.global_name,
            avatar_hash: user.avatar_hash,
            avatar_url: user.avatar_url,
        }
    }
}
