use super::order::Order;
use super::order::OrderKind;
use super::path::Path;
use super::phase::Phase;
use super::phase::PhaseContext;
use super::unit::UnitKind;
use std::collections::HashSet;

/// 命令フェイズの命令解決処理
///
/// 方針（暫定）:
/// - 現在は戻り値を持たず、`current_phase` への副作用で結果を反映する。
/// - 入力の主対象は `current_phase.data.orders`。
/// - 解決結果は `current_phase.units` や（将来的に）スタンドオフ情報へ書き戻す。
/// - `context` は参照用（過去フェイズ参照など）を基本とし、不要な更新は避ける。
/// - I/O は行わず、同じ入力に対して同じ結果になる決定的な処理を維持する。
pub fn resolve_orders_for_order_phase(current_phase: &mut Phase, context: &mut PhaseContext) {
    #[cfg(test)]
    {
        test_hook::mark_called();
    }

    // # 01. 移動命令検証
    validate_move_orders(current_phase.orders_mut(), context);

    // # 02. 支援命令検証
    validate_support_orders(current_phase.orders_mut(), context);

    // # 03. 輸送命令検証
    validate_convoy_orders(current_phase.orders_mut(), context);

    // # 04. 支援命令のカット
    handle_cutting_support_orders(current_phase.orders_mut(), context);

    // # 05 . 輸送妨害の優先解決
    handle_disruption_convoy_order(current_phase.orders_mut(), context);

    // # 06. 交換移動命令解決
    handle_switch_orders(current_phase.orders_mut(), context);

    // # 07. 未解決移動命令解決
    handle_remaining_move_orders(current_phase.orders_mut(), context);

    // # 08. 未処理の命令を全て成功判定
    succeed_remaining_orders(current_phase.orders_mut(), context);
}

/// 移動命令検証
fn validate_move_orders(original_orders: &mut [Order], _context: &PhaseContext) {
    let convoy_orders = collect_convoy_orders(original_orders);

    for idx in collect_move_indices(original_orders) {
        let move_order = &mut original_orders[idx];
        let OrderKind::Move(ref m) = move_order.kind else { unreachable!("expected Move") };

        // 隣接経路が成立していれば有効
        if Path::can_unit_move_to(move_order.unit, m.dest.code()) {
            move_order.set_valid();
            continue;
        }

        // 陸軍の遠隔移動は輸送経路が成立している場合のみ有効
        if let UnitKind::Army(_) = move_order.unit.kind {
            let matched_convoy_orders: Vec<&Order> = convoy_orders.iter().filter(|o| o.is_matching_target(move_order)).collect();
            let allowed_waters: HashSet<&str> = matched_convoy_orders.iter().map(|order| order.location().code()).collect();
            if Path::is_reachable_by_sea(move_order.location().code(), m.dest.code(), &allowed_waters) {
                move_order.set_valid();
                continue;
            } else {
                move_order.set_invalid();
                continue;
            }
        }
    }
}

/// 支援命令検証
fn validate_support_orders(original_orders: &mut [Order], _context: &PhaseContext) {
    let orders = collect_orders(original_orders);

    for idx in collect_support_indices(original_orders) {
        let support_order = &mut original_orders[idx];

        // 支援対象が存在しない場合は無効
        let Some(target) = orders.iter().find(|o| support_order.is_matching_target(o)) else {
            support_order.set_invalid();
            continue;
        };

        // 支援対象の移動命令の移動先に支援ユニットが移動できるなら有効
        if let OrderKind::Move(m) = &target.kind {
            if Path::can_unit_move_to(support_order.unit, m.dest.code()) {
                support_order.set_valid();
                continue;
            }
            support_order.set_invalid();
            continue;
        }

        // 支援対象の非移動命令の現在地に支援ユニットが移動できるなら有効
        if Path::can_unit_move_to(support_order.unit, target.location().code()) {
            support_order.set_valid();
            continue;
        }
        support_order.set_invalid();
        continue;
    }
}

/// 輸送命令検証
fn validate_convoy_orders(original_orders: &mut [Order], _context: &PhaseContext) {
    let move_orders = collect_move_orders(original_orders);

    for idx in collect_convoy_indices(original_orders) {
        let convoy_order = &mut original_orders[idx];

        // 水上にない艦への輸送命令は無効
        if !convoy_order.location().is_water() {
            convoy_order.set_invalid();
            continue;
        }

        // 輸送対象移動命令が存在すれば有効、なければ無効
        if move_orders.iter().any(|m| convoy_order.is_matching_target(m)) {
            convoy_order.set_valid();
            continue;
        }
        convoy_order.set_invalid();
        continue;
    }
}
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

/// 有効な命令のコレクションを作成
fn collect_orders(orders: &[Order]) -> Vec<Order> {
    orders.iter().filter(|o| !o.is_virtual() && !o.is_invalid()).copied().collect()
}

/// 有効な輸送命令のコレクションを作成
fn collect_convoy_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_virtual() && !o.is_invalid() && matches!(o.kind, OrderKind::Convoy(_)))
        .copied()
        .collect()
}

/// 有効な移動命令のコレクションを作成
fn collect_move_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_virtual() && !o.is_invalid() && matches!(o.kind, OrderKind::Move(_)))
        .copied()
        .collect()
}

/// 有効な移動命令のインデックスコレクションを作成
fn collect_move_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_virtual() && !o.is_invalid() && matches!(o.kind, OrderKind::Move(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 有効な支援命令のインデックスコレクションを作成
fn collect_support_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_virtual() && !o.is_invalid() && matches!(o.kind, OrderKind::Support(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 有効な輸送命令のインデックスコレクションを作成
fn collect_convoy_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_virtual() && !o.is_invalid() && matches!(o.kind, OrderKind::Convoy(_)))
        .map(|(i, _)| i)
        .collect()
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
