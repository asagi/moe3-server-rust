#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use uuid::Uuid;

// structs
use super::Power;

// ============================================================================
// definitions
// ============================================================================

/// プレイヤーの定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Player {
    pub(crate) user_uuid: Uuid,
    pub(crate) power: Option<Power>,
    pub(crate) is_accepting_draw: bool,
    pub(crate) is_owner: bool,
    pub(crate) requested_power: Option<Power>,
}
