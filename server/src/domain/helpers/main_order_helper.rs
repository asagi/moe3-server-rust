// ============================================================================
// imports
// ============================================================================

use std::collections::HashSet;

use indexmap::IndexSet;

use super::ConvoyOrder;
use super::MoveOrder;
use super::Order;
use super::OrderKind;
use super::Power;
use super::Province;

// ============================================================================
// definitions
// ============================================================================

///
/// メインフェイズの命令処理補助関数を提供するトレイト
///
pub trait MainOrderHelper {
    fn collect_not_assumed_orders(&self) -> Vec<Order>;
    fn collect_not_invalid_main_orders(&self) -> Vec<Order>;
    fn collect_unresolved_idxs(&self) -> Vec<usize>;
    fn collect_unresolved_move_idxs(&self) -> Vec<usize>;
    fn collect_valid_move_idxs(&self) -> Vec<usize>;
    fn collect_non_dislodged_move_idxs(&self) -> Vec<usize>;
    fn collect_attacker_indicies(&self, target_idx: usize) -> Vec<usize>;
    fn collect_unresolved_support_idxs(&self) -> Vec<usize>;
    fn collect_valid_support_orders(&self) -> Vec<Order>;
    fn collect_valid_support_idxs(&self) -> Vec<usize>;
    fn collect_unresolved_convoy_idxs(&self) -> Vec<usize>;
    fn collect_valid_convoy_idxs(&self) -> Vec<usize>;
    fn collect_matched_convoy_order_idxs(&self, attack_order: &Order) -> Vec<usize>;
    fn collect_own_matching_convoy_idxs(&self, move_idx: usize) -> Vec<usize>;
    fn collect_fleet_water_codes(&self) -> HashSet<&'static str>;
    fn collect_valid_move_destination_code_set(&self) -> IndexSet<&'static str>;
    fn find_support_target_idx(&self, support_idx: usize) -> Option<usize>;
    fn find_convoy_target_idx(&self, convoy_idx: usize) -> Option<usize>;
    fn find_convoy_at_dest_idx(&self, m: MoveOrder) -> Option<usize>;
    fn find_support_at_dest_idx(&self, move_idx: usize) -> Option<usize>;
    fn find_occupant_order_idx(&self, target_code: &str) -> Option<usize>;
    fn find_opposite_move_idx(&self, move_idx: usize) -> Option<usize>;
    fn get_support_target_move_order_kind(&self, support_idx: usize) -> Option<MoveOrder>;
    fn get_support_target_convoy_order_kind(&self, support_idx: usize) -> Option<(usize, ConvoyOrder)>;
    fn get_occupant_power_on_target(&self, target_code: &str) -> Option<Power>;
    fn get_unresolved_allowed_waters(&self, move_idx: usize) -> HashSet<&'static str>;
    fn get_valid_allowed_waters(&self, move_idx: usize, exclude_province: Option<Province>) -> HashSet<&'static str>;
    fn count_supports(&self, target_idx: usize, exclude_power: Option<Power>) -> usize;
    fn count_max_supports_for_attackers(&self, attacker_indicies: &[usize], target_idx: usize) -> usize;
    fn has_supports_excluding_occupant_power(&self, move_idx: usize, target_power: Option<Power>) -> bool;
}

/// メインフェイズの命令処理補助関数を提供するトレイトの実装
impl MainOrderHelper for [Order] {
    /// 全ての非仮定命令のコレクションを作成
    fn collect_not_assumed_orders(&self) -> Vec<Order> {
        self.iter().filter(|o| !o.is_assumed()).copied().collect()
    }

    /// 全ての命令のコレクションを作成
    fn collect_not_invalid_main_orders(&self) -> Vec<Order> {
        self.collect_not_assumed_orders()
            .iter()
            .filter(|o| !o.is_invalid())
            .copied()
            .collect()
    }

