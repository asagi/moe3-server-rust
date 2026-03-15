use super::Power;
use super::Province;
use super::UnitId;
use serde::Deserialize;
use serde::Serialize;

/// ユニットの定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Unit {
    Army(Army),
    Fleet(Fleet),
}

/// 陸軍の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Army {
    pub id: Option<UnitId>,
    pub power: Power,
    pub province: Province,
}

/// 海軍の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fleet {
    pub id: Option<UnitId>,
    pub power: Power,
    pub province: Province,
}

/// ユニットのロジック
impl Unit {
    pub fn new_army(power: Power, province: Province) -> Self {
        Self::Army(Army { id: None, power, province })
    }

    pub fn new_fleet(power: Power, province: Province) -> Self {
        Self::Fleet(Fleet { id: None, power, province })
    }

    pub fn id(&self) -> Option<UnitId> {
        match self {
            Unit::Army(u) => u.id,
            Unit::Fleet(u) => u.id,
        }
    }

    pub fn location(&self) -> Province {
        match self {
            Unit::Army(u) => u.province,
            Unit::Fleet(u) => u.province,
        }
    }

    pub fn power(&self) -> Power {
        match self {
            Unit::Army(u) => u.power,
            Unit::Fleet(u) => u.power,
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            Unit::Army(_) => "A",
            Unit::Fleet(_) => "F",
        }
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.symbol(), self.location())
    }

    /// 陸軍かどうか判定
    pub fn is_army(&self) -> bool {
        matches!(self, Unit::Army(_))
    }

    /// 海軍かどうか判定
    pub fn is_fleet(&self) -> bool {
        matches!(self, Unit::Fleet(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(code: &str) -> Province {
        Province::from_code(code).expect("valid province code")
    }

    impl Unit {
        pub fn with_id(mut self, id: UnitId) -> Self {
            match self {
                Unit::Army(ref mut a) => a.id = Some(id),
                Unit::Fleet(ref mut f) => f.id = Some(id),
            }
            self
        }
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
