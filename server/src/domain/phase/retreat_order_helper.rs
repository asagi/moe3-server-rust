use crate::domain::order::Order;
use crate::domain::order::OrderKind;

pub trait RetreatOrderHelper {
    fn collect_not_assumed_orders(&self) -> Vec<Order>;
    fn collect_retreat_indices(&self) -> Vec<usize>;
}

impl RetreatOrderHelper for [Order] {
    /// 全ての非仮定命令のコレクションを作成
    fn collect_not_assumed_orders(&self) -> Vec<Order> {
        self.iter().filter(|o| !o.is_assumed()).copied().collect()
    }

    /// 全ての撤退命令のインデックスのコレクションを作成
    fn collect_retreat_indices(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Retreat(_)))
            .map(|(i, _)| i)
            .collect()
    }
}
