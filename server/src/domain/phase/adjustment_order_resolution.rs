use super::super::order::OrderKind;
use super::super::phase::Phase;
use super::super::phase::adjustment_adjudicator::AdjustmentAdjudicator;
use super::super::phase::adjustment_order_helper::AdjustmentOrderHelper;

/// 調整フェイズの命令解決処理
pub fn resolve_orders_for_adjustment_phase(current_phase: &mut Phase) {
    // # 01. 増設命令検証
    AdjustmentAdjudicator::validate_build_orders(current_phase);

    // # 02. 解体命令検証
    AdjustmentAdjudicator::validate_disband_orders(current_phase);

    // # 03. 未処理命令をすべて無効判定
    AdjustmentAdjudicator::invalidate_unresolved_orders(current_phase);

    // # 04. 命令解決後のユニット配置情報をフェイズに反映
    apply_resolved_unit_locations(current_phase);
}

/// 命令解決後のユニット配置情報をフェイズに反映
fn apply_resolved_unit_locations(current_phase: &mut Phase) {
    for idx in current_phase.data.orders.collect_valid_idxs() {
        match current_phase.data.orders[idx].kind {
            OrderKind::Build(_) => {
                // 増設命令が有効な場合はユニットを追加
                current_phase.data.units.push(current_phase.data.orders[idx].unit);
            }
            OrderKind::Disband(_) => {
                // 解体命令が有効な場合はユニットを削除
                current_phase.data.units.retain(|u| u != &current_phase.data.orders[idx].unit);
            }
            _ => {}
        }
    }
}
