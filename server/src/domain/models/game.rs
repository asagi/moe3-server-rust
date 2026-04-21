// ============================================================================
// imports
// ============================================================================

use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

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
    pub(crate) is_canceled: bool,
    pub(crate) is_draw: bool,
    pub(crate) is_solo: bool,
    pub(crate) next_update: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GameStatus {
    Preparing,
    Ready,
    InProgress,
    Finished,
    Closed,
}
