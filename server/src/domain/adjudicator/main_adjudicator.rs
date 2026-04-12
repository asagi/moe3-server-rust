use super::super::helper::main_order_helper::MainOrderHelper;
use super::super::models::order::Order;
use super::super::models::order::OrderKind;
use super::super::models::order::OrderStatus;
use super::super::models::path::Path;
use super::super::models::power::Power;
use super::super::models::province::Province;
use super::super::models::unit::UnitKind;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::iter::successors;

pub struct MainAdjudicator;

impl MainAdjudicator {
    /// 移動命令検証
    pub(crate) fn validate_move_orders(orders: &mut [Order]) {
        for move_idx in orders.collect_unresolved_move_idxs() {
            // 隣接地域への移動は原則有効
            if Path::can_unit_move_to(
                &orders[move_idx].unit,
                orders[move_idx].dest().code(),
                orders[move_idx].via_convoy(),
            ) {
                orders[move_idx].set_valid();
                continue;
            }

            // 海軍は遠隔移動不可
            if let UnitKind::Fleet(_) = &orders[move_idx].unit.kind {
                orders[move_idx].set_invalid();
                continue;
            }

            // 陸軍の輸送は目的地が海岸でなければ無効
            if !orders[move_idx].dest().is_coast() {
                orders[move_idx].set_invalid();
                continue;
            }

            // 輸送前の所在地と目的地が同一の場合は無効
            if orders[move_idx].location().code() == orders[move_idx].dest().code() {
                orders[move_idx].set_invalid();
                continue;
            }

            // 輸送路の成立見込みがない場合は無効
            if !can_move_via_unresolved_matched_convoy(orders, move_idx) {
                orders[move_idx].set_invalid();
                continue;
            }

            orders[move_idx].set_valid();
            continue;
        }
    }

    /// 支援命令検証
    pub(crate) fn validate_support_orders(orders: &mut [Order]) {
        for support_idx in orders.collect_unresolved_support_idxs() {
            // 有効な支援対象が存在しない場合は無効
            let Some(target_idx) = orders.find_support_target_idx(support_idx) else {
                orders[support_idx].set_invalid();
                continue;
            };

            if let OrderKind::Move(_) = &orders[target_idx].kind {
                // 支援対象の移動命令の移動先に支援ユニットが移動できるなら有効
                if Path::can_unit_support_to(
                    &orders[support_idx].unit,
                    orders[support_idx].location().code(),
                    orders[target_idx].dest().code(),
                ) {
                    orders[support_idx].set_valid();
                    continue;
                }
            } else {
                // 支援対象の非移動命令の現在地に支援ユニットが移動できるなら有効
                if Path::can_unit_support_to(
                    &orders[support_idx].unit,
                    orders[support_idx].location().code(),
                    orders[target_idx].location().code(),
                ) {
                    orders[support_idx].set_valid();
                    continue;
                }
            }

            orders[support_idx].set_invalid();
            continue;
        }
    }

    /// 輸送命令検証
    pub(crate) fn validate_convoy_orders(orders: &mut [Order]) {
        let waters = orders.collect_fleet_water_codes();

        for convoy_idx in orders.collect_unresolved_convoy_idxs() {
            // 水上にない海軍への輸送命令は無効
            if !orders[convoy_idx].location().is_water() {
                orders[convoy_idx].set_invalid();
                continue;
            }

            // 輸送対象の所在地と目的地の両方に海路で接続される可能性がない場合は無効
            if !Path::is_reachable_by_sea(
                orders[convoy_idx].location().code(),
                orders[convoy_idx].target_unit().location().code(),
                &waters,
            ) {
                orders[convoy_idx].set_invalid();
                continue;
            }
            if !Path::is_reachable_by_sea(
                orders[convoy_idx].location().code(),
                orders[convoy_idx].target_dest().map(|d| d.code()).unwrap_or(""),
                &waters,
            ) {
                orders[convoy_idx].set_invalid();
                continue;
            }

            // 輸送対象移動命令が存在しなければ無効
            if orders.find_support_target_idx(convoy_idx).is_none() {
                orders[convoy_idx].set_invalid();
                continue;
            }

            // 輸送命令の有効を確定
            orders[convoy_idx].set_valid();

            // 輸送対象移動命令の海路利用の意思の判定
            if is_convoy_intended(orders, convoy_idx) {
                // 本輸送命令の存在を以て対象の移動命令の海路利用の意思が示されたものとする
                let move_idx = orders.find_convoy_target_idx(convoy_idx).expect("expected a target");
                orders[move_idx].set_via_convoy();
            }
            continue;
        }
    }

