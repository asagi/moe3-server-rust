#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use serde::Serialize;
use uuid::Uuid;

// ============================================================================
// definitions
// ============================================================================

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateGameResponse {
    pub game_uuid: Uuid,
    pub owner_user_uuid: Uuid,
    pub requested_power: Option<String>,
}
