use crate::domain::order::Order;
use crate::domain::order::OrderKind;
use crate::domain::path::Path;
use crate::domain::phase::Phase;
use crate::domain::phase::PhaseContext;
use crate::domain::province::Province;
use crate::domain::unit::UnitKind;
use std::cmp::Ordering;
use std::collections::HashSet;

/// 命令フェイズの命令解決処理
pub fn resolve_orders_for_order_phase(current_phase: &mut Phase, context: &mut PhaseContext) {
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
    handle_disruption_convoy_order(current_phase.orders_mut(), context);

    // # 06. 交換移動命令解決
    handle_switch_orders(current_phase.orders_mut(), context);

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
fn handle_disruption_convoy_order(original_orders: &mut [Order], context: &mut PhaseContext) {
    let support_orders = collect_valid_support_orders(original_orders);

    for convoy_order_idx in collect_valid_convoy_indices(original_orders) {
        // 輸送命令に対する攻撃競争の勝者を取得
        let Some(winner_idx) = handle_conflicting(original_orders, &original_orders[convoy_order_idx].location(), &mut context.standoff_provinces) else {
            // 勝者がいなければスキップ
            continue;
        };

        if original_orders[winner_idx].power == original_orders[convoy_order_idx].power {
            // 勝者が自国軍であればその移動は無条件失敗となりスキップ
            original_orders[winner_idx].set_failure();
            continue;
        }

        // 戦闘解決
        let convoy_supports_count = support_orders.iter().filter(|o| o.is_matching_target(&original_orders[convoy_order_idx])).count();
        let winner_supports_count = support_orders
            .iter()
            .filter(|o| o.is_matching_target(&original_orders[winner_idx]) && o.power != original_orders[convoy_order_idx].power)
            .count();
        if convoy_supports_count >= winner_supports_count {
            // 攻撃失敗
            original_orders[winner_idx].set_failure();
            continue;
        }

        // 輸送敗退
        original_orders[winner_idx].set_success();
        original_orders[convoy_order_idx].set_dislodged_from(&original_orders[winner_idx].location());

        // 輸送路切断判定
        let convoy_orders = collect_valid_convoy_orders(original_orders);
        let Some(move_order_idx) = original_orders.iter().position(|o| original_orders[convoy_order_idx].is_matching_target(o)) else {
            unreachable!("move order matching convoy order should exist");
        };
        let OrderKind::Move(m) = &original_orders[move_order_idx].kind else {
            unreachable!("expected Move")
        };
        let matched_convoy_orders = collect_matched_convoy_orders(&convoy_orders, &original_orders[move_order_idx]);
        if !can_move_via_convoy(&original_orders[move_order_idx], &m.dest, &matched_convoy_orders) {
            original_orders[move_order_idx].set_failure();
            continue;
        }
    }
}
/// 交換移動命令解決
fn handle_switch_orders(original_orders: &mut [Order], context: &mut PhaseContext) {
    let move_order_indices = collect_valid_move_indices(original_orders);
    if move_order_indices.len() < 2 {
        // 移動命令が 2 つ以上なければ終了
        return;
    }

    for idx in move_order_indices {
        // 過去のループで対向の判定時に同時に処理済みであればスキップ
        // - 以下 original_orders[idx] を甲軍とする
        if !original_orders[idx].is_unresolved() {
            continue;
        }
        let OrderKind::Move(m) = &original_orders[idx].kind else {
            unreachable!("expected Move")
        };

        // 対向する移動命令がなければスキップ
        // - 以下 original_orders[opposite_move_order_idx] を乙軍とする
        let opposite_move_order_idx = original_orders.iter().position(|o| {
            o != &original_orders[idx]
                && if let OrderKind::Move(om) = &o.kind {
                    o.location() == m.dest && original_orders[idx].location() == om.dest
                } else {
                    false
                }
        });
        let Some(opposite_idx) = opposite_move_order_idx else {
            continue;
        };

        // スタンドオフ判定
        let conflict_winner_idx = handle_conflicting(original_orders, &original_orders[opposite_idx].location(), &mut context.standoff_provinces);
        let opposite_conflict_winner_idx = handle_conflicting(original_orders, &original_orders[idx].location(), &mut context.standoff_provinces);
        if conflict_winner_idx.is_none() && opposite_conflict_winner_idx.is_none() {
            // 両地域スタンドオフで関連する全軍移動失敗
            continue;
        }

        if conflict_winner_idx == Some(idx) && opposite_conflict_winner_idx != Some(opposite_idx) {
            // 甲軍進軍成功かつ乙軍進軍失敗からの乙軍の防衛成否判定
            resolve_no_support_defense(original_orders, idx, opposite_idx, opposite_conflict_winner_idx);
            continue;
        } else if conflict_winner_idx != Some(idx) && opposite_conflict_winner_idx == Some(opposite_idx) {
            // 甲軍進軍失敗かつ乙軍進軍成功からの甲軍の防衛成否判定
            resolve_no_support_defense(original_orders, opposite_idx, idx, conflict_winner_idx);
            continue;
        } else if conflict_winner_idx != Some(idx) && opposite_conflict_winner_idx != Some(opposite_idx) {
            if let Some(conflict_winner_idx) = conflict_winner_idx {
                // 甲乙両軍進軍失敗からの乙軍の防衛成否判定
                resolve_no_support_defense(original_orders, conflict_winner_idx, opposite_idx, None);
            }
            if let Some(opposite_conflict_winner_idx) = opposite_conflict_winner_idx {
                // 甲乙両軍進軍失敗からの甲軍の防衛成否判定
                resolve_no_support_defense(original_orders, opposite_conflict_winner_idx, idx, None);
            }
            continue;
        }

        debug_assert!(conflict_winner_idx == Some(idx) && opposite_conflict_winner_idx == Some(opposite_idx));

        // 海路迂回交換移動判定
        let a_can = can_reach_via_convoy(original_orders, idx);
        let b_can = can_reach_via_convoy(original_orders, opposite_idx);
        if !a_can && !b_can {
            // 双方とも海路迂回移動不可なら双方移動失敗
            original_orders[idx].set_failure();
            original_orders[opposite_idx].set_failure();
            continue;
        }
        if a_can && b_can {
            // 双方とも海路迂回移動不可なら双方移動成功
            original_orders[idx].set_success();
            original_orders[opposite_idx].set_success();
            continue;
        }
        if a_can || b_can {
            // 隣接地に限りどちらか一方が海路迂回移動可能なら双方移動成功
            if Path::is_adjacent(original_orders[idx].location().code(), original_orders[opposite_idx].location().code()) {
                original_orders[idx].set_success();
                original_orders[opposite_idx].set_success();
                continue;
            } else {
                original_orders[idx].set_failure();
                original_orders[opposite_idx].set_failure();
                continue;
            }
        }

        // 直接対決
        match decide_winner_by_supports(original_orders, idx, opposite_idx) {
            Some(winner_idx) => {
                let loser_idx = if winner_idx == idx { opposite_idx } else { idx };
                original_orders[winner_idx].set_success();
                original_orders[loser_idx].set_dislodged_from(&original_orders[winner_idx].location());
                continue;
            }
            None => {
                original_orders[idx].set_failure();
                original_orders[opposite_idx].set_failure();
                continue;
            }
        }
    }
}

/// 未解決移動命令解決
fn handle_remaining_move_orders(_orders: &mut [Order]) {}

/// 未処理の命令を全て成功判定
fn succeed_remaining_orders(_orders: &mut [Order]) {}

/// 全ての命令のコレクションを作成
fn collect_not_invalid_orders(orders: &[Order]) -> Vec<Order> {
    orders.iter().filter(|o| !o.is_assumed() && !o.is_invalid()).copied().collect()
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

/// 有効な移動命令のコレクションを作成
fn collect_valid_move_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Move(_)))
        .copied()
        .collect()
}

