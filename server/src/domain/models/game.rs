#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use uuid::Uuid;

// structs
use super::Phase;
use super::Player;
use super::Regulation;

// ============================================================================
// definitions
// ============================================================================

/// 卓の定義
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Game {
    pub uuid: Uuid,
    pub(crate) game_number: Option<i32>,
    pub(crate) regulation: Regulation,
    pub(crate) players: Vec<Player>,
    pub(crate) phases: Vec<Phase>,
}
