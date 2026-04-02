use crate::domain::order::ConvoyOrder;
use crate::domain::order::MoveOrder;
use crate::domain::order::Order;
use crate::domain::order::OrderKind;
use crate::domain::power::Power;
use crate::domain::province::Province;
use indexmap::IndexSet;
use std::collections::HashSet;

pub trait OrderHelper {
    fn collect_not_invalid_orders(&self) -> Vec<Order>;
    fn collect_valid_order_indices(&self) -> Vec<usize>;
    fn collect_unresolved_indices(&self) -> Vec<usize>;
    fn collect_unresolved_move_indices(&self) -> Vec<usize>;
    fn collect_valid_move_orders(&self) -> Vec<Order>;
    fn collect_valid_move_indices(&self) -> Vec<usize>;
    fn collect_attacker_indicies(&self, target_idx: usize) -> Vec<usize>;
    fn collect_unresolved_support_indices(&self) -> Vec<usize>;
    fn collect_valid_support_orders(&self) -> Vec<Order>;
    fn collect_valid_support_indices(&self) -> Vec<usize>;
    fn collect_unresolved_convoy_indices(&self) -> Vec<usize>;
    fn collect_valid_convoy_orders(&self) -> Vec<Order>;
    fn collect_valid_convoy_indices(&self) -> Vec<usize>;
    fn collect_matched_convoy_order_indices(&self, attack_order: &Order) -> Vec<usize>;
    fn collect_valid_move_destination_set(&self) -> IndexSet<Province>;
    fn collect_fleet_water_codes(&self) -> HashSet<&'static str>;
    fn find_support_target_idx(&self, support_idx: usize) -> Option<usize>;
    fn find_convoy_target_idx(&self, convoy_idx: usize) -> Option<usize>;
    fn find_convoy_at_dest_idx(&self, m: MoveOrder) -> Option<usize>;
    fn find_support_at_dest_idx(&self, move_idx: usize) -> Option<usize>;
    fn find_occupant_order_idx(&self, target_location_code: &str) -> Option<usize>;
    fn get_support_target_move_order_kind(&self, support_order_idx: usize) -> Option<MoveOrder>;
    fn get_support_target_convoy_order_kind(&self, support_order_idx: usize) -> Option<(usize, ConvoyOrder)>;
    fn count_supports(&self, target_idx: usize, exclude_power: Option<&Power>) -> usize;
    fn count_max_supports_for_attackers(&self, attacker_indicies: &[usize], target_idx: usize) -> usize;
    fn has_confliction(&self, target_location_code: &str) -> bool;
    fn occupant_power_on_target(&self, target_code: &str) -> Option<Power>;
}

impl OrderHelper for [Order] {
    /// 全ての命令のコレクションを作成
    fn collect_not_invalid_orders(&self) -> Vec<Order> {
        self.iter().filter(|o| !o.is_assumed() && !o.is_invalid()).copied().collect()
    }

