#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use chrono::NaiveDateTime;
use uuid::Uuid;

// structs
use super::Game;

// enums
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
    fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError>;
    fn find_by_uuid(&self, game_uuid: Uuid) -> Result<Option<Game>, RepositoryError>;
    fn find_progress_candidates(&self, now: NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError>;
    fn update(&self, game: &Game) -> Result<(), RepositoryError>;
}
