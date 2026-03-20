use super::Power;
use super::Province;
use serde::Deserialize;
use serde::Serialize;

/// ユニットの定義
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    pub power: Power,
    pub province: Province,
    pub kind: UnitKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitKind {
    Army(Army),
    Fleet(Fleet),
}

/// 陸軍の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Army {}

/// 海軍の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fleet {}

/// ユニットのロジック
impl Unit {
    pub fn new_army(power: Power, province: Province) -> Self {
        Self {
            power,
            province,
            kind: UnitKind::Army(Army {}),
        }
    }

    pub fn new_fleet(power: Power, province: Province) -> Self {
        Self {
            power,
            province,
            kind: UnitKind::Fleet(Fleet {}),
        }
    }

    pub fn location(&self) -> Province {
        self.province
    }

    pub fn power(&self) -> Power {
        self.power
    }

    pub fn symbol(&self) -> &str {
        match self.kind {
            UnitKind::Army(_) => "A",
            UnitKind::Fleet(_) => "F",
        }
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.symbol(), self.location())
    }

    /// 陸軍かどうか判定
    pub fn is_army(&self) -> bool {
        matches!(self.kind, UnitKind::Army(_))
    }

    /// 海軍かどうか判定
    pub fn is_fleet(&self) -> bool {
        matches!(self.kind, UnitKind::Fleet(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(code: &str) -> Province {
        Province::from_code(code).expect("valid province code")
    }

    #[test]
    fn test_unit_creation() {
        let army = Unit::new_army(Power::France, p("par"));
        assert_eq!(army.symbol(), "A");
        assert_eq!(army.label(), "A par");

        let fleet = Unit::new_fleet(Power::England, p("lon"));
        assert_eq!(fleet.symbol(), "F");
        assert_eq!(fleet.label(), "F lon");
    }
}
