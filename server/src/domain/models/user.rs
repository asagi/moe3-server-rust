// type aliases
use super::UserId;

// external crates
use serde::Deserialize;
use serde::Serialize;

/// ユーザの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) struct User {
    id: Option<UserId>,
}
