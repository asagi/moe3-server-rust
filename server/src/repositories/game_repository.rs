#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

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
    fn update(&self, game: &Game) -> Result<(), RepositoryError>;
}
