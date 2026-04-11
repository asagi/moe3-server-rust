use crate::domain::order::Order;
use crate::domain::power::Power;

pub(crate) trait AdjustmentOrderHelper {
    fn get_unresolved_order_idxs_by_power(&self, power: &Power) -> Option<usize>;
    fn collect_unresolved_order_idxs(&self) -> Vec<usize>;
}

impl AdjustmentOrderHelper for [Order] {
    /// 指定した国の未処理命令のインデックスを取得
    fn get_unresolved_order_idxs_by_power(&self, power: &Power) -> Option<usize> {
        self.iter()
            .enumerate()
            .find(|(_, o)| o.is_unresolved() && &o.unit.power == power)
            .map(|(i, _)| i)
    }

    /// 全ての未処理命令のインデックスを取得
    fn collect_unresolved_order_idxs(&self) -> Vec<usize> {
        self.iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved())
            .map(|(i, _)| i)
            .collect()
    }
}
