use super::super::PlayerId;
use super::super::TableId;
use super::power::Power;
use super::user::User;
use serde::Deserialize;
use serde::Serialize;

/// プレイヤーの定義
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: Option<PlayerId>,      // プレイヤー ID
    pub user: User,                // ユーザ ID
    pub table_id: Option<TableId>, // 対戦テーブル ID
    pub power: Option<Power>,      // 担当している国
    pub is_accepting_draw: bool,   // 停戦合意フラグ
}
