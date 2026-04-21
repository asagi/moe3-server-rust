// ============================================================================
// imports
// ============================================================================

use super::Order;
use super::OrderKind;
use super::Power;

// ============================================================================
// definitions
// ============================================================================

pub(crate) trait AdjustmentOrderHelper {
    fn get_unresolved_build_idxs_by_power(&self, power: &Power) -> Option<usize>;
    fn collect_unresolved_adjustment_idxs(&self) -> Vec<usize>;
    fn get_unresolved_disband_idxs_by_power(&self, power: &Power) -> Option<usize>;
    fn collect_valid_adjustment_idxs(&self) -> Vec<usize>;
}

impl AdjustmentOrderHelper for [Order] {
    /// 指定した国の未処理の建造命令のインデックスを取得
    fn get_unresolved_build_idxs_by_power(&self, power: &Power) -> Option<usize> {
        self.iter()
            .enumerate()
            .find(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Build(_)) && &o.unit.power == power)
            .map(|(i, _)| i)
    }

    /// 指定した国の未処理の解体命令のインデックスを取得
    fn get_unresolved_disband_idxs_by_power(&self, power: &Power) -> Option<usize> {
        self.iter()
            .enumerate()
            .find(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Disband(_)) && &o.unit.power == power)
            .map(|(i, _)| i)
    }

    /// 全ての未処理命令のインデックスを取得
    fn collect_unresolved_adjustment_idxs(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved())
            .map(|(i, _)| i)
            .collect()
    }

    /// 全ての有効な命令のインデックスを取得
    fn collect_valid_adjustment_idxs(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| o.is_valid())
            .map(|(i, _)| i)
            .collect()
    }
}
