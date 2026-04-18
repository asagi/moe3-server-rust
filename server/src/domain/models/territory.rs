// models
use super::Power;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Territory {
    pub(crate) power: Power,
    pub(crate) code: String,
}

impl Territory {
    /// Territory オブジェクトを生成する
    pub(crate) fn new(power: Power, code: &str) -> Self {
        Self {
            power,
            code: code.to_string(),
        }
    }

    /// 地域コードを取得する
    pub(crate) fn code_with_coast(&self) -> &str {
        &self.code
    }

    /// 地域コードを取得する（海岸線を除く）
    pub(crate) fn code(&self) -> &str {
        &self.code[..3]
    }
}