/// 有効な移動命令のインデックスコレクションを作成
fn collect_valid_move_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Move(_)))
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

/// 有効な支援命令のコレクションを作成
fn collect_valid_support_orders(orders: &[Order]) -> Vec<Order> {
    orders
        .iter()
        .filter(|o| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Support(_)))
        .copied()
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

/// 未処理の輸送命令のインデックスコレクションを作成
fn collect_unresolved_convoy_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_unresolved() && matches!(o.kind, OrderKind::Convoy(_)))
        .map(|(i, _)| i)
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

/// 有効な輸送命令のインデックスコレクションを作成
fn collect_valid_convoy_indices(orders: &[Order]) -> Vec<usize> {
    orders
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.is_assumed() && o.is_valid() && matches!(o.kind, OrderKind::Convoy(_)))
        .map(|(i, _)| i)
        .collect()
}

/// attack_order にマッチする輸送命令のコレクションを作成
fn collect_matched_convoy_orders<'a>(convoy_orders: &'a [Order], attack_order: &Order) -> Vec<&'a Order> {
    convoy_orders
        .iter()
        .filter(|o| matches!(o.kind, OrderKind::Convoy(_)) && o.is_matching_target(attack_order))
        .collect()
}

/// 輸送経路が成立しているかどうかを判定
fn can_move_via_convoy(move_order: &Order, dest: &Province, matched_convoy_orders: &Vec<&Order>) -> bool {
    let allowed_waters: HashSet<&str> = matched_convoy_orders.iter().map(|order| order.location().code()).collect();
    Path::is_reachable_by_sea(move_order.location().code(), dest.code(), &allowed_waters)
}