    /// 全ての有効な命令のインデックスコレクションを作成
    fn collect_valid_order_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && !o.is_invalid() && !o.is_dislodged() && !o.is_unreachable())
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の命令のインデックスコレクションを作成
    fn collect_unresolved_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_unresolved())
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の移動命令のインデックスコレクションを作成
    fn collect_unresolved_move_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Move(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 有効な移動命令のコレクションを作成
    fn collect_valid_move_orders(&self) -> Vec<Order> {
        self.iter()
            .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Move(_)))
            .copied()
            .collect()
    }

    /// 有効な移動命令のインデックスコレクションを作成
    fn collect_valid_move_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Move(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 指定したユニットを攻撃する移動命令を収集（自国軍を除く）
    fn collect_attacker_indicies(&self, target_idx: usize) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_valid() && o.power != self[target_idx].power)
            .filter(|(_, o)| {
                if let OrderKind::Move(m) = &o.kind {
                    m.dest.code()[..3] == self[target_idx].location().code()[..3]
                } else {
                    false
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の支援命令のインデックスコレクションを作成
    fn collect_unresolved_support_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Support(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 有効な支援命令のコレクションを作成
    fn collect_valid_support_orders(&self) -> Vec<Order> {
        self.iter()
            .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Support(_)))
            .copied()
            .collect()
    }

    /// 有効な支援命令のインデックスコレクションを作成
    fn collect_valid_support_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Support(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の輸送命令のインデックスコレクションを作成
    fn collect_unresolved_convoy_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Convoy(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 有効な輸送命令のコレクションを作成
    fn collect_valid_convoy_orders(&self) -> Vec<Order> {
        self.iter()
            .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Convoy(_)))
            .copied()
            .collect()
    }

    /// 有効な輸送命令のインデックスコレクションを作成
    fn collect_valid_convoy_indices(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Convoy(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// attack_order にマッチする輸送命令のインデックスコレクションを返す
    fn collect_matched_convoy_order_indices(&self, attack_order: &Order) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| {
                matches!(o.kind, OrderKind::Convoy(_)) && o.is_matching_target(attack_order) && o.location().is_water()
            })
            .map(|(idx, _)| idx)
            .collect()
    }

    /// 未解決の有効な移動命令の移動先を重複なしで収集取する
    fn collect_valid_move_destination_set(&self) -> IndexSet<Province> {
        self.iter()
            .filter(|o| !o.is_assumed() && o.is_valid())
            .filter_map(|o| if let OrderKind::Move(m) = o.kind { Some(m.dest) } else { None })
            .collect()
    }

    /// 水域にいる全ての艦隊の現在地コードのコレクションを作成
    fn collect_fleet_water_codes(&self) -> HashSet<&'static str> {
        self.iter()
            .filter(|o| o.unit.is_fleet() && o.location().is_water())
            .map(|o| o.location().code())
            .collect()
    }

    /// 支援対象の命令のインデックスを返す
    /// - Invalid な移動命令は支援対象にならないので除外する
    fn find_support_target_idx(&self, support_idx: usize) -> Option<usize> {
        self.iter().position(|o| {
            !o.is_assumed()
                && self[support_idx].is_matching_target(o)
                && !(matches!(o.kind, OrderKind::Move(_)) && o.is_invalid())
        })
    }

    /// 輸送対象の移動命令のインデックスを返す
    fn find_convoy_target_idx(&self, convoy_idx: usize) -> Option<usize> {
        self.iter()
            .position(|o| self[convoy_idx].is_matching_target(o) && matches!(o.kind, OrderKind::Move(_)))
    }

    /// 移動先に輸送命令があればその輸送命令のインデックスを返す
    fn find_convoy_at_dest_idx(&self, m: MoveOrder) -> Option<usize> {
        self.iter().position(|o| o.location() == m.dest)
    }

    /// 移動先に支援命令があればその輸送命令のインデックスを返す
    fn find_support_at_dest_idx(&self, move_idx: usize) -> Option<usize> {
        let OrderKind::Move(m) = self[move_idx].kind else {
            return None;
        };
        self.iter()
            .position(|o| o.location() == m.dest && matches!(o.kind, OrderKind::Support(_)))
    }

    /// 指定地域に非移動命令または失敗した移動命令があればその命令のインデックスを返す
    fn find_occupant_order_idx(&self, target_location_code: &str) -> Option<usize> {
        self.iter().enumerate().position(|(_, o)| {
            !o.is_assumed()
                && o.location().code()[..3] == target_location_code[..3]
                && !(matches!(o.kind, OrderKind::Move(_)) && o.is_success())
        })
    }

    /// 支援対象が移動命令であればその移動命令の MoveOrder オブジェクトを返す
    fn get_support_target_move_order_kind(&self, support_order_idx: usize) -> Option<MoveOrder> {
        let target_order = self.iter().find(|o| self[support_order_idx].is_matching_target(o))?;
        match target_order.kind {
            OrderKind::Move(m) => Some(m),
            _ => None,
        }
    }

    /// 支援対象が輸送命令であればその輸送命令の ConvoyOrder オブジェクトを返す
    fn get_support_target_convoy_order_kind(&self, support_order_idx: usize) -> Option<(usize, ConvoyOrder)> {
        let c_idx = self.iter().position(|o| self[support_order_idx].is_matching_target(o))?;
        let OrderKind::Convoy(ck) = &self[c_idx].kind.clone() else {
            return None;
        };
        Some((c_idx, *ck))
    }

    /// 対象へのサポート数を数える
    fn count_supports(&self, target_idx: usize, exclude_power: Option<&Power>) -> usize {
        let support_orders = self.collect_valid_support_orders();
        support_orders
            .iter()
            .filter(|s| s.is_matching_target(&self[target_idx]))
            .filter(|s| match exclude_power {
                Some(power) => s.power != *power,
                None => true,
            })
            .count()
    }

    /// 対象への攻撃命令群の中で、最も支援数の多い攻撃命令の支援数を返す
    fn count_max_supports_for_attackers(&self, attacker_indicies: &[usize], target_idx: usize) -> usize {
        let support_orders = self.collect_valid_support_orders();
        attacker_indicies
            .iter()
            .map(|&i| {
                support_orders
                    .iter()
                    .filter(|s| s.is_matching_target(&self[i]) && s.power != self[target_idx].power)
                    .count()
            })
            .max()
            .unwrap_or(0)
    }

    /// 指定地点に移動を試みる複数の移動命令が存在するかどうかを判定する。
    fn has_confliction(&self, target_location_code: &str) -> bool {
        self.collect_valid_move_indices()
            .into_iter()
            .filter(|&idx| {
                if let OrderKind::Move(m) = self[idx].kind {
                    m.dest.code()[..3] == target_location_code[..3]
                } else {
                    false
                }
            })
            .count()
            > 1
    }

    /// 指定地点に非移動命令または失敗が予想される移動命令があればその勢力を返す
    fn occupant_power_on_target(&self, target_code: &str) -> Option<Power> {
        let occupant_order = self
            .iter()
            .find(|o| !o.is_assumed() && o.location().code()[..3] == target_code[..3])?;

        if !matches!(occupant_order.kind, OrderKind::Move(_)) {
            // 非移動命令が存在すればその勢力を返す
            return Some(occupant_order.unit.power);
        }

        if occupant_order.is_valid() {
            // 未処理の移動命令が存在すればその勢力を返す
            return Some(occupant_order.unit.power);
        }

        if occupant_order.is_failure() {
            // 失敗した移動命令が存在すればその勢力を返す
            return Some(occupant_order.power);
        }

        if occupant_order.is_success() {
            // 成功した移動命令は不在とみなす
            return None;
        }

        None
    }
}