    /// 支援命令のカット判定
    pub(crate) fn handle_cutting_support_orders(orders: &mut [Order]) {
        for support_idx in orders.collect_valid_support_idxs() {
            // support_order に向かう移動命令は攻撃とみなす（自国軍は除く）
            let attacker_indicies = orders.collect_attacker_indicies(support_idx);

            // 支援命令をカットし得る移動命令がなければスキップ
            if attacker_indicies.is_empty() {
                continue;
            }

            // 複数個所からの攻撃は即カット
            if attacker_indicies.len() > 1 {
                orders[support_idx].set_cut();
                continue;
            }
            let cutter_idx = attacker_indicies[0];

            // support_order の支援対象の移動先が cutter ならカット回避
            if orders[support_idx].target_dest() == Some(orders[cutter_idx].location()) {
                continue;
            }

            // cutter が非輸送近接攻撃の場合はカット
            if is_attacker_without_convoy(orders, cutter_idx) {
                orders[support_idx].set_cut();
                continue;
            }

            // cutter の輸送経路が成立していなければ経路不成立でカット回避
            if !can_move_via_valid_matched_convoy(orders, cutter_idx, None) {
                continue;
            }

            // Szykman ルールに従ったカット回避
            if should_avoid_cut_due_to_datc_6_f_18(orders, support_idx, cutter_idx) {
                orders[cutter_idx].set_unreachable();
                continue;
            }

            // 支援対象が移動命令の場合
            if orders.get_support_target_move_order_kind(support_idx).is_some() {
                if should_avoid_cut_due_to_datc_6_f_22(orders, support_idx) {
                    continue;
                }
                if should_avoid_cut_due_to_datc_6_f_24_a(orders, support_idx) {
                    continue;
                }
            };

            // 支援対象が輸送命令の場合
            if orders.get_support_target_convoy_order_kind(support_idx).is_some() {
                if should_avoid_cut_due_to_datc_6_f_23(orders, support_idx) {
                    continue;
                }
                if should_avoid_cut_due_to_datc_6_f_24_b(orders, support_idx) {
                    continue;
                }
            }

            orders[support_idx].set_cut();
        }
    }

    /// 輸送妨害の優先解決
    pub(crate) fn handle_disruption_convoy_order(orders: &mut [Order], standoff_codes: &mut Vec<String>) {
        for convoy_idx in orders.collect_valid_convoy_idxs() {
            // 輸送命令に対する攻撃競争の勝者を取得
            let contested_code = orders[convoy_idx].location().code();
            let Some(winner_idx) = handle_conflicting(orders, contested_code, standoff_codes, true) else {
                // 勝者なし
                continue;
            };
            if orders[winner_idx].power == orders[convoy_idx].power {
                // 勝者が自軍であれば攻撃失敗
                orders[winner_idx].set_failure();
                continue;
            }

            // 攻防戦力比較
            let convoy_supports_count = orders.count_supports(convoy_idx, None);
            let winner_supports_count = orders.count_supports(winner_idx, Some(orders[convoy_idx].power));
            if convoy_supports_count >= winner_supports_count {
                // 攻撃失敗
                orders[winner_idx].set_failure();
                continue;
            }

            // 攻撃成功
            orders[winner_idx].set_success();
            orders[convoy_idx].set_dislodged_by(&orders[winner_idx].clone());

            // 敗退した輸送命令の対象である移動命令の成否を検証
            let move_idx = orders.find_convoy_target_idx(convoy_idx).expect("expected a target");
            if is_land_move_without_convoy(orders, move_idx) {
                // 輸送対象が海路利用を明示していなければ輸送路切断からの移動失敗及び支援カット撤回判定は不要（6.G.3）
                continue;
            }
            if !can_move_via_valid_matched_convoy(orders, move_idx, None) {
                // 輸送経路切断による移動失敗
                orders[move_idx].set_unreachable();

                // 輸送先にカットされた支援命令があればカットを取り消す
                if let Some(idx) = orders.find_support_at_dest_idx(move_idx) {
                    orders[idx].set_valid();
                }
                continue;
            }
        }
    }

