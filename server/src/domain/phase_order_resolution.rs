use super::order::Order;
use super::order::OrderKind;
use super::order::OrderStatus;
use super::path::Path;
use super::phase::Phase;
use super::phase::PhaseContext;
use super::unit::Unit;
use std::collections::HashSet;

/// 命令フェイズの命令解決処理
///
/// 方針（暫定）:
/// - 現在は戻り値を持たず、`current_phase` への副作用で結果を反映する。
/// - 入力の主対象は `current_phase.orders`。
/// - 解決結果は `current_phase.units` や（将来的に）スタンドオフ情報へ書き戻す。
/// - `context` は参照用（過去フェイズ参照など）を基本とし、不要な更新は避ける。
/// - I/O は行わず、同じ入力に対して同じ結果になる決定的な処理を維持する。
pub fn resolve_orders_for_order_phase(current_phase: &mut Phase, context: &mut PhaseContext) {
    #[cfg(test)]
    {
        test_hook::mark_called();
    }

    let orders = &mut current_phase.orders;

    // # 01. 移動命令検証
    validate_move_orders(orders, context);

    // # 02. 支援命令検証
    validate_support_orders(orders, context);

    // # 03. 輸送命令検証
    validate_convoy_orders(orders, context);

    // # 04. 支援命令のカット
    handle_cutting_support_orders(orders, context);

    // # 05 . 輸送妨害の優先解決
    handle_disruption_convoy_order(orders, context);

    // # 06. 交換移動命令解決
    handle_switch_orders(orders, context);

    // # 07. 未解決移動命令解決
    handle_remaining_move_orders(orders, context);

    // # 08. 未処理の命令を全て成功判定
    succeed_remaining_orders(orders, context);
}

/// 移動命令検証
fn validate_move_orders(orders: &mut [Order], _context: &PhaseContext) {
    let convoy_orders: Vec<Order> = orders.iter().filter(|o| matches!(o.kind, OrderKind::Convoy(_))).cloned().collect();
    let move_orders: Vec<&mut Order> = orders.iter_mut().filter(|o| matches!(o.kind, OrderKind::Move(_))).collect();

    for move_order in move_orders {
        let OrderKind::Move(ref m) = move_order.kind else { continue };

        match &move_order.unit {
            Unit::Fleet(_) => {
                if !Path::can_fleet_move(move_order.location().code(), m.dest.code()) {
                    move_order.set_invalid();
                    continue;
                }
            }
            Unit::Army(_) => {
                if m.dest.is_water() {
                    move_order.set_invalid();
                    continue;
                }

                if Path::is_adjacent(move_order.location().code(), m.dest.code()) {
                    continue;
                }

                // 輸送経路の成立していない陸軍の遠隔地移動は失敗
                let effective_convoy_orders: Vec<&Order> = convoy_orders.iter().filter(|o| is_effective_convoy(o, move_order)).collect();
                let allowed_waters: HashSet<&str> = effective_convoy_orders.iter().map(|order| order.location().code()).collect();
                if !Path::is_reachable_by_sea(move_order.location().code(), m.dest.code(), &allowed_waters) {
                    move_order.set_invalid();
                    continue;
                }
            }
        }
    }
}

/// 支援命令検証
fn validate_support_orders(_orders: &mut [Order], _context: &PhaseContext) {}

/// 輸送命令検証
fn validate_convoy_orders(_orders: &mut [Order], _context: &PhaseContext) {}

/// 支援命令のカット
fn handle_cutting_support_orders(_orders: &mut [Order], _context: &PhaseContext) {}

/// 輸送妨害の優先解決
fn handle_disruption_convoy_order(_orders: &mut [Order], _context: &PhaseContext) {}

/// 交換移動命令解決
fn handle_switch_orders(_orders: &mut [Order], _context: &PhaseContext) {}

/// 未解決移動命令解決
fn handle_remaining_move_orders(_orders: &mut [Order], _context: &PhaseContext) {}

/// 未処理の命令を全て成功判定
fn succeed_remaining_orders(_orders: &mut [Order], _context: &PhaseContext) {}

fn is_effective_convoy(convoy_order: &Order, target: &Order) -> bool {
    let OrderKind::Convoy(convoy) = &convoy_order.kind else { return false };

    if convoy.target_unit != target.unit {
        return false;
    }
    if !matches!(&target.kind, OrderKind::Move(m) if convoy.target_dest == m.dest) {
        return false;
    }
    if convoy_order.status == OrderStatus::Dislodged {
        return false;
    }
    true
}

#[cfg(test)]
pub(crate) mod test_hook {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    pub(crate) fn reset() {
        CALL_COUNT.store(0, Ordering::SeqCst);
    }

    pub(crate) fn mark_called() {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn call_count() -> usize {
        CALL_COUNT.load(Ordering::SeqCst)
    }
}
