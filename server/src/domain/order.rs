use super::OrderId;
use super::power::Power;
use super::province::Province;
use super::unit::Unit;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// 命令の定義
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Order {
    pub id: Option<OrderId>,
    pub power: Power,
    pub unit: Unit,
    pub dislodged_from: Option<Province>,
    pub status: OrderStatus,
    pub kind: OrderKind,
}

/// 命令の状態
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Unresolved,
    Failure,
    Success,
    Dislodged,
    Cut,
    Valid,
    Invalid,
}

/// 命令の種類
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrderKind {
    Hold(HoldOrder),
    Move(MoveOrder),
    Support(SupportOrder),
    Convoy(ConvoyOrder),
}

/// ホールド命令
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct HoldOrder {
    pub power: Power,
}

/// 移動命令
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct MoveOrder {
    pub power: Power,
    pub dest: Province,
}

/// サポート命令
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct SupportOrder {
    pub power: Power,
    pub target_unit: Unit,
    pub target_dest: Option<Province>,
}

/// 輸送命令
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct ConvoyOrder {
    pub power: Power,
    pub target_unit: Unit,
    pub target_dest: Province,
}

/// 命令のロジック
impl Order {
    pub fn new_hold(power: Power, unit: Unit) -> Self {
        Order {
            id: None,
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Hold(HoldOrder { power }),
        }
    }

    pub fn new_move(power: Power, unit: Unit, dest: Province) -> Self {
        Order {
            id: None,
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Move(MoveOrder { power, dest }),
        }
    }

    pub fn new_support(power: Power, unit: Unit, target_unit: Unit, target_dest: Option<Province>) -> Self {
        Order {
            id: None,
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Support(SupportOrder { power, target_unit, target_dest }),
        }
    }

    pub fn new_convoy(power: Power, unit: Unit, target_unit: Unit, target_dest: Province) -> Self {
        Order {
            id: None,
            power,
            unit,
            dislodged_from: None,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Convoy(ConvoyOrder { power, target_unit, target_dest }),
        }
    }

    /// ユニットの現在地を返す
    pub fn location(&self) -> Province {
        self.unit.location()
    }

    /// ターゲット命令と一致するかどうかを判定
    pub fn is_matching_target(&self, other_order: &Order) -> bool {
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
    pub fn set_unresolved(&mut self) {
        self.status = OrderStatus::Unresolved;
    }

    /// ステータスを Failure に変更
    pub fn set_failure(&mut self) {
        self.status = OrderStatus::Failure;
    }

    /// ステータスを Success に変更
    pub fn set_success(&mut self) {
        self.status = OrderStatus::Success;
    }

    /// ステータスを Dislodged に変更
    pub fn set_dislodged(&mut self) {
        self.status = OrderStatus::Dislodged;
    }

    /// ステータスを Cut に変更
    pub fn set_cut(&mut self) {
        self.status = OrderStatus::Cut;
    }

    /// ステータスを Valid に変更
    pub fn set_valid(&mut self) {
        self.status = OrderStatus::Valid;
    }

    /// ステータスを Invalid に変更
    pub fn set_invalid(&mut self) {
        self.status = OrderStatus::Invalid;
    }

    /// ステータスが `Unresolved` かどうか
    pub fn is_unresolved(&self) -> bool {
        self.status == OrderStatus::Unresolved
    }

    /// ステータスが `Success` かどうか
    pub fn is_success(&self) -> bool {
        self.status == OrderStatus::Success
    }

    /// ステータスが `Dislodged` かどうか
    pub fn is_dislodged(&self) -> bool {
        self.status == OrderStatus::Dislodged
    }

    /// ステータスが `Cut` かどうか
    pub fn is_cut(&self) -> bool {
        self.status == OrderStatus::Cut
    }

    /// ステータスが `Valid` かどうか
    pub fn is_valid(&self) -> bool {
        self.status == OrderStatus::Valid
    }

    /// ステータスが `Invalid` かどうか
    pub fn is_invalid(&self) -> bool {
        self.status == OrderStatus::Invalid
    }

    /// ステータスが `Failure` かどうか
    pub fn is_failure(&self) -> bool {
        self.status == OrderStatus::Failure
    }

    /// 命令が他の勢力のユニットに対するもの（仮定命令）であるかどうか
    pub fn is_assumed(&self) -> bool {
        self.power != self.unit.power()
    }

    pub(crate) fn set_dislodged_from(&mut self, winner_location: &Province) {
        self.dislodged_from = Some(*winner_location);
    }
}

/// 命令を Diplomacy 風の短縮表記で整形する。
///
/// 例:
/// - Hold: A lon Holds
/// - Move: A lon - wal
/// - Support: A lon S A wal - yor
/// - Convoy: F eng C A lon - bre
impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = self.unit.label();

