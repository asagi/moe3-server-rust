use super::super::UserId;
use serde::Deserialize;
use serde::Serialize;

/// ユーザの定義
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Option<UserId>,
}
