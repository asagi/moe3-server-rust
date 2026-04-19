use super::Regulation;

/// 卓の定義
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Game {
    pub(crate) game_number: Option<i32>,
    pub(crate) regulation: Regulation,
    pub(crate) players: [i32; 7],
}