    /// 交換移動命令解決
    pub(crate) fn handle_switch_orders(orders: &mut [Order], standoff_codes: &mut Vec<String>) {
        let move_idxs = orders.collect_valid_move_idxs();
        if move_idxs.len() < 2 {
            // 移動命令が 2 つ以上なければ終了
            return;
        }

        for move_idx in move_idxs {
            // 過去のループで対向の判定時に同時に処理済みであればスキップ
            if !orders[move_idx].is_valid() {
                continue;
            }

            // 対向する移動命令がなければスキップ
            let opposite_idx = orders.find_opposite_move_idx(move_idx);
            let Some(opposite_idx) = opposite_idx else {
                continue;
            };

            // スタンドオフ判定
            let conflict_winner_idx = handle_conflicting(orders, orders[opposite_idx].location().code(), standoff_codes, false);
            let opposite_winner_idx = handle_conflicting(orders, orders[move_idx].location().code(), standoff_codes, false);
            if conflict_winner_idx.is_none() && opposite_winner_idx.is_none() {
                // 両地域スタンドオフで関連する全軍が移動失敗
                continue;
            }
            if orders[move_idx].is_failure() && orders[opposite_idx].is_failure() {
                // 両者移動失敗であればスキップ
                continue;
            }

            if !orders[move_idx].is_failure() && !orders[opposite_idx].is_failure() {
                // どちらも移動可能であれば海路迂回交換移動判定
                let a_can = can_reach_via_convoy(orders, move_idx);
                let b_can = can_reach_via_convoy(orders, opposite_idx);
                if a_can || b_can {
                    // どちらか一方が海路迂回移動可能なら両者移動成功
                    orders[move_idx].set_success();
                    orders[opposite_idx].set_success();
                    continue;
                }
            }

            // 直接対決
            let Some(winner_idx) = determine_conflict_winner(orders, move_idx, opposite_idx) else {
                // 勝者なしで両者移動失敗
                orders[move_idx].set_failure();
                orders[opposite_idx].set_failure();
                continue;
            };

            // BELEAGUERED GARRISON 対応
            let loser_idx = if winner_idx == move_idx { opposite_idx } else { move_idx };
            if orders[winner_idx].is_failure() {
                // 勝者の別要因による移動失敗が事前に確定している場合は敗者も移動失敗で終了
                orders[loser_idx].set_failure();
                continue;
            }

            // BELEAGUERED GARRISON を適用しても勝者が変わらないなら結果を確定させる
            // （そうでなければ判定を保留して次のループへ）
            if handle_conflicting(orders, orders[loser_idx].location().code(), standoff_codes, true) == Some(winner_idx) {
                orders[winner_idx].set_success();
                orders[loser_idx].set_dislodged_by(&orders[winner_idx].clone());

                // 海路迂回移動が絡む場合はスタンドオフの無効化はしない
                let a_can = can_reach_via_convoy(orders, winner_idx);
                let b_can = can_reach_via_convoy(orders, loser_idx);
                if a_can || b_can {
                    continue;
                }

                // 撃退された軍の元所在地に発生させたスタンドオフを解除
                reset_standoff_failures_to_attacker_origin(orders, winner_idx, loser_idx, standoff_codes);
            }
        }
    }

    /// 支援命令撃退の優先解決
    pub(crate) fn handle_dislodging_support_orders(orders: &mut [Order], standoff_codes: &mut Vec<String>) {
        for support_idx in orders.collect_valid_support_idxs() {
            // 支援命令に対する攻撃競争の勝者を取得
            let Some(attacker_idx) = handle_conflicting(orders, orders[support_idx].location().code(), standoff_codes, true)
            else {
                // 勝者がいなければスキップ
                continue;
            };

            // 支援命令に対する排除の成否判定を実施する
            resolve_attack_against_non_move(orders, attacker_idx, support_idx);
        }
    }

