use super::super::TableId;
use serde::Deserialize;
use serde::Serialize;

/// 卓の定義
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Table {
    pub id: Option<TableId>, // 卓 ID
}
