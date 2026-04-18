// models
use super::Power;
use super::Province;
use super::Unit;

// standard library
use std::fmt;

/// 命令の定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Order {
    pub(crate) power: Power,
    pub(crate) unit: Unit,
    pub(crate) dislodged_from: Option<Province>,
    pub(crate) status: OrderStatus,
    pub(crate) kind: OrderKind,
}

/// 命令の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderStatus {
    Unresolved,
    Failure,
    Success,
    Dislodged,
    Cut,
    Valid,
    Invalid,
    Unreachable,
}

/// 命令の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderKind {
    Hold(HoldOrder),
    Move(MoveOrder),
    Support(SupportOrder),
    Convoy(ConvoyOrder),
    Retreat(RetreatOrder),
    Build(BuildOrder),
    Disband(DisbandOrder),
}

/// ホールド命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HoldOrder {}

/// 移動命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MoveOrder {
    pub(crate) dest: Province,
    pub(crate) via_convoy: bool,
}

/// サポート命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SupportOrder {
    pub(crate) target_unit: Unit,
    pub(crate) target_dest: Option<Province>,
}

/// 輸送命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConvoyOrder {
    pub(crate) target_unit: Unit,
    pub(crate) target_dest: Province,
}

/// 撤退命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetreatOrder {
    pub(crate) dest: Province,
}

/// 建造命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BuildOrder {}

/// 解体命令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DisbandOrder {}

