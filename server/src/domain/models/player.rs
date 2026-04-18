// models
use super::Power;
use super::User;

/// プレイヤーの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Player {
    pub(crate) user: User,
    pub(crate) game_number: i32,
    pub(crate) power: Option<Power>,
    pub(crate) is_accepting_draw: bool,
}
