use super::super::super::adjudicator::adjustment_adjudicator::AdjustmentAdjudicator;
use super::super::super::adjudicator::main_adjudicator::MainAdjudicator;
use super::super::super::adjudicator::retreat_adjudicator::RetreatAdjudicator;
use super::super::super::helper::adjustment_order_helper::AdjustmentOrderHelper;
use super::super::super::helper::main_order_helper::MainOrderHelper;
use super::super::super::helper::retreat_order_helper::RetreatOrderHelper;
use super::super::order::OrderKind;
use super::Phase;
use super::Unit;

/// メインフェイズの命令解決処理
pub fn resolve_orders_for_main_phase(current_phase: &mut Phase) {
    let orders = &mut current_phase.data.orders;
    let standoff_province_codes = &mut current_phase.data.standoff_codes;

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
    for order in orders.collect_not_invalid_main_orders() {
        // 移動に成功した軍を更新
        if let OrderKind::Move(m) = &order.kind
            && order.is_success()
        {
            // 対象ユニットを削除
            if let Some(idx) = current_phase.data.units.iter().position(|u| u == &order.unit) {
                current_phase.data.units.remove(idx);
            }

            // 所在地を移動先に変更して保存
            current_phase.data.units.push(Unit {
                province: m.dest,
                ..order.unit
            });
            continue;
        }

        // 撃退された軍に攻撃元情報を記録
        if !order.is_dislodged() {
            continue;
        }
        let Some(dislodged_unit) = current_phase
            .data
            .units
            .iter_mut()
            .find(|u| u.power == order.unit.power && u.kind == order.unit.kind && u.province == order.unit.province)
        else {
            continue;
        };
        if let Some(dislodged_from) = order.unit.dislodged_from {
            dislodged_unit.dislodged_from(dislodged_from);
        } else {
            dislodged_unit.dislodged_via_convoy();
        }
    }
}

/// 撤退フェイズの命令解決処理
pub fn resolve_orders_for_retreat_phase(current_phase: &mut Phase) {
    let orders = &mut current_phase.data.orders;
    let units = &current_phase.data.units;
    let standoff_codes = &current_phase.data.standoff_codes;

    // # 01. 撤退命令検証
    RetreatAdjudicator::validate_retreat_orders(orders, units, standoff_codes);

    // # 02. 撤退命令処理
    RetreatAdjudicator::handle_retreat_orders(orders);

    // # 03. 命令解決後のユニット配置情報をフェイズに反映
    for order in orders.collect_not_assumed_retreats() {
        // 撤退フェイズでの処理対象の軍をいったん削除
        if matches!(&order.kind, OrderKind::Retreat(_) | OrderKind::Disband(_)) {
            if let Some(idx) = current_phase.data.units.iter().position(|u| u == &order.unit) {
                current_phase.data.units.remove(idx);
            }
        }

        if let OrderKind::Retreat(r) = &order.kind
            && order.is_success()
        {
            // 撤退に成功した軍のみ所在を移動先に変更して再配置
            current_phase.data.units.push(Unit {
                province: r.dest,
                dislodged_from: None,
                dislodged: false,
                ..order.unit
            });
        }
    }
}

/// 調整フェイズの命令解決処理
pub fn resolve_orders_for_adjustment_phase(current_phase: &mut Phase) {
    // # 01. 増設命令検証
    AdjustmentAdjudicator::validate_build_orders(current_phase);

    // # 02. 解体命令検証
    AdjustmentAdjudicator::validate_disband_orders(current_phase);

    // # 03. 未処理命令をすべて無効判定
    AdjustmentAdjudicator::invalidate_unresolved_orders(current_phase);

    // # 04. 命令解決後のユニット配置情報をフェイズに反映
    for idx in current_phase.data.orders.collect_valid_adjustment_idxs() {
        match current_phase.data.orders[idx].kind {
            OrderKind::Build(_) => {
                // 増設命令が有効な場合はユニットを追加
                current_phase.data.units.push(current_phase.data.orders[idx].unit);
            }
            OrderKind::Disband(_) => {
                // 解体命令が有効な場合はユニットを削除
                if let Some(idx) = current_phase
                    .data
                    .units
                    .iter()
                    .position(|u| u == &current_phase.data.orders[idx].unit)
                {
                    current_phase.data.units.remove(idx);
                }
            }
            _ => {}
        }
    }
}
