// models
use super::Phase;
use super::Player;
use super::Regulation;

// external crates
use uuid::Uuid;

/// 卓の定義
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Game {
    pub uuid: Uuid,
    pub(crate) game_number: Option<i32>,
    pub(crate) regulation: Regulation,
    pub(crate) players: Vec<Player>,
    pub(crate) phases: Vec<Phase>,
}
