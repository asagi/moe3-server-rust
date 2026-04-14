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
    pub fn new(power: Power, code: &str) -> Self {
        Self {
            power,
            code: code.to_string(),
        }
    }

    pub fn power(&self) -> &Power {
        &self.power
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}