    /// 未解決移動命令解決
    pub(crate) fn handle_remaining_move_orders(orders: &mut [Order], standoff_codes: &mut Vec<String>) {
        let mut dest_codes = orders.collect_valid_move_destination_code_set();
        let mut dest_snapshots: HashSet<Vec<&str>> = HashSet::new();

        loop {
            let Some(target_code) = dest_codes.shift_remove_index(0) else {
                // 全ての移動先に対する処理が終われば終了
                break;
            };

            // dest を IndexSet から Vec に変換して snapshot を取得
            if !dest_snapshots.insert(dest_codes.iter().copied().collect::<Vec<&str>>()) {
                // 全 dest に対する処理が一巡したので残りの全移動命令を成功判定でループ終了
                for idx in orders.collect_valid_move_idxs() {
                    orders[idx].set_success();
                }
                break;
            }

            // target_location に対する攻撃競争の勝者を取得
            let Some(attacker_idx) = handle_conflicting(orders, target_code, standoff_codes, true) else {
                // 単独勝者がいなければ関係全軍移動失敗
                dest_snapshots.clear();
                continue;
            };

            // 移動先の状況を確認
            let Some(occupant_idx) = orders.find_occupant_order_idx(target_code) else {
                // 移動先に駐留軍がいなければ attacker の移動成功で終了
                orders[attacker_idx].set_success();
                dest_snapshots.clear();
                continue;
            };

            // attacker の移動成否判定
            if !matches!(orders[occupant_idx].kind, OrderKind::Move(_)) {
                // 移動先の非移動駐留軍に対して排除判定を実施する
                resolve_attack_against_non_move(orders, attacker_idx, occupant_idx);
                dest_snapshots.clear();
                continue;
            } else {
                match &orders[occupant_idx].status {
                    OrderStatus::Success => {
                        // 移動先に駐留軍がいてもその移動が成功していれば attacker も移動成功で終了
                        orders[attacker_idx].set_success();
                        dest_snapshots.clear();
                        continue;
                    }
                    OrderStatus::Valid => {
                        // 移動先の駐留軍の移動が未解決の場合は処理を後回し
                        dest_codes.insert(target_code);
                        continue;
                    }
                    _ => {
                        // 移動先の駐留軍は移動に失敗しているので排除判定を実施する
                        resolve_no_support_defense(orders, attacker_idx, occupant_idx);
                        dest_snapshots.clear();
                        continue;
                    }
                }
            }
        }
    }

    /// 未処理の命令を全て成功判定
    pub(crate) fn succeed_remaining_orders(orders: &mut [Order]) {
        let unresolved_idxs = orders.collect_unresolved_idxs();

        for idx in unresolved_idxs {
            orders[idx].set_success();
        }
    }
}

/// 輸送経路が成立する最低限の可能性があるかを判定する。
fn can_move_via_unresolved_matched_convoy(orders: &[Order], move_idx: usize) -> bool {
    Path::is_reachable_by_sea(
        &orders[move_idx].location().code()[..3],
        &orders[move_idx].dest().code()[..3],
        &orders.get_unresolved_allowed_waters(move_idx),
    )
}

/// 輸送経路が成立しているかどうかを判定する。
fn can_move_via_valid_matched_convoy(orders: &[Order], move_idx: usize, exclude: Option<Province>) -> bool {
    // 隣接地移動の場合は輸送経路成立には下記いずれかの条件を満たす必要がある
    // - 移動命令に海路が指定されている
    // - 自国海軍に該当する輸送命令が出ている
    // 遠隔地移動の場合は海路指定の明示は必要ない（明示されていても良い）
    if Path::is_adjacent(orders[move_idx].location().code(), orders[move_idx].dest().code())
        && !orders[move_idx].via_convoy()
        && orders.collect_own_matching_convoy_idxs(move_idx).is_empty()
    {
        // 隣接地移動かつどちらの条件にも該当しない場合は輸送経路不成立
        return false;
    }

    Path::is_reachable_by_sea(
        &orders[move_idx].location().code()[..3],
        &orders[move_idx].dest().code()[..3],
        &orders.get_valid_allowed_waters(move_idx, exclude),
    )
}