/// 命令のロジック
impl Order {
    #[allow(dead_code)]
    pub(crate) fn new_hold(power: Power, unit: Unit) -> Self {
        Order {
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Hold(HoldOrder {}),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_move(power: Power, unit: Unit, dest: Province) -> Self {
        Order {
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Move(MoveOrder { dest, via_convoy: false }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_support(power: Power, unit: Unit, target_unit: Unit, target_dest: Option<Province>) -> Self {
        Order {
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Support(SupportOrder {
                target_unit,
                target_dest,
            }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_convoy(power: Power, unit: Unit, target_unit: Unit, target_dest: Province) -> Self {
        Order {
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Convoy(ConvoyOrder {
                target_unit,
                target_dest,
            }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_retreat(power: Power, unit: Unit, dest: Province) -> Self {
        Order {
            power,
            unit,
            dislodged_from: unit.dislodged_from(),
            status: OrderStatus::Unresolved,
            kind: OrderKind::Retreat(RetreatOrder { dest }),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_build(power: Power, unit: Unit) -> Self {
        Order {
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Build(BuildOrder {}),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new_disband(power: Power, unit: Unit) -> Self {
        Order {
            power,
            unit,
            dislodged_from: unit.dislodged_from(),
            status: OrderStatus::Unresolved,
            kind: OrderKind::Disband(DisbandOrder {}),
        }
    }

    /// ユニットの現在地を返す
    pub(crate) fn location(&self) -> Province {
        self.unit.location()
    }

    /// ユニットの目的地を返す
    pub(crate) fn dest(&self) -> Province {
        match &self.kind {
            OrderKind::Move(m) => m.dest,
            OrderKind::Retreat(r) => r.dest,
            _ => unreachable!("Not move order does not have a destination"),
        }
    }

    /// ターゲットユニットを返す
    pub(crate) fn target_unit(&self) -> Unit {
        match &self.kind {
            OrderKind::Support(s) => s.target_unit,
            OrderKind::Convoy(c) => c.target_unit,
            _ => unreachable!("Only support and convoy orders have target units"),
        }
    }

    /// ターゲットの目的地を返す
    pub(crate) fn target_dest(&self) -> Option<Province> {
        match &self.kind {
            OrderKind::Support(s) => s.target_dest,
            OrderKind::Convoy(c) => Some(c.target_dest),
            _ => unreachable!("Only support and convoy orders have target destinations"),
        }
    }

    /// 移動命令で海路指定フラグが立っているかどうかを返す
    pub(crate) fn via_convoy(&self) -> bool {
        match &self.kind {
            OrderKind::Move(m) => m.via_convoy,
            _ => false,
        }
    }

    /// ターゲット命令と一致するかどうかを判定
    pub(crate) fn is_matching_target(&self, other_order: &Order) -> bool {
        match &self.kind {
            OrderKind::Support(s) => {
                if s.target_unit != other_order.unit {
                    return false;
                }
                match &other_order.kind {
                    OrderKind::Move(m) => s.target_dest == Some(m.dest),
                    _ => s.target_dest.is_none(),
                }
            }
            OrderKind::Convoy(c) => {
                if c.target_unit != other_order.unit {
                    return false;
                }
                match &other_order.kind {
                    OrderKind::Move(m) => c.target_dest == m.dest,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// ステータスを Unresolved に変更
    #[allow(dead_code)]
    pub(crate) fn set_unresolved(&mut self) -> Self {
        self.status = OrderStatus::Unresolved;
        *self
    }

    /// ステータスを Failure に変更
    pub(crate) fn set_failure(&mut self) -> Self {
        self.status = OrderStatus::Failure;
        *self
    }

    /// ステータスを Success に変更
    pub(crate) fn set_success(&mut self) -> Self {
        self.status = OrderStatus::Success;
        *self
    }

    /// ステータスを Dislodged に変更
    #[allow(dead_code)]
    pub(crate) fn set_dislodged(&mut self) -> Self {
        self.status = OrderStatus::Dislodged;
        *self
    }

    /// ステータスを Cut に変更
    pub(crate) fn set_cut(&mut self) -> Self {
        self.status = OrderStatus::Cut;
        *self
    }

    /// ステータスを Valid に変更
    pub(crate) fn set_valid(&mut self) -> Self {
        self.status = OrderStatus::Valid;
        *self
    }

    /// ステータスを Invalid に変更
    pub(crate) fn set_invalid(&mut self) -> Self {
        self.status = OrderStatus::Invalid;
        *self
    }

    /// ステータスを Unreachable に変更
    pub(crate) fn set_unreachable(&mut self) -> Self {
        self.status = OrderStatus::Unreachable;
        *self
    }

    /// ステータスが `Unresolved` かどうか
    pub(crate) fn is_unresolved(&self) -> bool {
        self.status == OrderStatus::Unresolved
    }

    /// ステータスが `Success` かどうか
    pub(crate) fn is_success(&self) -> bool {
        self.status == OrderStatus::Success
    }

    /// ステータスが `Dislodged` かどうか
    pub(crate) fn is_dislodged(&self) -> bool {
        self.status == OrderStatus::Dislodged
    }

    /// ステータスが `Cut` かどうか
    #[allow(dead_code)]
    pub(crate) fn is_cut(&self) -> bool {
        self.status == OrderStatus::Cut
    }

    /// ステータスが `Valid` かどうか
    pub(crate) fn is_valid(&self) -> bool {
        self.status == OrderStatus::Valid
    }

    /// ステータスが `Invalid` かどうか
    pub(crate) fn is_invalid(&self) -> bool {
        self.status == OrderStatus::Invalid
    }

    /// ステータスが `Failure` かどうか
    pub(crate) fn is_failure(&self) -> bool {
        self.status == OrderStatus::Failure
    }

    /// ステータスが `Unreachable` かどうか
    pub(crate) fn is_unreachable(&self) -> bool {
        self.status == OrderStatus::Unreachable
    }

    /// 命令が他の勢力のユニットに対するもの（仮定命令）であるかどうか
    pub(crate) fn is_assumed(&self) -> bool {
        self.power != self.unit.power()
    }

    pub(crate) fn set_dislodged_by(&mut self, attacker: &Order) {
        self.status = OrderStatus::Dislodged;
        self.dislodged_from = if attacker.via_convoy() {
            None
        } else {
            Some(attacker.location())
        };
        self.unit.set_dislodged_from(self.dislodged_from);
    }

    /// 命令を仮定命令に変換
    #[allow(dead_code)]
    pub fn assumed_by(&mut self, power: Power) -> Self {
        self.power = power;
        *self
    }

    /// 移動命令に海路指定フラグを設定する
    pub fn set_via_convoy(&mut self) -> Self {
        if self.unit.is_fleet() {
            unreachable!("expected Army")
        }
        if let OrderKind::Move(move_order) = &mut self.kind {
            move_order.via_convoy = true;
        } else {
            unreachable!("expected Move")
        }
        *self
    }
}

/// 命令を Diplomacy 風の短縮表記で整形する。
///
/// 例:
/// - Hold: A Lon Holds
/// - Move: A Lon - Wal
/// - Support: A Lon S A Wal - Yor
/// - Convoy: F Eng C A Lon - Bre
impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            OrderKind::Hold(_) => {
                write!(f, "{} Holds", self.unit.label())
            }
            OrderKind::Move(o) => {
                write!(f, "{} - {}", self.unit.label(), o.dest.short_name())
            }
            OrderKind::Support(o) => {
                // 自分の勢力とターゲットの勢力が違う場合、形容詞を取得
                let target_label = if self.power != o.target_unit.power() {
                    format!("{} {}", o.target_unit.power().adjective(), o.target_unit.label())
                } else {
                    o.target_unit.label()
                };

                if let Some(dest) = o.target_dest {
                    write!(f, "{} S {} - {}", self.unit.label(), target_label, dest.short_name())
                } else {
                    write!(f, "{} S {}", self.unit.label(), target_label)
                }
            }
            OrderKind::Convoy(o) => {
                let target_label = if self.power != o.target_unit.power() {
                    format!("{} {}", o.target_unit.power().adjective(), o.target_unit.label())
                } else {
                    o.target_unit.label()
                };

                write!(f, "{} C {} - {}", self.unit.label(), target_label, o.target_dest.short_name())
            }
            OrderKind::Retreat(o) => {
                write!(f, "{} - {}", self.unit.label(), o.dest.short_name())
            }
            OrderKind::Build(_) => {
                write!(f, "Build {}", self.unit.label())
            }
            OrderKind::Disband(_) => {
                write!(f, "Remove {}", self.unit.label())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tests::a;
    use crate::domain::tests::f;
    use crate::domain::tests::p;

    #[test]
    fn test_order_creation() {
        let unit = a("e", "lon");
        let order = Order::new_move(Power::England, unit, p("lon"));
        assert_eq!(order.unit, unit);
        assert_eq!(order.power, Power::England);
    }

    #[test]
    fn test_hold_creation() {
        let unit = f("e", "lon");
        let order = Order::new_hold(Power::Austria, unit);
        assert_eq!(order.unit, unit);
    }

    #[test]
    fn test_display_hold() {
        let unit = a("a", "vie");
        let order = Order::new_hold(Power::Austria, unit);

        assert_eq!(order.to_string(), "A Vie Holds");
    }

    #[test]
    fn test_display_move() {
        let unit = a("e", "lon");
        let order = Order::new_move(Power::England, unit, p("wal"));

        assert_eq!(order.to_string(), "A Lon - Wal");
    }

    #[test]
    fn test_display_support_hold() {
        let unit = a("g", "ber");
        let target_unit = a("g", "sil");
        let order = Order::new_support(Power::Germany, unit, target_unit, None);
        assert_eq!(order.to_string(), "A Ber S A Sil");
    }

    #[test]
    fn test_display_support_move() {
        let unit = f("f", "gol");
        let target_unit = f("f", "tyn");
        let order = Order::new_support(Power::France, unit, target_unit, Some(p("nap")));
        assert_eq!(order.to_string(), "F GoL S F Tyn - Nap");
    }

    #[test]
    fn test_display_convoy() {
        let unit = f("e", "nth");
        let target_unit = a("e", "lon");
        let order = Order::new_convoy(Power::England, unit, target_unit, p("bel"));
        assert_eq!(order.to_string(), "F Nth C A Lon - Bel");
    }
}
