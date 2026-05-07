// ============================================================================
// imports
// ============================================================================

use chrono::NaiveDateTime;
use uuid::Uuid;

use super::Game;
use super::GameStatus;
use super::Power;
use super::Regulation;
use super::RepositoryError;

// ============================================================================
// definitions
// ============================================================================

///
/// 新規卓生成用の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewGame {
    pub game: Game,
}

///
/// 卓一覧取得のステータスフィルタの列挙体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameStatusFilter {
    /// Aborted・Closed 以外のすべてのステータス
    Active,
    Closed,
    Aborted,
}

///
/// 卓一覧取得用の軽量サマリ構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GameSummary {
    pub uuid: Uuid,
    pub game_number: Option<i32>,
    pub status: GameStatus,
    pub next_update_at: Option<NaiveDateTime>,
    pub regulation: Regulation,
    pub player_count: u64,
    pub season_label: Option<String>,
}

///
/// 卓リポジトリのトレイト
///
pub(crate) trait GameRepository {
    fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError>;

    #[cfg_attr(not(test), allow(dead_code))]
    fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError>;

    fn find_by_uuid(&self, game_uuid: Uuid) -> Result<Option<Game>, RepositoryError>;

    fn find_progress_candidates(&self, now: NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError>;

    /// ユーザーが Finished / Closed / Aborted 以外のステータスの卓に参加中かどうかを返す。
    fn exists_active_game_for_user(&self, user_uuid: Uuid) -> Result<bool, RepositoryError>;

    fn update(&self, game: &Game) -> Result<(), RepositoryError>;

    /// 新規プレイヤーを卓に追加する（DB永続化）
    fn add_player(&self, game_uuid: Uuid, user_uuid: Uuid, requested_power: Option<Power>) -> Result<(), RepositoryError>;

    /// 指定した卓に卓番号をアトミックに採番・割り当てる（排他制御付き）。
    /// すでに卓番号が割り当てられている場合はそのまま返す。
    fn assign_game_number(&self, game_uuid: Uuid) -> Result<i32, RepositoryError>;

    /// ステータスフィルタでページネーションして卓サマリ一覧と総件数を返す。
    fn find_paginated_by_status(
        &self,
        filter: GameStatusFilter,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<GameSummary>, u64), RepositoryError>;

    /// 指定ユーザーが参加している卓サマリ一覧と総件数をページネーションして返す。
    fn find_paginated_by_user_uuid(
        &self,
        user_uuid: Uuid,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<GameSummary>, u64), RepositoryError>;
}
