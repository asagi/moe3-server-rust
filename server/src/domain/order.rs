use super::OrderId;
use super::Power;
use super::Province;
use super::Unit;
use super::UnitId;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// 命令の定義
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Order {
    pub id: Option<OrderId>,
    pub power: Power,
    pub unit: Unit,
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrderKind {
    Hold(HoldOrder),
    Move(MoveOrder),
    Support(SupportOrder),
    Convoy(ConvoyOrder),
}

/// ホールド命令
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct HoldOrder {
    pub power: Power,
}

/// 移動命令
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MoveOrder {
    pub power: Power,
    pub dest: Province,
}

/// サポート命令
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SupportOrder {
    pub power: Power,
    pub target_unit: Unit,
    pub target_dest: Option<Province>,
}

/// 輸送命令
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ConvoyOrder {
    pub power: Power,
    pub target_unit: Unit,
    pub target_dest: Province,
}

/// 命令のロジック
impl Order {
    /// ユニットIDを取得
    pub fn unit_id(&self) -> Option<UnitId> {
        self.unit.id()
    }

    /// ターゲット命令と一致するかどうかを判定
    pub fn is_matching_target(&self, other_order: &Order) -> bool {
        match &self.kind {
            OrderKind::Support(s) => {
                if s.target_unit.id() != other_order.unit_id() {
                    return false;
                }
                match &other_order.kind {
                    OrderKind::Move(m) => s.target_dest == Some(m.dest),
                    _ => s.target_dest.is_none(),
                }
            }
            OrderKind::Convoy(c) => {
                if c.target_unit.id() != other_order.unit_id() {
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
}

/// 維持命令の実装
impl HoldOrder {
    pub fn new(power: Power, unit: Unit) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Hold(HoldOrder { power }),
        }
    }
}

/// 移動命令の実装
impl MoveOrder {
    pub fn new(power: Power, unit: Unit, dest: Province) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Move(MoveOrder { power, dest }),
        }
    }
}

/// サポート命令の実装
impl SupportOrder {
    pub fn new(power: Power, unit: Unit, target_unit: Unit, target_dest: Option<Province>) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Support(SupportOrder { power, target_unit, target_dest }),
        }
    }
}

/// 輸送命令の実装
impl ConvoyOrder {
    pub fn new(power: Power, unit: Unit, target_unit: Unit, target_dest: Province) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            kind: OrderKind::Convoy(ConvoyOrder { power, target_unit, target_dest }),
        }
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
    use super::super::Army;
    use super::super::Fleet;
    use super::*;

    fn p(code: &str) -> Province {
        Province::from_code(code).expect("valid province code")
    }

    #[test]
    fn test_order_creation() {
        let unit = Army::new(Power::England, p("lon")).with_id(10);
        let order = MoveOrder::new(Power::England, unit, p("lon"));
        assert_eq!(order.unit_id(), Some(10));
        assert_eq!(order.power, Power::England);
    }

    #[test]
    fn test_hold_creation() {
        let unit = Fleet::new(Power::England, p("lon")).with_id(20);
        let order = HoldOrder::new(Power::Austria, unit);
        assert_eq!(order.unit_id(), Some(20));
    }

    #[test]
    fn test_display_hold() {
        let unit = Army::new(Power::Austria, p("vie")); // label は "A vie" と想定
        let order = HoldOrder::new(Power::Austria, unit);

        assert_eq!(order.to_string(), "A vie Holds");
    }

    #[test]
    fn test_display_move() {
        let unit = Army::new(Power::England, p("lon"));
        let order = MoveOrder::new(Power::England, unit, p("wal"));

        assert_eq!(order.to_string(), "A lon - wal");
    }

    #[test]
    fn test_display_support_hold() {
        let unit = Army::new(Power::Germany, p("ber"));
        let target_unit = Army::new(Power::Germany, p("sil"));
        let order = SupportOrder::new(Power::Germany, unit, target_unit, None);
        assert_eq!(order.to_string(), "A ber S A sil");
    }

    #[test]
    fn test_display_support_move() {
        let unit = Fleet::new(Power::France, p("lyo"));
        let target_unit = Fleet::new(Power::France, p("tys"));
        let order = SupportOrder::new(Power::France, unit, target_unit, Some(p("nap")));
        assert_eq!(order.to_string(), "F lyo S F tys - nap");
    }

    #[test]
    fn test_display_convoy() {
        let unit = Fleet::new(Power::England, p("nth"));
        let target_unit = Army::new(Power::England, p("lon"));
        let order = ConvoyOrder::new(Power::England, unit, target_unit, p("bel"));
        assert_eq!(order.to_string(), "F nth C A lon - bel");
    }
}
