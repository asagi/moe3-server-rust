// external crates
use serde::Deserialize;
use serde::Serialize;

/// 卓の定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) struct Game {
    number: i32,
}
