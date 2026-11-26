// ============================================================================
// imports
// ============================================================================

use uuid::Uuid;

use super::Power;

// ============================================================================
// definitions
// ============================================================================

///
/// プレイヤーの構造体
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Player {
    pub(crate) user_uuid: Uuid,
    pub(crate) power: Option<Power>,
    pub(crate) is_accepting_draw: bool,
    pub(crate) progress_consented: bool,
    /// 無政府化を検出済みか（復帰時にシステムメッセージを出すための印）
    pub(crate) is_idle: bool,
    pub(crate) is_owner: bool,
    pub(crate) requested_power: Option<Power>,
}
