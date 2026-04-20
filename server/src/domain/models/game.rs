#![allow(dead_code)] // TODO: 後で削除する
// ============================================================================
// imports
// ============================================================================

// external crates
use serde::Deserialize;
use serde::Serialize;
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
    pub(crate) status: GameStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GameStatus {
    Preparing,
    Ready,
    InProgress,
    Draw,
    Solo,
    ClosedOnDraw,
    ClosedOnSolo,
}
