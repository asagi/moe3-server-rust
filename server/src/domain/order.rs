use serde::Deserialize;
use serde::Serialize;
use std::fmt;

use super::OrderId;
use super::Power;
use super::Province;
use super::UnitId;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Unresolved, // 未解決
    Failure,    // 失敗
    Success,    // 成功
    Dislodged,  // 敗退
    Cut,        // カット
    Valid,      // 有効
    Invalid,    // 無効
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Hold,
    Move { dest: Province },
    Support { target_unit_id: UnitId, target_dest: Option<Province> },
    Convoy { target_unit_id: UnitId, target_dest: Province },
    Retreat { dest: Province },
    Disband,
    GainArmy,
    GainFleet,
    Lose,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: Option<OrderId>,
    pub power: Power,
    pub unit_id: Option<UnitId>,
    pub status: OrderStatus,
    pub action: Action,
}

impl Order {
    pub fn new(power: Power, unit_id: Option<UnitId>, action: Action) -> Self {
        Self {
            id: None,
            power,
            unit_id,
            status: OrderStatus::Unresolved,
            action,
        }
    }

    pub fn is_matching_target(&self, other_order: &Order) -> bool {
        match &self.action {
            // Support の場合
            Action::Support { target_unit_id, target_dest } => {
                if Some(*target_unit_id) != other_order.unit_id {
                    return false;
                }

                match &other_order.action {
                    Action::Move { dest } => Some(*dest) == *target_dest,
                    _ => target_dest.is_none(),
                }
            }

            Action::Convoy { target_unit_id, target_dest } => {
                // 1. ユニットIDチェック
                if Some(*target_unit_id) != other_order.unit_id {
                    return false;
                }

                match &other_order.action {
                    Action::Move { dest } => dest == target_dest,
                    _ => false,
                }
            }

            _ => false,
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit_prefix = match &self.unit_id {
            Some(unit_id) => format!("Unit({})", unit_id),
            None => "No Unit".to_string(),
        };

        match &self.action {
            Action::Hold => write!(f, "{} Holds", unit_prefix),

            Action::Move { dest } => write!(f, "{} - {}", unit_prefix, dest),

            Action::Support { target_unit_id, target_dest } => {
                if let Some(dest) = target_dest {
                    write!(f, "{} S Unit({}) - {}", unit_prefix, target_unit_id, dest)
                } else {
                    write!(f, "{} S Unit({})", unit_prefix, target_unit_id)
                }
            }

            Action::Convoy { target_unit_id, target_dest } => {
                write!(f, "{} C Unit({}) - {}", unit_prefix, target_unit_id, target_dest)
            }
            _ => write!(f, "{}: {:?}", unit_prefix, self.action),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_support_matching() {
        // 1. 移動するユニット（相手）
        let move_order = Order {
            id: None,
            power: Power::Austria,
            unit_id: Some(10),
            status: OrderStatus::Unresolved,
            action: Action::Move { dest: Province::Cly },
        };

        // 2. 正しい移動支援（自分）
        let support_move = Order {
            id: None,
            power: Power::England,
            unit_id: Some(11),
            status: OrderStatus::Unresolved,
            action: Action::Support {
                target_unit_id: 10,
                target_dest: Some(Province::Cly),
            },
        };

        // 3. 目的地が違う支援（自分）
        let wrong_support = Order {
            id: None,
            power: Power::England,
            unit_id: Some(11),
            status: OrderStatus::Unresolved,
            action: Action::Support {
                target_unit_id: 10,
                target_dest: Some(Province::Yor), // 違う場所を支援している
            },
        };

        // アサーション（検証）
        assert!(support_move.is_matching_target(&move_order)); // 成功するはず
        assert!(!wrong_support.is_matching_target(&move_order)); // 失敗するはず
    }

    #[test]
    fn test_order_serialization_move() {
        let order = Order {
            id: None,
            power: Power::England,
            unit_id: Some(1),
            status: OrderStatus::Unresolved,
            action: Action::Move { dest: Province::Nth },
        };

        let json = serde_json::to_string(&order).unwrap();

        assert!(json.contains("\"type\":\"move\""));
        assert!(json.contains("\"dest\":\"nth\""));
    }

    #[test]
    fn test_order_serialization_hold() {
        let order = Order::new(Power::Austria, Some(2), Action::Hold);

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("\"type\":\"hold\""));
    }

    #[test]
    fn test_order_serialization_format_strict() {
        use serde_json::json;

        // 1. テストデータの作成
        let order = Order {
            id: Some(1),
            power: Power::England,
            unit_id: Some(10),
            status: OrderStatus::Unresolved,
            action: Action::Move { dest: Province::Lon },
        };

        let actual_json: serde_json::Value = serde_json::to_value(&order).unwrap();
        let expected_json = json!({
            "id": 1,
            "power": "England",
            "unit_id": 10,
            "status": "unresolved",
            "action": {
                "type": "move",
                "dest": "lon"
            }
        });

        assert_eq!(actual_json, expected_json);
    }
}
