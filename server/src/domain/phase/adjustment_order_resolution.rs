use crate::domain::phase::Phase;
use crate::domain::phase::PhaseContext;
use crate::domain::phase::adjustment_adjudicator::AdjustmentAdjudicator;

/// 撤退フェイズの命令解決処理
pub fn resolve_orders_for_adjustment_phase(current_phase: &mut Phase, context: &PhaseContext) {
    let orders = &mut current_phase.data.orders;

    // # 01. 増設命令検証
    AdjustmentAdjudicator::validate_build_orders(orders, context);

    // # 02. 解体命令検証
    AdjustmentAdjudicator::validate_disband_orders(orders);

    // # 03. 命令解決後のユニット配置情報をフェイズに反映
    apply_resolved_unit_locations(current_phase, context);
}

fn apply_resolved_unit_locations(_current_phase: &mut Phase, _context: &PhaseContext) {
    // TODO
}
