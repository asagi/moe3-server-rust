// ============================================================================
// imports
// ============================================================================

use serde::Serialize;
use uuid::Uuid;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓作成レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateGameResponse {
    pub game_uuid: Uuid,
    pub owner_user_uuid: Uuid,
    pub requested_power: Option<String>,
}

///
/// 卓参加レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct JoinGameResponse {
    pub game_uuid: Uuid,
    pub user_uuid: Uuid,
    pub requested_power: Option<String>,
}
