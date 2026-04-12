use super::order::BuildOrder;
use super::order::ConvoyOrder;
use super::order::DisbandOrder;
use super::order::HoldOrder;
use super::order::MoveOrder;
use super::order::Order;
use super::order::OrderKind;
use super::order::OrderStatus;
use super::order::RetreatOrder;
use super::order::SupportOrder;
use super::power::Power;
use super::province::Province;
use serde::Deserialize;
use serde::Serialize;

/// ユニットの定義
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    pub power: Power,
    pub province: Province,
    pub kind: UnitKind,
    pub dislodged_from: Option<Province>,
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
            dislodged_from: None,
        }
    }

    pub fn new_fleet(power: Power, province: Province) -> Self {
        Self {
            power,
            province,
            kind: UnitKind::Fleet(Fleet {}),
            dislodged_from: None,
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
        format!("{} {}", self.symbol(), self.location().short_name())
    }

    /// 陸軍かどうか判定
    pub fn is_army(&self) -> bool {
        matches!(self.kind, UnitKind::Army(_))
    }

    /// 海軍かどうか判定
    pub fn is_fleet(&self) -> bool {
        matches!(self.kind, UnitKind::Fleet(_))
    }

    /// 維持命令を生成
    pub fn hold(&self) -> Order {
        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Hold(HoldOrder {}),
        }
    }

    /// 移動命令を生成
    pub fn move_to(&self, dest: Province) -> Order {
        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Move(MoveOrder { dest, via_convoy: false }),
        }
    }

    /// 維持サポート命令を生成
    pub fn support_hold(&self, target_unit: Unit) -> Order {
        self.support(target_unit, None)
    }

    /// 移動サポート命令を生成
    pub fn support_move(&self, target_unit: Unit, target_dest: Province) -> Order {
        self.support(target_unit, Some(target_dest))
    }

    /// サポート命令を生成
    fn support(&self, target_unit: Unit, target_dest: Option<Province>) -> Order {
        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Support(SupportOrder {
                target_unit,
                target_dest,
            }),
        }
    }

    /// 輸送命令を生成
    pub fn convoy(&self, target_unit: Unit, target_dest: Province) -> Order {
        if !self.is_fleet() {
            panic!("Only fleets can convoy");
        }

        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Convoy(ConvoyOrder {
                target_unit,
                target_dest,
            }),
        }
    }

    /// 撤退命令を生成
    pub fn retreat_to(&self, dest: Province) -> Order {
        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: self.dislodged_from,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Retreat(RetreatOrder { dest }),
        }
    }

    /// 建造命令を生成
    pub fn build(&self) -> Order {
        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Build(BuildOrder {}),
        }
    }

    /// 解体命令を生成
    pub fn disband(&self) -> Order {
        Order {
            id: None,
            power: self.power,
            unit: *self,
            dislodged_from: self.dislodged_from,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Disband(DisbandOrder {}),
        }
    }

    /// ユニットがどこから撃退されたかを設定
    pub fn dislodged_from(&mut self, province: Province) -> Self {
        self.dislodged_from = Some(province);
        *self
    }

    pub fn dislodged_via_convoy(&mut self) -> Self {
        self.dislodged_from = None;
        *self
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
        assert_eq!(army.label(), "A Par");

        let fleet = Unit::new_fleet(Power::England, p("lon"));
        assert_eq!(fleet.symbol(), "F");
        assert_eq!(fleet.label(), "F Lon");
    }
}
