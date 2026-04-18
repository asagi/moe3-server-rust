// models
use super::Power;
use super::User;

// external crates
use serde::Deserialize;
use serde::Serialize;

/// プレイヤーの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) struct Player {
    user: User,
    game_number: i32,
    power: Option<Power>,
    is_accepting_draw: bool,
}
