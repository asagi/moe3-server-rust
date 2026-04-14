// type aliases
use super::TableId;

// external crates
use serde::Deserialize;
use serde::Serialize;

/// 卓の定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) struct Table {
    id: Option<TableId>, // 卓 ID
}
