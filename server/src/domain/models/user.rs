// type aliases
use super::UserId;

/// ユーザの定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct User {
    id: Option<UserId>,
}
