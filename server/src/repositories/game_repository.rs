// ============================================================================
// imports
// ============================================================================

use chrono::NaiveDateTime;
use uuid::Uuid;

use super::Game;
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
}