/// 輸送命令の存在によって移動命令に海路利用の意図が示されていたかを判定する。
fn is_convoy_intended(orders: &[Order], convoy_idx: usize) -> bool {
    // 輸送命令と移動命令の勢力が異なる場合は海路利用の意図の明示とは認めない
    if orders[convoy_idx].power != orders[convoy_idx].target_unit().power {
        return false;
    }

    // 輸送対象の現在地と目的地が隣接していない場合は海路利用の意図の明示とみなす
    if !Path::is_adjacent(
        orders[convoy_idx].target_unit().location().code(),
        orders[convoy_idx].target_dest().map(|d| d.code()).unwrap_or(""),
    ) {
        return true;
    }

    // 輸送対象の現在地と目的地が海軍視点で隣接していない場合は海路利用の意図の明示とみなす
    if !Path::can_convoy_move(
        orders[convoy_idx].target_unit().location().code(),
        orders[convoy_idx].target_dest().map(|d| d.code()).unwrap_or(""),
    ) {
        return true;
    }

    // 輸送対象の現在地と目的地が隣接している場合は輸送命令がその両地と隣接していなければ
    // 海路利用の意図の明示とは認めない
    if !Path::is_adjacent(
        orders[convoy_idx].location().code(),
        orders[convoy_idx].target_unit().location().code(),
    ) {
        return false;
    }
    if !Path::is_adjacent(
        orders[convoy_idx].location().code(),
        orders[convoy_idx].target_dest().map(|d| d.code()).unwrap_or(""),
    ) {
        return false;
    }

    true
}

/// DATC テストケース 6.F.18 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
fn should_avoid_cut_due_to_datc_6_f_18(orders: &mut [Order], s1_idx: usize, attacker_idx: usize) -> bool {
    // 支援対象が輸送命令でなければカット回避失敗（S1 → C）
    let Some((c_idx, ck)) = orders.get_support_target_convoy_order_kind(s1_idx) else {
        return false;
    };

    // 輸送対象の移動先が支援命令でなければカット回避失敗（S1 → C → S2）
    let Some(s2_idx) = orders.iter().position(|o| o.location() == ck.target_dest) else {
        return false;
    };
    if !matches!(orders[s2_idx].kind, OrderKind::Support(_)) {
        return false;
    };

    // 自身の支援がなくても輸送海軍が撃退されない見込みなら本ルールによるカット回避不可
    if is_target_safe_without_own_support(orders, c_idx) {
        return false;
    }

    // 該当の輸送海軍を除いても輸送経路が維持されるのであれば本ルールによるカット回避不可
    if can_move_via_valid_matched_convoy(orders, attacker_idx, Some(orders[c_idx].location())) {
        return false;
    }
    true
}

/// DATC テストケース 6.F.22 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
fn should_avoid_cut_due_to_datc_6_f_22(orders: &[Order], start_idx: usize) -> bool {
    successors(Some(start_idx), |&idx| should_avoid_cut_due_to_datc_6_f_22_core(orders, idx))
        .skip(1)
        .any(|idx| idx == start_idx)
}

// DATC テストケース 6.F.22 対応判定処理本体
fn should_avoid_cut_due_to_datc_6_f_22_core(orders: &[Order], start_idx: usize) -> Option<usize> {
    // 支援対象が移動命令でなければカット回避失敗（S1 → M2）
    let m1k = orders.get_support_target_move_order_kind(start_idx)?;

    // 支援対象の移動先が輸送命令でなければカット回避失敗（S1 → M1 → C）
    let c_idx = orders.find_convoy_at_dest_idx(m1k)?;

    // 支援対象の移動先の輸送命令の輸送対象の移動先が支援命令でなければカット回避失敗（S1 → M1 → C → M2 → S2）
    // 支援対象の移動先の輸送命令の輸送対象移動命令（m2）を取得
    let m2 = orders.iter().find(|o| orders[c_idx].is_matching_target(o))?;
    let m2_idx = orders.iter().position(|o| o == m2)?;

    // 支援対象の移動先の輸送命令の輸送対象移動命令の移動先（s2）のインデックスを取得
    let s2_idx = orders.find_support_at_dest_idx(m2_idx)?;

    // m1 が c を撃退しても m2 の移動経路が維持されるならカット回避失敗
    if can_move_via_valid_matched_convoy(orders, m2_idx, Some(orders[c_idx].location())) {
        return None;
    };

    Some(s2_idx)
}

