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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewGame {
    pub game: Game,
}

pub(crate) trait GameRepository {
    fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError>;

    #[cfg_attr(not(test), allow(dead_code))]
    fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError>;

    fn find_by_uuid(&self, game_uuid: Uuid) -> Result<Option<Game>, RepositoryError>;

    fn find_progress_candidates(&self, now: NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError>;

    fn update(&self, game: &Game) -> Result<(), RepositoryError>;
}
