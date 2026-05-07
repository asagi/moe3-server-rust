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

///
/// 即時進行合意設定レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SetProgressConsensusResponse {
    pub game_uuid: Uuid,
    pub agreed: bool,
}

///
/// 次回更新時刻変更レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SetNextUpdateAtResponse {
    pub game_uuid: Uuid,
    pub next_update_at: String,
}

///
/// 卓一覧レスポンスのレギュレーション構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct GameListItemRegulation {
    pub face_type: String,
    pub progress_mode: String,
    pub duration_type: String,
}

///
/// 卓一覧レスポンスの卓概要構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct GameListItem {
    pub game_uuid: Uuid,
    pub game_number: Option<i32>,
    pub status: String,
    pub season: Option<String>,
    pub next_update_at: Option<String>,
    pub regulation: GameListItemRegulation,
    pub player_count: u64,
}

///
/// 卓一覧レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct GetGamesResponse {
    pub games: Vec<GameListItem>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}