/// DATC テストケース 6.F.23 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
fn should_avoid_cut_due_to_datc_6_f_23(orders: &[Order], start_idx: usize) -> bool {
    successors(Some(start_idx), |&idx| {
        should_avoid_cut_due_to_datc_6_f_23_core(orders, idx, start_idx)
    })
    .skip(1)
    .any(|idx| idx == start_idx)
}

// DATC テストケース 6.F.23 対応判定処理本体
fn should_avoid_cut_due_to_datc_6_f_23_core(orders: &[Order], start_idx: usize, end_idx: usize) -> Option<usize> {
    // 支援対象が輸送命令でなければカット回避失敗（S1 → C）
    let (c_idx, ck) = orders.get_support_target_convoy_order_kind(start_idx)?;

    // 自身の支援がなくても輸送海軍が撃退されない見込みなら本ルールによるカット回避不可
    if is_target_safe_without_own_support(orders, c_idx) {
        return None;
    }

    // 輸送対象の移動先が支援命令でなければカット回避失敗（S1 → C → M1 → S2）
    let s2_idx = orders.iter().position(|o| o.location() == ck.target_dest)?;
    if !matches!(orders[s2_idx].kind, OrderKind::Support(_)) {
        return None;
    };

    // 支援対象の輸送対象を取得
    let attack_order_idx = orders.iter().position(|o| orders[c_idx].is_matching_target(o))?;

    // 該当の輸送海軍を除いても輸送経路が維持されるのであれば本ルールによるカット回避不可
    if can_move_via_valid_matched_convoy(orders, attack_order_idx, Some(orders[c_idx].location())) {
        return None;
    }

    if s2_idx == end_idx && orders[attack_order_idx].power == orders[end_idx].power {
        // 自軍輸送での循環攻撃偽装によるカット回避は認めない
        return None;
    }

    Some(s2_idx)
}

/// DATC テストケース 6.F.24 の Szykman ルールに従ったカット回避が成立するかどうかを判定する
/// 6.F.22 と 6.F.23 の複合ケース
fn should_avoid_cut_due_to_datc_6_f_24_a(orders: &[Order], s1_idx: usize) -> bool {
    successors(should_avoid_cut_due_to_datc_6_f_22_core(orders, s1_idx), |&s2_idx| {
        should_avoid_cut_due_to_datc_6_f_23_core(orders, s2_idx, s1_idx)
    })
    .any(|s3_idx| s3_idx == s1_idx)
}

/// DATC テストケース 6.F.24 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
/// 6.F.23 と 6.F.22 の複合ケース
fn should_avoid_cut_due_to_datc_6_f_24_b(orders: &[Order], s1_idx: usize) -> bool {
    successors(should_avoid_cut_due_to_datc_6_f_23_core(orders, s1_idx, s1_idx), |&s2_idx| {
        should_avoid_cut_due_to_datc_6_f_22_core(orders, s2_idx)
    })
    .any(|s3_idx| s3_idx == s1_idx)
}

/// 対象への攻撃命令が支援を一つ減らした状態で撃退されない可能性の有無を判定する
fn is_target_safe_without_own_support(orders: &[Order], target_idx: usize) -> bool {
    // 輸送海軍を攻撃する移動命令を収集（自国軍を除く）
    let attacker_indicies = orders.collect_attacker_indicies(target_idx);
    if attacker_indicies.is_empty() {
        return true;
    }

    // 自身の支援がなくても輸送海軍が撃退されない見込みなら本ルールによるカット回避不可
    let convoy_supports_count = orders.count_supports(target_idx, None).saturating_sub(1);
    let max_attacker_supports = orders.count_max_supports_for_attackers(&attacker_indicies, target_idx);
    if max_attacker_supports <= convoy_supports_count {
        return true;
    }
    false
}

