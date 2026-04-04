use crate::domain::order::Order;
use crate::domain::phase::PhaseContext;
use crate::domain::phase::retreat_order_helper::RetreatOrderHelper;

pub struct RetreatAdjudicator;

impl RetreatAdjudicator {
    pub(crate) fn resolve_retreat_orders(orders: &mut [Order], context: &mut PhaseContext) {
        // 全ての撤退命令の移動先を取得
        let dest_codes = orders.collect_retreat_destination_code_set();
        for code in dest_codes {
            // 同じ移動先を指定している撤退命令を取得
            let conflict_idxs = orders.collect_unresolved_retreat_idxs();

            // 同じ移動先を指定している撤退命令が複数ある場合は全て撤退失敗
            if conflict_idxs.len() > 1 {
                for idx in conflict_idxs {
                    orders[idx].set_failure();
                }
                continue;
            }

            let retreat_idx = conflict_idxs[0];

            // 撤退先が攻撃元の場合は無効
            if orders[retreat_idx]
                .unit
                .dislodged_from
                .expect("should have a dislodged_from")
                .code()
                == code
            {
                orders[retreat_idx].set_invalid();
                continue;
            }

            // 撤退先先にユニットがいる場合は無効
            if context
                .last_resolved_units
                .iter()
                .find(|u| u.location().code() == code)
                .is_some()
            {
                orders[retreat_idx].set_invalid();
                continue;
            }

            // 撤退先がスタンドオフエリアの場合は無効
            if context.standoff_codes.iter().find(|&p| *p == &code[..3]).is_some() {
                orders[retreat_idx].set_invalid();
                continue;
            }

            // 上記すべてに該当しない場合のみ撤退成功
            orders[retreat_idx].set_success();
            continue;
        }
    }
}
