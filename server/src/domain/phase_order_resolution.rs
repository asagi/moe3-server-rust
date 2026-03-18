use super::order::Order;
use super::order::OrderKind;
use super::path::Path;
use super::phase::Phase;
use super::phase::PhaseContext;
use super::province::Province;
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
pub fn resolve_orders_for_order_phase(current_phase: &mut Phase, _context: &mut PhaseContext) {
    #[cfg(test)]
    {
        test_hook::mark_called();
    }

    // # 01. 移動命令検証
    validate_move_orders(current_phase.orders_mut());

    // # 02. 支援命令検証
    validate_support_orders(current_phase.orders_mut());

    // # 03. 輸送命令検証
    validate_convoy_orders(current_phase.orders_mut());

    // # 04. 支援命令のカット
    handle_cutting_support_orders(current_phase.orders_mut());

    // # 05 . 輸送妨害の優先解決
    handle_disruption_convoy_order(current_phase.orders_mut());

    // # 06. 交換移動命令解決
    handle_switch_orders(current_phase.orders_mut());

    // # 07. 未解決移動命令解決
    handle_remaining_move_orders(current_phase.orders_mut());

    // # 08. 未処理の命令を全て成功判定
    succeed_remaining_orders(current_phase.orders_mut());
}

/// 移動命令検証
fn validate_move_orders(original_orders: &mut [Order]) {
    let convoy_orders = collect_unresolved_convoy_orders(original_orders);

    for idx in collect_unresolved_move_indices(original_orders) {
        let move_order = &mut original_orders[idx];
        let OrderKind::Move(m) = &move_order.kind else { unreachable!("expected Move") };

        // 隣接経路が成立していれば有効
        if Path::can_unit_move_to(&move_order.unit, m.dest.code()) {
            move_order.set_valid();
            continue;
        }

        // 陸軍の遠隔移動は輸送経路が成立している場合のみ有効
        if let UnitKind::Army(_) = &move_order.unit.kind {
            let matched_convoy_orders: Vec<&Order> = convoy_orders.iter().filter(|o| o.is_matching_target(move_order)).collect();
            if can_move_via_convoy(move_order, &m.dest, &matched_convoy_orders) {
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
fn validate_support_orders(original_orders: &mut [Order]) {
    let orders = collect_not_invalid_orders(original_orders);

    for idx in collect_unresolved_support_indices(original_orders) {
        let support_order = &mut original_orders[idx];

        // 支援対象が存在しない場合は無効
        let Some(target) = orders.iter().find(|o| support_order.is_matching_target(o)) else {
            support_order.set_invalid();
            continue;
        };

        // 支援対象の移動命令の移動先に支援ユニットが移動できるなら有効
        if let OrderKind::Move(m) = &target.kind {
            if Path::can_unit_move_to(&support_order.unit, m.dest.code()) {
                support_order.set_valid();
                continue;
            }
            support_order.set_invalid();
            continue;
        }

        // 支援対象の非移動命令の現在地に支援ユニットが移動できるなら有効
        if Path::can_unit_move_to(&support_order.unit, target.location().code()) {
            support_order.set_valid();
            continue;
        }
        support_order.set_invalid();
        continue;
    }
}

/// 輸送命令検証
fn validate_convoy_orders(original_orders: &mut [Order]) {
    let move_orders = collect_valid_move_orders(original_orders);

    for idx in collect_unresolved_convoy_indices(original_orders) {
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
fn handle_cutting_support_orders(original_orders: &mut [Order]) {
    let convoy_orders = collect_valid_convoy_orders(original_orders);
    let move_orders = collect_valid_move_orders(original_orders);

    // 支援命令をカットし得る移動命令がなければ終了
    if !move_orders.iter().any(|m| m.is_valid()) {
        return;
    }

    for idx in collect_valid_support_indices(original_orders) {
        let support_order = &mut original_orders[idx];

        // support_order に向かう移動命令は攻撃とみなす（自国軍は除く）
        let attack_orders: Vec<&Order> = move_orders
            .iter()
            .filter(|o| o.power != support_order.power)
            .filter(|o| if let OrderKind::Move(m) = &o.kind { m.dest == support_order.location() } else { false })
            .collect();

        // 支援命令をカットし得る移動命令がなければスキップ
        if attack_orders.is_empty() {
            continue;
        }

        // 複数個所からの攻撃は即カット
        if attack_orders.len() > 1 {
            support_order.set_cut();
            continue;
        }
        let attack_order = attack_orders[0];

        // support_order が移動命令の支援でなければ攻撃を向けられた時点でカット
        let OrderKind::Support(s) = &support_order.kind else { unreachable!("expected Support") };
        let Some(support_target_dest) = &s.target_dest else {
            support_order.set_cut();
            continue;
        };

        // support_order の支援対象の移動先が attack_order ならカット回避
        if support_target_dest == &attack_order.location() {
            continue;
        }

        // attack_order が隣接地域からの場合はカット（遠隔攻撃の場合はさらに経路判定が必要）
        if Path::is_adjacent(attack_order.location().code(), support_order.location().code()) {
            support_order.set_cut();
            continue;
        }

        // 輸送経路が成立していなければ経路不成立でカット回避
        let matched_convoy_orders = collect_matched_convoy_orders(&convoy_orders, attack_order);
        if !can_move_via_convoy(attack_order, &support_order.location(), &matched_convoy_orders) {
            continue;
        }

        // 支援対象の移動先が輸送海軍でなければ経路寸断見込みなしでカット
        let Some(convoy_order_support_target_attacking) = &matched_convoy_orders.iter().find(|c| support_target_dest == &c.location()) else {
            support_order.set_cut();
            continue;
        };

        // 支援対象の移動先の輸送海軍の輸送対象が attack_order でなければカット回避とは無関係のためカット
        let OrderKind::Convoy(c) = &convoy_order_support_target_attacking.kind else {
            unreachable!("expected Convoy")
        };
        if c.target_dest != support_order.location() {
            support_order.set_cut();
            continue;
        }

        // 支援対象の移動先の輸送海軍を除去したと仮定しても輸送経路が寸断されなければカット
        let matched_convoy_orders_without_support_target: Vec<&Order> = matched_convoy_orders
            .iter()
            .filter(|o| o.location() != convoy_order_support_target_attacking.location())
            .copied()
            .collect();
        if can_move_via_convoy(attack_order, &support_order.location(), &matched_convoy_orders_without_support_target) {
            support_order.set_cut();
            continue;
        }
    }
}

/// 輸送妨害の優先解決
fn handle_disruption_convoy_order(_orders: &mut [Order]) {}

/// 交換移動命令解決
fn handle_switch_orders(_orders: &mut [Order]) {}

/// 未解決移動命令解決
fn handle_remaining_move_orders(_orders: &mut [Order]) {}

/// 未処理の命令を全て成功判定
fn succeed_remaining_orders(_orders: &mut [Order]) {}

/// 全ての命令のコレクションを作成
fn collect_not_invalid_orders(orders: &[Order]) -> Vec<Order> {
    orders.iter().filter(|o| !o.is_assumed() && !o.is_invalid()).copied().collect()
}

/// 有効な移動命令のコレクションを作成
fn collect_valid_move_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Move(_)))
        .copied()
        .collect()
}

/// 未処理の移動命令のインデックスコレクションを作成
fn collect_unresolved_move_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Move(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 未処理の支援命令のインデックスコレクションを作成
fn collect_unresolved_support_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Support(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 有効な支援命令のインデックスコレクションを作成
fn collect_valid_support_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Support(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 全ての輸送命令のコレクションを作成
fn collect_unresolved_convoy_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Convoy(_)))
        .copied()
        .collect()
}

/// 有効な輸送命令のコレクションを作成
fn collect_valid_convoy_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Convoy(_)))
        .copied()
        .collect()
}

/// attack_order にマッチする輸送命令のコレクションを作成
fn collect_matched_convoy_orders<'a>(convoy_orders: &'a [Order], attack_order: &Order) -> Vec<&'a Order> {
    convoy_orders
        .iter()
        .filter(|o| matches!(o.kind, OrderKind::Convoy(_)) && o.is_matching_target(attack_order))
        .collect()
}

/// 未処理の輸送命令のインデックスコレクションを作成
fn collect_unresolved_convoy_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Convoy(_)))
        .map(|(i, _)| i)
        .collect()
}

/// 輸送経路が成立しているかどうかを判定
fn can_move_via_convoy(move_order: &Order, dest: &Province, matched_convoy_orders: &Vec<&Order>) -> bool {
    let allowed_waters: HashSet<&str> = matched_convoy_orders.iter().map(|order| order.location().code()).collect();
    Path::is_reachable_by_sea(move_order.location().code(), dest.code(), &allowed_waters)
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