/// 戦闘解決
fn handle_conflicting(
    orders: &mut [Order],
    target_code: &str,
    standoff_codes: &mut Vec<String>,
    enable_bg: bool,
) -> Option<usize> {
    // BELEAGUERED GARRISON が有効な場合は既に失敗判定された移動命令（敗退済みを除く）も再判定に加える
    let move_idxs: Vec<usize> = if enable_bg {
        orders.collect_non_dislodged_move_idxs()
    } else {
        orders.collect_valid_move_idxs()
    };
    let conflict_idxs: Vec<usize> = move_idxs
        .into_iter()
        .filter(|&idx| orders[idx].dest().code()[..3] == target_code[..3])
        .collect();

    // 移動命令がなければ勝者なしで終了
    if conflict_idxs.is_empty() {
        return None;
    }

    // 移動命令が 1 つなら即勝者確定で終了
    if conflict_idxs.len() == 1 {
        let winner_idx = conflict_idxs[0];
        return Some(winner_idx);
    }

    // 指定地点に非移動命令か失敗した移動命令があればその勢力を取得
    let occupant_power = if enable_bg {
        orders.get_occupant_power_on_target(target_code)
    } else {
        None
    };

    // 戦闘解決
    let winner_idx = handle_conflicting_core(orders, &conflict_idxs, target_code, standoff_codes, occupant_power)?;

    // DATC の解釈では
    // - 自軍撃退支援を有効とすることで逆にスタンドオフが発生して撃退を回避できるならその支援は有効
    // - そうでなければその支援は無効
    if occupant_power.is_none() {
        // 指定地点に駐留軍がいないなら自己撃退支援の有効無効は関係ないので勝者確定で判定終了
        return Some(winner_idx);
    }

    // 駐留軍の自己撃退支援有効での判定についての検証
    if orders.has_supports_excluding_occupant_power(winner_idx, occupant_power) {
        // 勝者に一つ以上の支援の付いている場合は自己撃退支援無効の条件で再判定した結果を正とする
        return handle_conflicting_core(orders, &conflict_idxs, target_code, standoff_codes, None);
    }

    // 勝者に有効な支援がなければ駐留軍の自己撃退含めて支援がなかったということで勝者確定で判定終了
    Some(winner_idx)
}

/// 戦闘解決コア
fn handle_conflicting_core(
    orders: &mut [Order],
    conflict_idxs: &[usize],
    target_code: &str,
    standoff_codes: &mut Vec<String>,
    occupant_power: Option<Power>,
) -> Option<usize> {
    // 戦力比較
    let strength_table = calc_conflict_strengths(orders, conflict_idxs, occupant_power);
    let Some(winner_idx) = get_unique_winner_idx(&strength_table) else {
        // 単独勝利なしで全軍移動失敗
        for (idx, _) in strength_table {
            orders[idx].set_failure();
        }

        // スタンドオフ地点を記録
        standoff_codes.push(target_code[..3].to_string());
        return None;
    };

    // 単独勝者以外の全軍移動失敗
    for (idx, _) in strength_table {
        if idx != winner_idx {
            orders[idx].set_failure();
        }
    }

    Some(winner_idx)
}

/// 競合戦力の集計
fn calc_conflict_strengths(orders: &[Order], conflict_idxs: &[usize], occupant_power: Option<Power>) -> Vec<(usize, usize)> {
    // - strength_table: (move_idx, 支援数) の配列
    // - strength_table は 支援数降順（戦力順）にソートする
    let mut strength_table: Vec<(usize, usize)> = conflict_idxs
        .iter()
        .map(|&idx| (idx, orders.count_supports(idx, occupant_power)))
        .collect();
    strength_table.sort_by(|a, b| b.1.cmp(&a.1));
    strength_table
}

/// 戦力比較の結果、単独勝利が存在しないことを判定する。
fn get_unique_winner_idx(strength_table: &[(usize, usize)]) -> Option<usize> {
    if strength_table
        .iter()
        .filter(|(_, count)| *count == strength_table[0].1)
        .count()
        == 1
    {
        return Some(strength_table[0].0);
    }
    None
}

/// 移動命令の移動先に対して、既存の有効な輸送命令群で海路到達可能か判定する。
fn can_reach_via_convoy(orders: &[Order], move_idx: usize) -> bool {
    if orders[move_idx].unit.is_fleet() {
        // 陸軍以外は海路迂回不可
        return false;
    }

    if !orders[move_idx].via_convoy() {
        return false;
    }

    can_move_via_valid_matched_convoy(orders, move_idx, None)
}

/// 移動命令が非輸送近接攻撃かどうかを判定する。
fn is_attacker_without_convoy(orders: &[Order], move_idx: usize) -> bool {
    if orders[move_idx].via_convoy() {
        return false;
    }

    Path::can_unit_move_to(&orders[move_idx].unit, orders[move_idx].dest().code(), false)
}

