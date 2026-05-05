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

///
/// ユニット配置制御レスポンスのユニットボディ構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct UnitResponseBody {
    pub power: String,
    pub kind: String,
}

///
/// ユニット配置制御レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SetUnitResponse {
    pub game_uuid: Uuid,
    pub location: String,
    pub unit: Option<UnitResponseBody>,
}

///
/// 占領情報編集レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SetTerritoryResponse {
    pub game_uuid: Uuid,
    pub code: String,
    pub power: Option<String>,
}

///
/// 進行モード変更レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SetProgressModeResponse {
    pub game_uuid: Uuid,
    pub progress_mode: String,
}
