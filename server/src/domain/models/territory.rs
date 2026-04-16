// models
use super::Power;

// external crates
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Territory {
    power: Power,
    code: String,
}

impl Territory {
    /// Territory オブジェクトを生成する
    pub(crate) fn new(power: Power, code: &str) -> Self {
        Self {
            power,
            code: code.to_string(),
        }
    }

    /// 占領国を取得する
    pub(crate) fn power(&self) -> &Power {
        &self.power
    }

    /// 占領国を設定する
    pub(crate) fn set_power(&mut self, power: Power) {
        self.power = power;
    }

    /// 地域コードを取得する
    pub(crate) fn code(&self) -> &str {
        &self.code
    }
}
