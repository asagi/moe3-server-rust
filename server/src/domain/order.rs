use super::OrderId;
use super::Power;
use super::Province;
use super::Unit;
use super::UnitId;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct HoldOrder {
    pub power: Power,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct MoveOrder {
    pub power: Power,
    pub dest: Province,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SupportOrder {
    pub power: Power,
    pub target_unit: Unit,
    pub target_dest: Option<Province>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ConvoyOrder {
    pub power: Power,
    pub target_unit: Unit,
    pub target_dest: Province,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Hold(HoldOrder),
    Move(MoveOrder),
    Support(SupportOrder),
    Convoy(ConvoyOrder),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: Option<OrderId>,
    pub power: Power,
    pub unit: Unit,
    pub status: OrderStatus,
    pub action: Action,
}

impl HoldOrder {
    pub fn new(power: Power, unit: Unit) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            action: Action::Hold(HoldOrder { power }),
        }
    }
}

impl MoveOrder {
    pub fn new(power: Power, unit: Unit, dest: Province) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            action: Action::Move(MoveOrder { power, dest }),
        }
    }
}

impl SupportOrder {
    pub fn new(power: Power, unit: Unit, target_unit: Unit, target_dest: Option<Province>) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            action: Action::Support(SupportOrder { power, target_unit, target_dest }),
        }
    }
}

impl ConvoyOrder {
    pub fn new(power: Power, unit: Unit, target_unit: Unit, target_dest: Province) -> Order {
        Order {
            id: None,
            power: power,
            unit: unit,
            status: OrderStatus::Unresolved,
            action: Action::Convoy(ConvoyOrder { power, target_unit, target_dest }),
        }
    }
}

impl Order {
    pub fn unit_id(&self) -> Option<UnitId> {
        self.unit.id()
    }

    pub fn is_matching_target(&self, other_order: &Order) -> bool {
        match &self.action {
            Action::Support(s) => {
                if s.target_unit.id() != other_order.unit_id() {
                    return false;
                }
                match &other_order.action {
                    Action::Move(m) => s.target_dest == Some(m.dest),
                    _ => s.target_dest.is_none(),
                }
            }
            Action::Convoy(c) => {
                if c.target_unit.id() != other_order.unit_id() {
                    return false;
                }
                match &other_order.action {
                    Action::Move(m) => c.target_dest == m.dest,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = self.unit.label();

        match &self.action {
            Action::Hold(_) => {
                write!(f, "{} Holds", prefix)
            }
            Action::Move(o) => {
                write!(f, "{} - {}", prefix, o.dest)
            }
            Action::Support(o) => {
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
            Action::Convoy(o) => {
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

    #[test]
    fn test_order_creation() {
        let unit = Army::new(Power::England, Province::Lon).with_id(10);
        let order = MoveOrder::new(Power::England, unit, Province::Lon);
        assert_eq!(order.unit_id(), Some(10));
        assert_eq!(order.power, Power::England);
    }

    #[test]
    fn test_hold_creation() {
        let unit = Fleet::new(Power::England, Province::Lon).with_id(20);
        let order = HoldOrder::new(Power::Austria, unit);
        assert_eq!(order.unit_id(), Some(20));
    }

    #[test]
    fn test_display_hold() {
        let unit = Army::new(Power::Austria, Province::Vie); // label は "A vie" と想定
        let order = HoldOrder::new(Power::Austria, unit);

        assert_eq!(order.to_string(), "A vie Holds");
    }

    #[test]
    fn test_display_move() {
        let unit = Army::new(Power::England, Province::Lon);
        let order = MoveOrder::new(Power::England, unit, Province::Wal);

        assert_eq!(order.to_string(), "A lon - wal");
    }

    #[test]
    fn test_display_support_hold() {
        let unit = Army::new(Power::Germany, Province::Ber);
        let target_unit = Army::new(Power::Germany, Province::Sil);
        let order = SupportOrder::new(Power::Germany, unit, target_unit, None);
        assert_eq!(order.to_string(), "A ber S A sil");
    }

    #[test]
    fn test_display_support_move() {
        let unit = Fleet::new(Power::France, Province::Lyo);
        let target_unit = Fleet::new(Power::France, Province::Tys);
        let order = SupportOrder::new(Power::France, unit, target_unit, Some(Province::Nap));
        assert_eq!(order.to_string(), "F lyo S F tys - nap");
    }

    #[test]
    fn test_display_convoy() {
        let unit = Fleet::new(Power::England, Province::Nth);
        let target_unit = Army::new(Power::England, Province::Lon);
        let order = ConvoyOrder::new(Power::England, unit, target_unit, Province::Bel);
        assert_eq!(order.to_string(), "F nth C A lon - bel");
    }
}
