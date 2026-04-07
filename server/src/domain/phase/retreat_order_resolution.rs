use crate::domain::order::OrderKind;
use crate::domain::phase::Phase;
use crate::domain::phase::PhaseContext;
use crate::domain::phase::retreat_adjudicator::RetreatAdjudicator;
use crate::domain::phase::retreat_order_helper::RetreatOrderHelper;
use crate::domain::unit::Unit;

/// 撤退フェイズの命令解決処理
pub fn resolve_orders_for_retreat_phase(current_phase: &mut Phase, context: &PhaseContext) {
    let orders = &mut current_phase.data.orders;

    // # 01. 撤退命令検証
    RetreatAdjudicator::validate_retreat_orders(orders, context);

    // # 02. 撤退命令処理
    RetreatAdjudicator::handle_retreat_orders(orders);

    // # 03. 命令解決後のユニット配置情報をフェイズに反映
    apply_resolved_unit_locations(current_phase, context);
}

/// 命令解決後のユニット配置情報をフェイズに反映
fn apply_resolved_unit_locations(current_phase: &mut Phase, context: &PhaseContext) {
    for order in current_phase.data.orders.collect_not_invalid_orders() {
        // 撤退に成功したユニットのみ保存
        if let OrderKind::Retreat(r) = &order.kind
            && order.is_success()
        {
            current_phase.data.resolved_units.push(Unit {
                province: r.dest,
                ..order.unit
            });
            continue;
        }

        // 撤退フェイズの影響を受けないユニットは現状維持
        for unit in context.last_resolved_units.clone() {
            if current_phase.data.orders.iter().find(|o| o.unit == unit).is_some() {
                continue;
            }
            current_phase.data.resolved_units.push(Unit { ..unit });
        }
    }
}
