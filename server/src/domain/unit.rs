use serde::Deserialize;
use serde::Serialize;

use super::UnitId;
use super::power::Power;
use super::province::Province;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    Army,
    Fleet,
}

impl UnitType {
    /// ユニットの種類に応じた記号を返却する
    pub fn symbol(&self) -> &str {
        match self {
            UnitType::Army => "A",
            UnitType::Fleet => "F",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    pub id: Option<UnitId>,
    pub power: Power,
    pub unit_type: UnitType,
    pub location: Province,
}

impl Unit {
    pub fn new(power: Power, unit_type: UnitType, location: Province) -> Self {
        Self {
            id: None,
            power,
            unit_type,
            location,
        }
    }

    /// ユニットの表示用ラベル（例: "A Par"）を取得する
    pub fn label(&self) -> String {
        format!("{} {}", self.unit_type.symbol(), self.location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_type_symbol() {
        assert_eq!(UnitType::Army.symbol(), "A");
        assert_eq!(UnitType::Fleet.symbol(), "F");
    }

    #[test]
    fn test_unit_label() {
        let unit = Unit::new(Power::France, UnitType::Army, Province::Par);
        assert_eq!(unit.label(), "A par");
    }
}
