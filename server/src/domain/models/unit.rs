// models
use super::BuildOrder;
use super::ConvoyOrder;
use super::DisbandOrder;
use super::HoldOrder;
use super::MoveOrder;
use super::Order;
use super::Power;
use super::Province;
use super::RetreatOrder;
use super::SupportOrder;

// enums
use super::OrderKind;
use super::OrderStatus;

/// ユニットの定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Unit {
    pub(crate) power: Power,
    pub(crate) location: Province,
    pub(crate) kind: UnitKind,
    pub(crate) dislodged_from: Option<Province>,
    pub(crate) dislodged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnitKind {
    Army(Army),
    Fleet(Fleet),
}

/// 陸軍の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Army {}

/// 海軍の定義
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fleet {}

/// ユニットのロジック
impl Unit {
    pub(crate) fn new_army(power: Power, province: Province) -> Self {
        Self {
            power,
            location: province,
            kind: UnitKind::Army(Army {}),
            dislodged_from: None,
            dislodged: false,
        }
    }

    pub(crate) fn new_fleet(power: Power, province: Province) -> Self {
        Self {
            power,
            location: province,
            kind: UnitKind::Fleet(Fleet {}),
            dislodged_from: None,
            dislodged: false,
        }
    }

    /// ユニットの所在地を返す
    pub(crate) fn location(&self) -> Province {
        self.location
    }

    /// ユニットの所属国を返す
    pub(crate) fn power(&self) -> Power {
        self.power
    }

    /// ユニットのシンボルを返す
    pub(crate) fn symbol(&self) -> &str {
        match self.kind {
            UnitKind::Army(_) => "A",
            UnitKind::Fleet(_) => "F",
        }
    }

    /// ユニットのラベルを返す
    pub(crate) fn label(&self) -> String {
        format!("{} {}", self.symbol(), self.location().short_name())
    }

    /// ユニットの種別を返す
    pub(crate) fn kind(&self) -> UnitKind {
        self.kind
    }

    /// ユニットが撃退されたかどうかを返す
    #[allow(dead_code)]
    pub(crate) fn is_dislodged(&self) -> bool {
        self.dislodged
    }

    /// ユニットがどこから撃退されたかを返す
    pub(crate) fn dislodged_from(&self) -> Option<Province> {
        self.dislodged_from
    }

    /// 陸軍かどうか判定
    #[allow(dead_code)]
    pub(crate) fn is_army(&self) -> bool {
        matches!(self.kind, UnitKind::Army(_))
    }

    /// 海軍かどうか判定
    pub(crate) fn is_fleet(&self) -> bool {
        matches!(self.kind, UnitKind::Fleet(_))
    }

    /// 維持命令を生成
    pub(crate) fn hold(&self) -> Order {
        Order {
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Hold(HoldOrder {}),
        }
    }

    /// 移動命令を生成
    #[allow(dead_code)]
    pub(crate) fn move_to(&self, dest: Province) -> Order {
        Order {
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Move(MoveOrder { dest, via_convoy: false }),
        }
    }

    /// 維持サポート命令を生成
    #[allow(dead_code)]
    pub(crate) fn support_hold(&self, target_unit: Unit) -> Order {
        self.support(target_unit, None)
    }

    /// 移動サポート命令を生成
    #[allow(dead_code)]
    pub(crate) fn support_move(&self, target_unit: Unit, target_dest: Province) -> Order {
        self.support(target_unit, Some(target_dest))
    }

    /// サポート命令を生成
    pub(crate) fn support(&self, target_unit: Unit, target_dest: Option<Province>) -> Order {
        Order {
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
    #[allow(dead_code)]
    pub(crate) fn convoy(&self, target_unit: Unit, target_dest: Province) -> Order {
        if !self.is_fleet() {
            panic!("Only fleets can convoy");
        }

        Order {
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
    #[allow(dead_code)]
    pub(crate) fn retreat_to(&self, dest: Province) -> Order {
        Order {
            power: self.power,
            unit: *self,
            dislodged_from: self.dislodged_from,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Retreat(RetreatOrder { dest }),
        }
    }

    /// 建造命令を生成
    #[allow(dead_code)]
    pub(crate) fn build(&self) -> Order {
        Order {
            power: self.power,
            unit: *self,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Build(BuildOrder {}),
        }
    }

    /// 解体命令を生成
    pub(crate) fn disband(&self) -> Order {
        Order {
            power: self.power,
            unit: *self,
            dislodged_from: self.dislodged_from,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Disband(DisbandOrder {}),
        }
    }

    /// ユニットがどこから撃退されたかを設定
    pub(crate) fn set_dislodged_from(&mut self, province: Option<Province>) -> Self {
        self.dislodged_from = province;
        self.dislodged = true;
        *self
    }

    pub(crate) fn set_dislodged_via_convoy(&mut self) -> Self {
        self.dislodged_from = None;
        self.dislodged = true;
        *self
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::tests::a;
    use crate::domain::tests::f;

    #[test]
    fn test_unit_creation() {
        let army = a("f", "par");
        assert_eq!(army.symbol(), "A");
        assert_eq!(army.label(), "A Par");

        let fleet = f("e", "lon");
        assert_eq!(fleet.symbol(), "F");
        assert_eq!(fleet.label(), "F Lon");
    }
}
