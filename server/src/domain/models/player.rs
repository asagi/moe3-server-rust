// models
use super::Power;
use super::User;

// type aliases
use super::PlayerId;
use super::TableId;

// external crates
use serde::Deserialize;
use serde::Serialize;

/// プレイヤーの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) struct Player {
    id: Option<PlayerId>,      // プレイヤー ID
    user: User,                // ユーザ ID
    table_id: Option<TableId>, // 対戦テーブル ID
    power: Option<Power>,      // 担当している国
    is_accepting_draw: bool,   // 停戦合意フラグ
}
