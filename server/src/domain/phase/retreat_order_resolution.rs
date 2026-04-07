use crate::domain::phase::Phase;
use crate::domain::phase::PhaseContext;
use crate::domain::phase::retreat_adjudicator::RetreatAdjudicator;

/// 撤退フェイズの命令解決処理
pub fn resolve_orders_for_retreat_phase(current_phase: &mut Phase, context: &mut PhaseContext) {
    let orders = &mut current_phase.data.orders;

    // # 01. 撤退命令検証
    RetreatAdjudicator::validate_retreat_orders(orders, context);

    // # 02. 撤退命令処理
    RetreatAdjudicator::handle_retreat_orders(orders);
}
