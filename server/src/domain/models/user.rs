// ============================================================================
// imports
// ============================================================================

use uuid::Uuid;

// ============================================================================
// definitions
// ============================================================================

///
/// ユーザの定義
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct User {
    pub uuid: Uuid,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}
