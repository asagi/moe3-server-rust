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
    fn try_claim_progression(
        &self,
        game_uuid: Uuid,
        lock_id: Uuid,
        lock_until: NaiveDateTime,
        now: NaiveDateTime,
    ) -> Result<bool, RepositoryError>;
    fn update_if_claimed(&self, game: &Game, lock_id: Uuid) -> Result<bool, RepositoryError>;
    fn release_progression_claim(&self, game_uuid: Uuid, lock_id: Uuid) -> Result<(), RepositoryError>;
    fn update(&self, game: &Game) -> Result<(), RepositoryError>;
}
