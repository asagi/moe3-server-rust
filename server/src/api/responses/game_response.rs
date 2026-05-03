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

///
/// 和平終了フラグ設定レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SetDrawProposalResponse {
    pub game_uuid: Uuid,
    pub draw_proposal: bool,
}
