use super::power::Power;
use super::province::Province;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Territory {
    power: Power,
    province: Province,
}

impl Territory {
    pub fn new(power: Power, province: Province) -> Self {
        Self { power, province }
    }

    pub fn power(&self) -> &Power {
        &self.power
    }

    pub fn province(&self) -> &Province {
        &self.province
    }
}
