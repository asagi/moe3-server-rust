// models
use super::Power;
use super::User;

/// プレイヤーの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Player {
    user: User,
    game_number: i32,
    power: Option<Power>,
    is_accepting_draw: bool,
}