/// 交換移動命令双方の支援が生きている前提で支援数を比較し勝者のインデックスを返す。
/// - 同点または同勢力の場合は `None` を返す
fn determine_conflict_winner(orders: &[Order], a_idx: usize, b_idx: usize) -> Option<usize> {
    // 自軍同士は勝者なし
    if orders[a_idx].power == orders[b_idx].power {
        return None;
    }

    let support_idxs = orders.collect_valid_support_idxs();
    let a_supports = support_idxs
        .iter()
        .copied()
        .filter(|&idx| orders[idx].is_matching_target(&orders[a_idx]) && orders[idx].power != orders[b_idx].power)
        .count();
    let b_supports = support_idxs
        .iter()
        .copied()
        .filter(|&idx| orders[idx].is_matching_target(&orders[b_idx]) && orders[idx].power != orders[a_idx].power)
        .count();

    match a_supports.cmp(&b_supports) {
        Ordering::Greater => Some(a_idx),
        Ordering::Less => Some(b_idx),
        Ordering::Equal => None,
    }
}

/// 撃退された命令が攻撃側の元所在地に発生させたスタンドオフを無効化する。
fn reset_standoff_failures_to_attacker_origin(
    orders: &mut [Order],
    attacker_idx: usize,
    defender_idx: usize,
    standoff_codes: &mut Vec<String>,
) {
    // 防御側が撃退されていなければ処理は不要
    if !orders[defender_idx].is_dislodged() {
        return;
    }

    let origin_code = &orders[attacker_idx].location().code()[..3];

    for order in orders.iter_mut() {
        if !matches!(order.kind, OrderKind::Move(_)) || !order.is_failure() {
            continue;
        }
        if &order.dest().code()[..3] == origin_code {
            order.set_valid();
        }
    }

    standoff_codes.retain(|code| code != origin_code);
}

/// 移動命令が非移動命令を攻撃した場合の判定を行う
fn resolve_attack_against_non_move(orders: &mut [Order], attacker_idx: usize, defender_idx: usize) {
    // 自軍攻撃は失敗
    if orders[attacker_idx].power == orders[defender_idx].power {
        orders[attacker_idx].set_failure();
        return;
    }

    let attacker_supports = orders.count_supports(attacker_idx, Some(orders[defender_idx].power));
    let defender_supports = orders.count_supports(defender_idx, None);
    if attacker_supports > defender_supports {
        orders[attacker_idx].set_success();
        orders[defender_idx].set_dislodged_by(&orders[attacker_idx].clone());
    } else {
        // 同点・守備優勢ともに攻撃失敗
        orders[attacker_idx].set_failure();
    }
}

/// attacker 進軍成功かつ defender 進軍失敗からの defender の防衛成否判定
// - defender は進軍に失敗しているので支援は全て切れている
// - attacker に有効な支援が残っていれば進軍成功で defender の敗退
fn resolve_no_support_defense(orders: &mut [Order], attacker_idx: usize, defender_idx: usize) {
    let support_orders = orders.collect_valid_support_orders();

    if orders[attacker_idx].power == orders[defender_idx].power {
        // 自国軍同士の衝突は攻撃失敗
        orders[attacker_idx].set_failure();
        return;
    }
    if support_orders
        .iter()
        .any(|o| o.is_matching_target(&orders[attacker_idx]) && o.power != orders[defender_idx].power)
    {
        // defender 防衛失敗
        orders[attacker_idx].set_success();
        orders[defender_idx].set_dislodged_by(&orders[attacker_idx].clone());
        return;
    }

    // attacker の進軍失敗確定で終了
    orders[attacker_idx].set_failure();
}

/// 移動命令に海路利用の明示がないことを判定する
fn is_land_move_without_convoy(orders: &mut [Order], move_idx: usize) -> bool {
    // 下記条件をすべて満たす場合は true を返す
    // - 移動命令の移動先が隣接地域である
    // - 海路利用の明示がない
    // - 有効な自国海軍による輸送命令が存在しない
    if !Path::is_adjacent(orders[move_idx].location().code(), orders[move_idx].dest().code()) {
        return false;
    }

    if orders[move_idx].via_convoy() {
        return false;
    }

    orders.collect_own_matching_convoy_idxs(move_idx).is_empty()
}
