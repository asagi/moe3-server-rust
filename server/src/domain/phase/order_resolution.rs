use crate::domain::order::Order;
use crate::domain::order::OrderKind;
use crate::domain::phase::Phase;
use crate::domain::phase::adjudicator::Adjudicator;
use crate::domain::phase::order_helper::OrderHelper;
use crate::domain::unit::Unit;

/// 命令フェイズの命令解決処理
pub fn resolve_orders_for_order_phase(current_phase: &mut Phase) {
    let orders = &mut current_phase.data.orders;
    let standoff_province_codes = &mut current_phase.data.standoff_province_codes;
    let resolved_units = &mut current_phase.data.resolved_units;

    // # 01. 移動命令検証
    Adjudicator::validate_move_orders(orders);

    // # 02. 支援命令検証
    Adjudicator::validate_support_orders(orders);

    // # 03. 輸送命令検証
    Adjudicator::validate_convoy_orders(orders);

    // # 04. 支援命令のカット
    Adjudicator::handle_cutting_support_orders(orders);

    // # 05 . 輸送妨害の優先解決
    Adjudicator::handle_disruption_convoy_order(orders, standoff_province_codes);

    // # 06. 交換移動命令解決
    Adjudicator::handle_switch_orders(orders, standoff_province_codes);

    // # 07. 支援命令撃退の優先解決
    Adjudicator::handle_dislodging_support_orders(orders, standoff_province_codes);

    // # 08. 未解決移動命令解決
    Adjudicator::handle_remaining_move_orders(orders, standoff_province_codes);

    // # 09. 未処理の命令を全て成功判定
    Adjudicator::succeed_remaining_orders(orders);

    // # 10. 命令解決後のユニット配置情報をフェイズに反映
    apply_resolved_unit_locations(orders, resolved_units);
}

/// 命令解決後のユニット配置情報をフェイズに反映
fn apply_resolved_unit_locations(orders: &[Order], resolved_units: &mut Vec<Unit>) {
    for order in orders.collect_not_invalid_orders() {
        // 移動に成功した軍の保存
        if let OrderKind::Move(m) = &order.kind
            && order.is_success()
        {
            resolved_units.push(Unit {
                province: m.dest,
                ..order.unit
            });
            continue;
        }

        // 撃退された軍の保存
        if order.is_dislodged() {
            resolved_units.push(Unit {
                province: order
                    .dislodged_from
                    .expect("dislodged_from should be set if is_dislodged is true"),
                ..order.unit
            });
            continue;
        }

        // それ以外の軍は現状維持
        resolved_units.push(Unit { ..order.unit });
    }
}