/// 戦闘解決
fn handle_conflicting(original_orders: &mut [Order], target_location: &Province, standoff_provinces: &mut Vec<Province>) -> Option<usize> {
    let support_orders = collect_valid_support_orders(original_orders);
    let conflicting_move_indicies: Vec<usize> = collect_valid_move_indices(original_orders)
        .into_iter()
        .filter(|&idx| {
            if let OrderKind::Move(m) = original_orders[idx].kind {
                m.dest == *target_location
            } else {
                false
            }
        })
        .collect();

    // 移動命令がなければ勝者なしで終了
    if conflicting_move_indicies.is_empty() {
        return None;
    }

    // 移動命令が 1 つなら即勝者確定で終了
    if conflicting_move_indicies.len() == 1 {
        let winner_idx = conflicting_move_indicies[0];
        return Some(winner_idx);
    }

    // 支援数集計
    // - support_counts: (move_order の index, 支援数) の配列
    // - support_counts は 支援数降順（戦力順）にソートする
    let mut support_counts: Vec<(usize, usize)> = conflicting_move_indicies
        .iter()
        .map(|&idx| (idx, support_orders.iter().filter(|s| s.is_matching_target(&original_orders[idx])).count()))
        .collect();
    support_counts.sort_by(|a, b| b.1.cmp(&a.1));

    // 戦闘解決： 単独勝利以外は移動失敗
    if support_counts.iter().filter(|(_, count)| *count == support_counts[0].1).count() > 1 {
        // 戦力トップが複数なら勝者なしで終了
        for (idx, _) in support_counts {
            original_orders[idx].set_failure();
        }

        // スタンドオフ地点を記録
        standoff_provinces.push(*target_location);
        return None;
    }

    // 支援数トップの単独勝利
    let winner_idx = support_counts[0].0;
    Some(winner_idx)
}

/// attacker 進軍成功かつ defender 進軍失敗からの defender の防衛成否判定
// - defender は進軍に失敗しているので支援は全て切れている
// - attacker に有効な支援が残っていれば進軍成功で defender の敗退
fn resolve_no_support_defense(original_orders: &mut [Order], attacker_idx: usize, defender_idx: usize, flanker_idx: Option<usize>) {
    let support_orders = collect_valid_support_orders(original_orders);

    if original_orders[attacker_idx].power == original_orders[defender_idx].power {
        // 自国軍同士の衝突は攻撃失敗
        original_orders[attacker_idx].set_failure();
        return;
    }
    if support_orders.iter().any(|o| o.is_matching_target(&original_orders[attacker_idx])) {
        // defender 防衛失敗
        original_orders[attacker_idx].set_success();
        original_orders[defender_idx].set_dislodged_from(&original_orders[attacker_idx].location());
        return;
    }

    // defender の防衛成功により attacker の進軍失敗からの attacker の防衛成否判定
    let Some(flanker_idx) = flanker_idx else {
        // flanker 不在による防衛成否判定不要で attacker の進軍失敗確定で終了
        original_orders[attacker_idx].set_failure();
        return;
    };

    debug_assert!(flanker_idx != attacker_idx && flanker_idx != defender_idx);

    if original_orders[flanker_idx].power == original_orders[attacker_idx].power {
        // 自国軍同士の衝突は攻撃失敗
        original_orders[attacker_idx].set_failure();
        return;
    }

    // defender との進軍競争に勝ち抜いた flanker からの攻撃に対する attacker の防衛成否判定
    if support_orders.iter().any(|o| o.is_matching_target(&original_orders[flanker_idx])) {
        // attacker 防衛失敗
        original_orders[attacker_idx].set_dislodged_from(&original_orders[flanker_idx].location());
        original_orders[flanker_idx].set_success();
    } else {
        // attacker 防衛成功（進軍は失敗）
        original_orders[attacker_idx].set_failure();
        original_orders[flanker_idx].set_failure();
    }
}

/// `original_orders[idx]` の移動先に対して、既存の有効な輸送命令群で海路到達可能か判定する。
fn can_reach_via_convoy(original_orders: &[Order], idx: usize) -> bool {
    let convoy_orders = collect_valid_convoy_orders(original_orders);
    let move_order = original_orders[idx];
    let OrderKind::Move(m) = move_order.kind else { unreachable!("expected Move") };
    let matched_convoys = collect_matched_convoy_orders(&convoy_orders, &move_order);

    can_move_via_convoy(&move_order, &m.dest, &matched_convoys)
}

/// `a_idx` と `b_idx` 双方の支援が生きている前提で支援数を比較し勝者のインデックスを返す。
/// - 同点または同勢力の場合は `None` を返す
fn decide_winner_by_supports(original_orders: &[Order], a_idx: usize, b_idx: usize) -> Option<usize> {
    // 自軍同士は勝者なし
    if original_orders[a_idx].power == original_orders[b_idx].power {
        return None;
    }

    let support_orders = collect_valid_support_orders(original_orders);
    let a_supports = support_orders.iter().filter(|s| s.is_matching_target(&original_orders[a_idx])).count();
    let b_supports = support_orders.iter().filter(|s| s.is_matching_target(&original_orders[b_idx])).count();

    match a_supports.cmp(&b_supports) {
        Ordering::Greater => Some(a_idx),
        Ordering::Less => Some(b_idx),
        Ordering::Equal => None,
    }
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
