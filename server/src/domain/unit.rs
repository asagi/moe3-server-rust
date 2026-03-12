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
    pub fn id(&self) -> Option<UnitId> {
        match self {
            Unit::Army(u) => u.id,
            Unit::Fleet(u) => u.id,
        }
    }

    pub fn province(&self) -> Province {
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
        format!("{} {}", self.symbol(), self.province())
    }
}

/// 陸軍の実装
impl Army {
    pub fn new(power: Power, province: Province) -> Unit {
        Unit::Army(Army { id: None, power, province })
    }
}

/// 海軍の実装
impl Fleet {
    pub fn new(power: Power, province: Province) -> Unit {
        Unit::Fleet(Fleet { id: None, power, province })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let army = Army::new(Power::France, Province::Par);
        assert_eq!(army.symbol(), "A");
        assert_eq!(army.label(), "A par");

        let fleet = Fleet::new(Power::England, Province::Lon);
        assert_eq!(fleet.symbol(), "F");
        assert_eq!(fleet.label(), "F lon");
    }
}
