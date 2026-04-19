// models
use super::Power;

// external crates
use uuid::Uuid;

/// プレイヤーの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Player {
    pub(crate) user_uuid: Uuid,
    pub(crate) game_number: i32,
    pub(crate) power: Option<Power>,
    pub(crate) is_accepting_draw: bool,
}
