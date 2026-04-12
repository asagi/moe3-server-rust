use super::super::models::order::Order;
use super::super::models::order::OrderKind;
use std::collections::HashSet;

pub trait RetreatOrderHelper {
    fn collect_not_assumed_orders(&self) -> Vec<Order>;
    fn collect_retreat_destination_code_set(&self) -> HashSet<&'static str>;
    fn collect_unresolved_retreat_idxs(&self) -> Vec<usize>;
    fn collect_unresolved_retreat_idxs_by_dest(&self, dest_code: &str) -> Vec<usize>;
    fn collect_not_invalid_retreat_orders(&self) -> Vec<Order>;
}

impl RetreatOrderHelper for [Order] {
    /// 全ての非仮定命令のコレクションを作成
    fn collect_not_assumed_orders(&self) -> Vec<Order> {
        self.iter().filter(|o| !o.is_assumed()).copied().collect()
    }

    /// 全ての撤退命令の移動先コードセットを作成
    fn collect_retreat_destination_code_set(&self) -> HashSet<&'static str> {
        self.collect_not_assumed_orders()
            .iter()
            .filter_map(|o| {
                if let OrderKind::Retreat(r) = o.kind {
                    Some(r.dest)
                } else {
                    None
                }
            })
            .map(|p| &p.code()[..3])
            .collect()
    }

    /// 未処理の撤退命令のインデックスコレクションを作成
    fn collect_unresolved_retreat_idxs(&self) -> Vec<usize> {
        self.collect_not_assumed_orders()
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_unresolved() && matches!(o.kind, OrderKind::Retreat(_)))
            .map(|(i, _)| i)
            .collect()
    }

    /// 指定地域に対する未処理の撤退命令のインデックスコレクションを作成
    fn collect_unresolved_retreat_idxs_by_dest(&self, dest_code: &str) -> Vec<usize> {
        self.collect_unresolved_retreat_idxs()
            .into_iter()
            .filter(|&idx| self[idx].dest().code()[..3] == dest_code[..3])
            .collect()
    }

    /// 全ての無効でない命令のコレクションを作成
    fn collect_not_invalid_retreat_orders(&self) -> Vec<Order> {
        self.iter().filter(|o| !o.is_invalid()).copied().collect()
    }
}