    /// 未処理の命令のインデックスコレクションを作成
    fn collect_unresolved_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved())
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の移動命令のインデックスコレクションを作成
    fn collect_unresolved_move_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Move(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 有効な移動命令のインデックスコレクションを作成
    fn collect_valid_move_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_valid() && matches!(o.kind, OrderKind::Move(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 撃退されていない有効な移動命令（失敗判定済み含む）のインデックスコレクションを作成
    fn collect_non_dislodged_move_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| matches!(o.kind, OrderKind::Move(_)))
            .filter(|(_, o)| !o.is_invalid() && !o.is_dislodged() && !o.is_unreachable())
            .map(|(i, _)| i)
            .collect()
    }

    /// 指定したユニットを攻撃する移動命令を収集（自国軍を除く）
    fn collect_attacker_indicies(&self, target_idx: usize) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_valid() && o.power != self[target_idx].power)
            .filter(|(_, o)| {
                if let OrderKind::Move(m) = &o.kind {
                    m.dest.code() == self[target_idx].location().code()
                } else {
                    false
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の支援命令のインデックスコレクションを作成
    fn collect_unresolved_support_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Support(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 有効な支援命令のコレクションを作成
    fn collect_valid_support_orders(&self) -> Vec<Order> {
        self.collect_not_assumed_orders()
            .iter()
            .filter(|o| o.is_valid() && matches!(o.kind, OrderKind::Support(_)))
            .copied()
            .collect()
    }

    /// 有効な支援命令のインデックスコレクションを作成
    fn collect_valid_support_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_valid() && matches!(o.kind, OrderKind::Support(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 未処理の輸送命令のインデックスコレクションを作成
    fn collect_unresolved_convoy_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Convoy(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 有効な輸送命令のインデックスコレクションを作成
    fn collect_valid_convoy_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_valid() && matches!(o.kind, OrderKind::Convoy(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 移動命令にマッチする輸送命令のインデックスコレクションを返す
    fn collect_matched_convoy_order_idxs(&self, attack_order: &Order) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                matches!(o.kind, OrderKind::Convoy(_)) && o.is_matching_target(attack_order) && o.location().is_water()
            })
            .map(|(idx, _)| idx)
            .collect()
    }

    /// 移動命令にマッチする自国の輸送命令のインデックスコレクションを返す
    fn collect_own_matching_convoy_idxs(&self, move_idx: usize) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_valid() && matches!(o.kind, OrderKind::Convoy(_)))
            .filter(|(_, o)| o.is_matching_target(&self[move_idx]) && o.power == self[move_idx].power)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// 水域にいる全ての艦隊の現在地コードのコレクションを作成
    fn collect_fleet_water_codes(&self) -> HashSet<&'static str> {
        self.collect_not_assumed_orders()
            .iter()
            .filter(|o| o.unit.is_fleet() && o.location().is_water())
            .map(|o| o.location().code_with_coast())
            .collect()
    }

    /// 未解決の有効な移動命令の移動先を重複なしで収集取する
    fn collect_valid_move_destination_code_set(&self) -> IndexSet<&'static str> {
        self.collect_not_assumed_orders()
            .iter()
            .filter(|o| o.is_valid())
            .filter_map(|o| if let OrderKind::Move(m) = o.kind { Some(m.dest) } else { None })
            .map(|p| p.code())
            .collect()
    }

    /// 支援対象の命令のインデックスを返す
    /// - Invalid な移動命令は支援対象にならないので除外する
    fn find_support_target_idx(&self, support_idx: usize) -> Option<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .position(|o| self[support_idx].is_matching_target(o) && !(matches!(o.kind, OrderKind::Move(_)) && o.is_invalid()))
    }

    /// 輸送対象の移動命令のインデックスを返す
    fn find_convoy_target_idx(&self, convoy_idx: usize) -> Option<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .position(|o| self[convoy_idx].is_matching_target(o) && matches!(o.kind, OrderKind::Move(_)))
    }

    /// 移動先に輸送命令があればその輸送命令のインデックスを返す
    fn find_convoy_at_dest_idx(&self, m: MoveOrder) -> Option<usize> {
        self.collect_not_assumed_orders().iter().position(|o| o.location() == m.dest)
    }

    /// 移動先に支援命令があればその輸送命令のインデックスを返す
    fn find_support_at_dest_idx(&self, move_idx: usize) -> Option<usize> {
        let OrderKind::Move(m) = self[move_idx].kind else {
            return None;
        };
        self.collect_not_assumed_orders()
            .iter()
            .position(|o| o.location() == m.dest && matches!(o.kind, OrderKind::Support(_)))
    }

    /// 指定地域に非移動命令または失敗した移動命令があればその命令のインデックスを返す
    fn find_occupant_order_idx(&self, target_location_code: &str) -> Option<usize> {
        self.collect_not_assumed_orders().iter().enumerate().position(|(_, o)| {
            o.location().code() == target_location_code && !(matches!(o.kind, OrderKind::Move(_)) && o.is_success())
        })
    }

    /// 移動命令に対する対向移動命令のインデックスを返す
    fn find_opposite_move_idx(&self, move_idx: usize) -> Option<usize> {
        self.collect_not_assumed_orders().iter().position(|o| match o.kind {
            OrderKind::Move(_) => {
                o.location().code() == self[move_idx].dest().code() && o.dest().code() == self[move_idx].location().code()
            }
            _ => false,
        })
    }

    /// 支援対象が移動命令であればその移動命令の MoveOrder オブジェクトを返す
    fn get_support_target_move_order_kind(&self, support_order_idx: usize) -> Option<MoveOrder> {
        let target_order = self
            .collect_not_assumed_orders()
            .into_iter()
            .find(|o| self[support_order_idx].is_matching_target(o))?;
        match target_order.kind {
            OrderKind::Move(m) => Some(m),
            _ => None,
        }
    }

    /// 支援対象が輸送命令であればその輸送命令の ConvoyOrder オブジェクトを返す
    fn get_support_target_convoy_order_kind(&self, support_order_idx: usize) -> Option<(usize, ConvoyOrder)> {
        let c_idx = self
            .collect_not_assumed_orders()
            .iter()
            .position(|o| self[support_order_idx].is_matching_target(o))?;
        let OrderKind::Convoy(ck) = &self[c_idx].kind.clone() else {
            return None;
        };
        Some((c_idx, *ck))
    }

    /// 指定地点に非移動命令または失敗が予想される移動命令があればその勢力を返す
    fn get_occupant_power_on_target(&self, target_code: &str) -> Option<Power> {
        let occupant_order = self
            .collect_not_assumed_orders()
            .into_iter()
            .find(|o| o.location().code() == target_code)?;

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

    /// 移動命令にマッチする輸送命令が存在する水域コードのコレクションを返す
    fn get_unresolved_allowed_waters(&self, move_idx: usize) -> HashSet<&'static str> {
        self.collect_matched_convoy_order_idxs(&self[move_idx])
            .iter()
            .map(|&idx| self[idx].location().code_with_coast())
            .collect()
    }

    /// 移動命令にマッチする輸送命令が存在する水域コードのコレクションを返す
    fn get_valid_allowed_waters(&self, move_idx: usize, exclude_province: Option<Province>) -> HashSet<&'static str> {
        self.collect_matched_convoy_order_idxs(&self[move_idx])
            .iter()
            .filter(|&&idx| self[idx].is_valid() && Some(self[idx].location()) != exclude_province)
            .map(|&idx| self[idx].location().code_with_coast())
            .collect()
    }

    /// 対象へのサポート数を数える
    fn count_supports(&self, target_idx: usize, exclude_power: Option<Power>) -> usize {
        self.collect_valid_support_orders()
            .iter()
            .filter(|s| s.is_matching_target(&self[target_idx]))
            .filter(|s| match exclude_power {
                Some(power) => s.power != power,
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

    fn has_supports_excluding_occupant_power(&self, move_idx: usize, target_power: Option<Power>) -> bool {
        self.collect_valid_support_orders()
            .iter()
            .any(|s| s.is_matching_target(&self[move_idx]) && Some(s.power) != target_power)
    }
}
