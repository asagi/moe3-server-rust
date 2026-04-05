use crate::domain::order::Order;
use crate::domain::phase::PhaseContext;
use crate::domain::phase::retreat_order_helper::RetreatOrderHelper;

pub struct RetreatAdjudicator;

impl RetreatAdjudicator {
    /// 撤退命令の妥当性を検査
    pub(crate) fn validate_retreat_orders(orders: &mut [Order], context: &mut PhaseContext) {
        for &dest_code in orders.collect_retreat_destination_code_set().iter() {
            // 指定地域に対する未処理の撤退命令のインデックスを取得
            let conflict_idxs: Vec<usize> = orders.collect_unresolved_retreat_idxs_by_dest(dest_code);

            // 撤退先先にユニットがいる場合は無効
            if context
                .last_resolved_units
                .iter()
                .find(|u| u.location().code() == dest_code)
                .is_some()
            {
                for idx in conflict_idxs {
                    orders[idx].set_invalid();
                }
                continue;
            }

            // 撤退先がスタンドオフエリアの場合は無効
            if context.standoff_codes.iter().find(|&&c| c == &dest_code[..3]).is_some() {
                for idx in conflict_idxs {
                    orders[idx].set_invalid();
                }
                continue;
            }

            // 撤退先が攻撃元の場合は無効
            if let Some(&idx) = conflict_idxs
                .iter()
                .find(|&&idx| orders[idx].unit.dislodged_from.expect("should have a dislodged_from").code() == dest_code)
            {
                orders[idx].set_invalid()
            }
        }
    }

    /// 撤退命令の解決
    pub(crate) fn handle_retreat_orders(orders: &mut [Order]) {
        for &dest_code in orders.collect_retreat_destination_code_set().iter() {
            // 指定地域に対する未処理の撤退命令のインデックスを取得
            let conflict_idxs: Vec<usize> = orders.collect_unresolved_retreat_idxs_by_dest(dest_code);

            if conflict_idxs.is_empty() {
                // 指定地域に対する有効な撤退命令がなければ終了
                continue;
            }

            if conflict_idxs.len() > 1 {
                // 撤退先が重複した場合は全て撤退失敗
                for idx in conflict_idxs {
                    orders[idx].set_failure();
                }
                continue;
            }

            // 撤退成功
            let retreat_idx = conflict_idxs[0];
            orders[retreat_idx].set_success();
        }
    }
}
