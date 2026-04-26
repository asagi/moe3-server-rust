// ============================================================================
// imports
// ============================================================================

use super::Power;

// ============================================================================
// definitions
// ============================================================================

///
/// 地域の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Territory {
    pub(crate) power: Power,
    pub(crate) code: String,
}

/// 地域の構造体の実装
impl Territory {
    ///
    /// Territory オブジェクトを生成する
    ///
    pub(crate) fn new(power: Power, code: &str) -> Self {
        Self {
            power,
            code: code.to_string(),
        }
    }

    ///
    /// 地域コードを返却する
    ///
    pub(crate) fn code_with_coast(&self) -> &str {
        &self.code
    }

    ///
    /// 地域コードを返却する（港情報を除く）
    ///
    pub(crate) fn code(&self) -> &str {
        &self.code[..3]
    }
}