        match &self.kind {
            OrderKind::Hold(_) => {
                write!(f, "{} Holds", prefix)
            }
            OrderKind::Move(o) => {
                write!(f, "{} - {}", prefix, o.dest)
            }
            OrderKind::Support(o) => {
                // 自分の勢力とターゲットの勢力が違う場合、形容詞を取得
                let target_label = if self.power != o.target_unit.power() {
                    format!("{} {}", o.target_unit.power().adjective(), o.target_unit.label())
                } else {
                    o.target_unit.label()
                };

                if let Some(dest) = o.target_dest {
                    write!(f, "{} S {} - {}", prefix, target_label, dest)
                } else {
                    write!(f, "{} S {}", prefix, target_label)
                }
            }
            OrderKind::Convoy(o) => {
                let target_label = if self.power != o.target_unit.power() {
                    format!("{} {}", o.target_unit.power().adjective(), o.target_unit.label())
                } else {
                    o.target_unit.label()
                };

                write!(f, "{} C {} - {}", prefix, target_label, o.target_dest)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(code: &str) -> Province {
        Province::from_code(code).expect("valid province code")
    }

    #[test]
    fn test_order_creation() {
        let unit = Unit::new_army(Power::England, p("lon"));
        let order = Order::new_move(Power::England, unit, p("lon"));
        assert_eq!(order.unit, unit);
        assert_eq!(order.power, Power::England);
    }

    #[test]
    fn test_hold_creation() {
        let unit = Unit::new_fleet(Power::England, p("lon"));
        let order = Order::new_hold(Power::Austria, unit);
        assert_eq!(order.unit, unit);
    }

    #[test]
    fn test_display_hold() {
        let unit = Unit::new_army(Power::Austria, p("vie"));
        let order = Order::new_hold(Power::Austria, unit);

        assert_eq!(order.to_string(), "A vie Holds");
    }

    #[test]
    fn test_display_move() {
        let unit = Unit::new_army(Power::England, p("lon"));
        let order = Order::new_move(Power::England, unit, p("wal"));

        assert_eq!(order.to_string(), "A lon - wal");
    }

    #[test]
    fn test_display_support_hold() {
        let unit = Unit::new_army(Power::Germany, p("ber"));
        let target_unit = Unit::new_army(Power::Germany, p("sil"));
        let order = Order::new_support(Power::Germany, unit, target_unit, None);
        assert_eq!(order.to_string(), "A ber S A sil");
    }

    #[test]
    fn test_display_support_move() {
        let unit = Unit::new_fleet(Power::France, p("lyo"));
        let target_unit = Unit::new_fleet(Power::France, p("tys"));
        let order = Order::new_support(Power::France, unit, target_unit, Some(p("nap")));
        assert_eq!(order.to_string(), "F lyo S F tys - nap");
    }

    #[test]
    fn test_display_convoy() {
        let unit = Unit::new_fleet(Power::England, p("nth"));
        let target_unit = Unit::new_army(Power::England, p("lon"));
        let order = Order::new_convoy(Power::England, unit, target_unit, p("bel"));
        assert_eq!(order.to_string(), "F nth C A lon - bel");
    }
}
