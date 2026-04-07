use crate::domain::order::OrderKind;
use crate::domain::phase::Phase;
use crate::domain::phase::main_adjudicator::MainAdjudicator;
use crate::domain::phase::main_order_helper::MainOrderHelper;
use crate::domain::unit::Unit;

/// メインフェイズの命令解決処理
pub fn resolve_orders_for_main_phase(current_phase: &mut Phase) {
    let orders = &mut current_phase.data.orders;
    let standoff_province_codes = &mut current_phase.data.standoff_province_codes;

    // # 01. 移動命令検証
    MainAdjudicator::validate_move_orders(orders);

    // # 02. 支援命令検証
    MainAdjudicator::validate_support_orders(orders);

    // # 03. 輸送命令検証
    MainAdjudicator::validate_convoy_orders(orders);

    // # 04. 支援命令のカット
    MainAdjudicator::handle_cutting_support_orders(orders);

    // # 05 . 輸送妨害の優先解決
    MainAdjudicator::handle_disruption_convoy_order(orders, standoff_province_codes);

    // # 06. 交換移動命令解決
    MainAdjudicator::handle_switch_orders(orders, standoff_province_codes);

    // # 07. 支援命令撃退の優先解決
    MainAdjudicator::handle_dislodging_support_orders(orders, standoff_province_codes);

    // # 08. 未解決移動命令解決
    MainAdjudicator::handle_remaining_move_orders(orders, standoff_province_codes);

    // # 09. 未処理の命令を全て成功判定
    MainAdjudicator::succeed_remaining_orders(orders);

    // # 10. 命令解決後のユニット配置情報をフェイズに反映
    apply_resolved_unit_locations(current_phase);
}

/// 命令解決後のユニット配置情報をフェイズに反映
fn apply_resolved_unit_locations(current_phase: &mut Phase) {
    for order in current_phase.data.orders.collect_not_invalid_orders() {
        // 移動に成功した軍の保存
        if let OrderKind::Move(m) = &order.kind
            && order.is_success()
        {
            current_phase.data.resolved_units.push(Unit {
                province: m.dest,
                ..order.unit
            });
            continue;
        }

        // 撃退された軍の保存
        if order.is_dislodged() {
            current_phase.data.resolved_units.push(Unit {
                dislodged_from: order.dislodged_from,
                ..order.unit
            });
            continue;
        }

        // それ以外の軍は現状維持
        current_phase.data.resolved_units.push(Unit { ..order.unit });
    }
}
