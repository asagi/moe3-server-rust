pub struct Adjudicator;

use crate::domain::order::Order;
use crate::domain::order::OrderKind;
use crate::domain::order::OrderStatus;
use crate::domain::path::Path;
use crate::domain::phase::order_helper::OrderHelper;
use crate::domain::power::Power;
use crate::domain::province::Province;
use crate::domain::unit::UnitKind;
use indexmap::IndexSet;
use std::cmp::Ordering;
use std::collections::HashSet;

impl Adjudicator {
    /// 移動命令検証
    pub(crate) fn validate_move_orders(orders: &mut [Order]) {
        for move_idx in orders.collect_unresolved_move_indices() {
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
            if !Self::can_move_via_unresolved_matched_convoy(orders, move_idx) {
                orders[move_idx].set_invalid();
                continue;
            }

            orders[move_idx].set_valid();
            continue;
        }
    }

    /// 支援命令検証
    pub(crate) fn validate_support_orders(orders: &mut [Order]) {
        for support_idx in orders.collect_unresolved_support_indices() {
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
    pub(crate) fn validate_convoy_orders(original_orders: &mut [Order]) {
        let move_orders = original_orders.collect_valid_move_orders();
        let allowed_waters: HashSet<&str> = original_orders
            .iter()
            .filter(|o| o.unit.is_fleet() && o.location().is_water())
            .map(|o| o.location().code())
            .collect();

        for convoy_order_idx in original_orders.collect_unresolved_convoy_indices() {
            // 水上にない艦への輸送命令は無効
            if !original_orders[convoy_order_idx].location().is_water() {
                original_orders[convoy_order_idx].set_invalid();
                continue;
            }

            // 輸送対象の所在地と目的地の両方に海路で接続される可能性がない場合は無効
            let OrderKind::Convoy(c) = &original_orders[convoy_order_idx].kind.clone() else {
                unreachable!("expected Convoy")
            };
            if !Path::is_adjacent(
                original_orders[convoy_order_idx].location().code(),
                c.target_unit.location().code(),
            ) && !Path::is_reachable_by_sea(
                &original_orders[convoy_order_idx].location().code()[..3],
                &c.target_unit.location().code()[..3],
                &allowed_waters,
            ) {
                original_orders[convoy_order_idx].set_invalid();
                continue;
            }
            if !Path::is_adjacent(original_orders[convoy_order_idx].location().code(), c.target_dest.code())
                && !Path::is_reachable_by_sea(
                    &original_orders[convoy_order_idx].location().code()[..3],
                    &c.target_dest.code()[..3],
                    &allowed_waters,
                )
            {
                original_orders[convoy_order_idx].set_invalid();
                continue;
            }

            // 輸送対象移動命令が存在すれば有効、なければ無効
            if move_orders
                .iter()
                .any(|m| original_orders[convoy_order_idx].is_matching_target(m))
            {
                original_orders[convoy_order_idx].set_valid();

                // convoy_order の対象移動命令を取得
                let Some(move_order_idx) = original_orders
                    .iter()
                    .position(|m| original_orders[convoy_order_idx].is_matching_target(m))
                else {
                    unreachable!("matching move order should exist")
                };

                if Self::is_convoy_intended(original_orders, convoy_order_idx, move_order_idx) {
                    // 本輸送命令の存在を以て対象の移動命令の海路利用の意思が示されたものとする
                    original_orders[move_order_idx].set_via_convoy();
                }
                continue;
            }

            original_orders[convoy_order_idx].set_invalid();
            continue;
        }
    }

    /// 支援命令のカット
    pub(crate) fn handle_cutting_support_orders(original_orders: &mut [Order]) {
        let move_orders = original_orders.collect_valid_move_orders();

        // 支援命令をカットし得る移動命令がなければ終了
        if !move_orders.iter().any(|m| m.is_valid()) {
            return;
        }

        for idx in original_orders.collect_valid_support_indices() {
            // support_order に向かう移動命令は攻撃とみなす（自国軍は除く）
            let attack_orders: Vec<&Order> = move_orders
                .iter()
                .filter(|o| o.power != original_orders[idx].power)
                .filter(|o| {
                    if let OrderKind::Move(m) = &o.kind {
                        m.dest.code()[..3] == original_orders[idx].location().code()[..3]
                    } else {
                        false
                    }
                })
                .collect();

            // 支援命令をカットし得る移動命令がなければスキップ
            if attack_orders.is_empty() {
                continue;
            }

            // 複数個所からの攻撃は即カット
            if attack_orders.len() > 1 {
                original_orders[idx].set_cut();
                continue;
            }
            let attack_order = attack_orders[0];
            let Some(attack_order_idx) = original_orders.iter().position(|o| o == attack_order) else {
                unreachable!("attack order should exist in original orders")
            };
            let OrderKind::Move(m) = &attack_order.kind else {
                unreachable!("expected Move")
            };

            // support_order の支援対象の移動先が attack_order ならカット回避
            let OrderKind::Support(s) = &original_orders[idx].kind.clone() else {
                unreachable!("expected Support")
            };
            if s.target_dest == Some(attack_order.location()) {
                continue;
            }

            // attack_order が隣接地域からの場合はカット（遠隔攻撃の場合はさらに経路判定が必要）
            if !m.via_convoy && Path::is_adjacent(attack_order.location().code(), original_orders[idx].location().code()) {
                original_orders[idx].set_cut();
                continue;
            }

            // 輸送経路が成立していなければ経路不成立でカット回避
            if !Self::can_move_via_valid_convoy(original_orders, attack_order_idx, None) {
                continue;
            }

            // Szykman ルールに従ったカット回避
            if Self::should_avoid_cut_due_to_datc_6_f_18(original_orders, idx, attack_order_idx) {
                original_orders[attack_order_idx].set_unreachable();
                continue;
            }

            // 支援対象が移動命令の場合
            if original_orders.get_support_target_move_order_kind(idx).is_some() {
                if Self::should_avoid_cut_due_to_datc_6_f_22(original_orders, idx, idx) {
                    continue;
                }
                if Self::should_avoid_cut_due_to_datc_6_f_24_a(original_orders, idx, idx) {
                    continue;
                }
            };

            // 支援対象が輸送命令の場合
            if original_orders.get_support_target_convoy_order_kind(idx).is_some() {
                if Self::should_avoid_cut_due_to_datc_6_f_23(original_orders, idx, idx) {
                    continue;
                }
                if Self::should_avoid_cut_due_to_datc_6_f_24_b(original_orders, idx, idx) {
                    continue;
                }
            }

            original_orders[idx].set_cut();
        }
    }

    /// 輸送妨害の優先解決
    pub(crate) fn handle_disruption_convoy_order(original_orders: &mut [Order], standoff_province_codes: &mut Vec<String>) {
        let support_orders = original_orders.collect_valid_support_orders();

        for convoy_order_idx in original_orders.collect_valid_convoy_indices() {
            // 輸送命令に対する攻撃競争の勝者を取得
            let Some(winner_idx) = Self::handle_conflicting(
                original_orders,
                original_orders[convoy_order_idx].location().code(),
                standoff_province_codes,
                true,
            ) else {
                // 勝者がいなければスキップ
                continue;
            };
            if original_orders[winner_idx].power == original_orders[convoy_order_idx].power {
                // 勝者が自国軍であればその移動は無条件失敗となりスキップ
                original_orders[winner_idx].set_failure();
                continue;
            }

            // 戦闘解決
            let convoy_supports_count = support_orders
                .iter()
                .filter(|o| o.is_matching_target(&original_orders[convoy_order_idx]))
                .count();
            let winner_supports_count = support_orders
                .iter()
                .filter(|o| {
                    o.is_matching_target(&original_orders[winner_idx]) && o.power != original_orders[convoy_order_idx].power
                })
                .count();
            if convoy_supports_count >= winner_supports_count {
                // 攻撃失敗
                original_orders[winner_idx].set_failure();
                continue;
            }

            // 輸送敗退
            original_orders[winner_idx].set_success();
            original_orders[convoy_order_idx].set_dislodged_from(&original_orders[winner_idx].location());

            // 敗退した輸送命令の対象である移動命令の成否を検証
            let Some(move_order_idx) = original_orders
                .iter()
                .position(|o| original_orders[convoy_order_idx].is_matching_target(o))
            else {
                unreachable!("move order matching convoy order should exist");
            };
            let OrderKind::Move(m) = &original_orders[move_order_idx].kind.clone() else {
                unreachable!("expected Move")
            };
            if Path::is_adjacent(original_orders[move_order_idx].location().code(), m.dest.code())
                && !m.via_convoy
                && original_orders
                    .collect_valid_convoy_orders()
                    .iter()
                    .find(|o| {
                        o.is_matching_target(&original_orders[move_order_idx]) && o.power == original_orders[move_order_idx].power
                    })
                    .is_none()
            {
                // 敗退した輸送命令の対象である移動命令が陸路移動可能かつ海路利用の明示がなければ輸送路切断判定は不要
                continue;
            }
            if !Self::can_move_via_valid_convoy(original_orders, move_order_idx, None) {
                // 輸送経路切断による移動失敗
                original_orders[move_order_idx].set_unreachable();

                // 輸送先にカットされた支援命令があればカットを取り消す
                for idx in original_orders.collect_cut_support_indices() {
                    let support_order = &mut original_orders[idx];
                    if support_order.location().code()[..3] == m.dest.code()[..3] {
                        support_order.set_valid();
                    }
                }
                continue;
            }
        }
    }

    /// 交換移動命令解決
    pub(crate) fn handle_switch_orders(original_orders: &mut [Order], standoff_province_codes: &mut Vec<String>) {
        let move_order_indices = original_orders.collect_valid_move_indices();
        if move_order_indices.len() < 2 {
            // 移動命令が 2 つ以上なければ終了
            return;
        }

        for idx in move_order_indices {
            // 過去のループで対向の判定時に同時に処理済みであればスキップ
            // - 以下 original_orders[idx] を甲軍とする
            if !original_orders[idx].is_valid() {
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
                        o.location().code()[..3] == m.dest.code()[..3]
                            && original_orders[idx].location().code()[..3] == om.dest.code()[..3]
                    } else {
                        false
                    }
            });
            let Some(opposite_idx) = opposite_move_order_idx else {
                continue;
            };

            // スタンドオフ判定
            let conflict_winner_idx = Self::handle_conflicting(
                original_orders,
                original_orders[opposite_idx].location().code(),
                standoff_province_codes,
                false,
            );
            let opposite_conflict_winner_idx = Self::handle_conflicting(
                original_orders,
                original_orders[idx].location().code(),
                standoff_province_codes,
                false,
            );
            if conflict_winner_idx.is_none() && opposite_conflict_winner_idx.is_none() {
                // 両地域スタンドオフで関連する全軍移動失敗
                continue;
            }
            if original_orders[idx].is_failure() && original_orders[opposite_idx].is_failure() {
                // 両者移動失敗であればスキップ
                continue;
            }

            if !original_orders[idx].is_failure() && !original_orders[opposite_idx].is_failure() {
                // どちらも移動可能であれば海路迂回交換移動判定
                let a_can = Self::can_reach_via_convoy(original_orders, idx);
                let b_can = Self::can_reach_via_convoy(original_orders, opposite_idx);
                if a_can || b_can {
                    // どちらか一方が海路迂回移動可能なら双方移動成功
                    original_orders[idx].set_success();
                    original_orders[opposite_idx].set_success();
                    continue;
                }
            }

            // 直接対決
            if let Some(winner_idx) = Self::decide_move_conflict_winner(original_orders, idx, opposite_idx) {
                let loser_idx = if winner_idx == idx { opposite_idx } else { idx };
                if original_orders[winner_idx].is_valid() {
                    let redecided_winner_idx = Self::handle_conflicting(
                        original_orders,
                        original_orders[loser_idx].location().code(),
                        standoff_province_codes,
                        true,
                    );
                    if redecided_winner_idx == Some(winner_idx) {
                        // BELEAGUERED GARRISON を適用しても winner が勝利できるなら winner の移動成功で処理を続行
                        original_orders[winner_idx].set_success();
                        original_orders[loser_idx].set_dislodged_from(&original_orders[winner_idx].location());

                        // 海路迂回移動が絡む場合はスタンドオフの無効化はしない
                        let a_can = Self::can_reach_via_convoy(original_orders, winner_idx);
                        let b_can = Self::can_reach_via_convoy(original_orders, loser_idx);
                        if a_can || b_can {
                            continue;
                        }

                        // 撃退された軍の元所在地に発生させたスタンドオフを無効化
                        Self::reset_standoff_failures_to_attacker_origin(
                            original_orders,
                            winner_idx,
                            loser_idx,
                            standoff_province_codes,
                        );
                        continue;
                    }
                    continue;
                } else {
                    // 勝者のスタンドオフによる移動失敗が事前に確定している場合
                    original_orders[loser_idx].set_failure();
                    continue;
                }
            } else {
                original_orders[idx].set_failure();
                original_orders[opposite_idx].set_failure();
                continue;
            }
        }
    }

    /// 支援命令撃退の優先解決
    pub(crate) fn handle_dislodging_support_orders(original_orders: &mut [Order], standoff_province_codes: &mut Vec<String>) {
        for idx in original_orders.collect_valid_support_indices() {
            // 支援命令に対する攻撃競争の勝者を取得
            let Some(attacker_idx) = Self::handle_conflicting(
                original_orders,
                original_orders[idx].location().code(),
                standoff_province_codes,
                true,
            ) else {
                // 勝者がいなければスキップ
                continue;
            };

            // 支援命令に対する排除の成否判定を実施する
            Self::resolve_attack_against_non_move(original_orders, attacker_idx, idx);
        }
    }

    /// 未解決移動命令解決
    pub(crate) fn handle_remaining_move_orders(original_orders: &mut [Order], standoff_province_codes: &mut Vec<String>) {
        let move_orders = original_orders.collect_valid_move_orders();
        let mut dest_codes: IndexSet<&str> = move_orders
            .collect_valid_move_destination_set()
            .iter()
            .map(|&p| &p.code()[..3])
            .collect();

        let mut dest_snapshots: HashSet<Vec<&str>> = HashSet::new();

        loop {
            let Some(target_location_code) = dest_codes.shift_remove_index(0) else {
                // 全ての移動先に対する処理が終われば終了
                break;
            };

            // dest を IndexSet から Vec に変換して snapshot を取得
            if !dest_snapshots.insert(dest_codes.iter().copied().collect::<Vec<&str>>()) {
                // 全 dest に対する処理が一巡したので残りの全移動命令を成功判定でループ終了
                for idx in original_orders.collect_valid_move_indices() {
                    original_orders[idx].set_success();
                }
                break;
            }

            // target_location に対する攻撃競争の勝者を取得
            let Some(attacker_idx) =
                Self::handle_conflicting(original_orders, target_location_code, standoff_province_codes, true)
            else {
                // 勝者がいなければ関係全軍移動失敗
                dest_snapshots.clear();
                continue;
            };

            // 移動先の状況を確認
            let Some(occupant_idx) = original_orders.find_occupant_order_idx(target_location_code) else {
                // 移動先に駐留軍がいなければ勝者の移動成功で終了
                original_orders[attacker_idx].set_success();
                dest_snapshots.clear();
                continue;
            };

            // winner の移動成否判定
            if !matches!(original_orders[occupant_idx].kind, OrderKind::Move(_)) {
                // 移動先の非移動駐留軍に対して排除判定を実施する
                Self::resolve_attack_against_non_move(original_orders, attacker_idx, occupant_idx);
                dest_snapshots.clear();
                continue;
            } else {
                match &original_orders[occupant_idx].status {
                    OrderStatus::Success => {
                        if original_orders[occupant_idx].is_success() {
                            // 移動先に駐留軍がいてもの軍の移動が成功していれば勝者の移動成功で終了
                            original_orders[attacker_idx].set_success();
                            dest_snapshots.clear();
                            continue;
                        }
                    }
                    OrderStatus::Valid => {
                        if original_orders.has_confliction(target_location_code) {
                            // 移動先の駐留軍の移動が未解決のため勝者の暫定的に勝者の移動成功として処理する
                            original_orders[attacker_idx].set_success();
                            dest_snapshots.clear();
                            continue;
                        } else {
                            // 移動先の駐留軍の移動が未解決の場合は処理を後回し
                            dest_codes.insert(target_location_code);
                            continue;
                        }
                    }
                    _ => {
                        // 移動先の駐留軍は移動に失敗しているので排除判定を実施する
                        Self::resolve_no_support_defense(original_orders, attacker_idx, occupant_idx);
                        dest_snapshots.clear();
                        continue;
                    }
                }
            }
        }
    }

    /// 未処理の命令を全て成功判定
    pub(crate) fn succeed_remaining_orders(original_orders: &mut [Order]) {
        let unresolved_indices = original_orders.collect_unresolved_indices();

        for idx in unresolved_indices {
            original_orders[idx].set_success();
        }
    }

    /// 輸送経路が成立しているかどうかを判定する。
    fn can_move_via_unresolved_matched_convoy(orders: &[Order], move_idx: usize) -> bool {
        let allowed_waters: HashSet<&str> = orders
            .collect_matched_convoy_order_indices(&orders[move_idx])
            .iter()
            .map(|&idx| orders[idx].location().code())
            .collect();

        Path::is_reachable_by_sea(
            &orders[move_idx].location().code()[..3],
            &orders[move_idx].dest().code()[..3],
            &allowed_waters,
        )
    }

    /// 輸送命令の存在によって移動命令に海路利用の意図が示されていたかを判定する。
    fn is_convoy_intended(original_orders: &[Order], convoy_order_idx: usize, move_order_idx: usize) -> bool {
        let OrderKind::Convoy(c) = &original_orders[convoy_order_idx].kind.clone() else {
            unreachable!("expected Convoy")
        };

        if original_orders[convoy_order_idx].power != original_orders[move_order_idx].power {
            return false;
        }

        if !Path::is_adjacent(original_orders[move_order_idx].location().code(), c.target_dest.code()) {
            return true;
        }

        if !Path::can_convoy_move(original_orders[move_order_idx].location().code(), c.target_dest.code()) {
            return true;
        }

        if !Path::is_adjacent(
            original_orders[convoy_order_idx].location().code(),
            original_orders[move_order_idx].location().code(),
        ) {
            return false;
        }

        if !Path::is_adjacent(original_orders[convoy_order_idx].location().code(), c.target_dest.code()) {
            return false;
        }

        true
    }

    /// 輸送経路が成立しているかどうかを判定する。
    fn can_move_via_valid_convoy(orders: &[Order], move_order_idx: usize, exclude: Option<Province>) -> bool {
        let convoy_orders = orders.collect_valid_convoy_orders();
        let OrderKind::Move(m) = &orders[move_order_idx].kind else {
            unreachable!("expected Move")
        };

        // 隣接地移動の場合は輸送経路成立には下記いずれかの条件を満たす必要がある
        // - 移動命令に海路が指定されている
        // - 自国海軍に該当する輸送命令が出ている
        // 遠隔地移動の場合は海路指定の明示は必要ない（明示されていても良い）
        if Path::is_adjacent(orders[move_order_idx].location().code(), m.dest.code())
            && !m.via_convoy
            && convoy_orders
                .iter()
                .find(|o| o.is_matching_target(&orders[move_order_idx]) && o.power == orders[move_order_idx].power)
                .is_none()
        {
            return false;
        }

        let matched_convoy_orders: Vec<&Order> = convoy_orders
            .collect_matched_convoy_order_indices(&orders[move_order_idx])
            .iter()
            .filter(|&&idx| Some(convoy_orders[idx].location()) != exclude)
            .map(|&idx| &convoy_orders[idx])
            .collect();

        let allowed_waters: HashSet<&str> = matched_convoy_orders.iter().map(|order| order.location().code()).collect();
        Path::is_reachable_by_sea(
            &orders[move_order_idx].location().code()[..3],
            &m.dest.code()[..3],
            &allowed_waters,
        )
    }

    // DATC テストケース 6.F.18 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
    fn should_avoid_cut_due_to_datc_6_f_18(original_orders: &mut [Order], s1_idx: usize, attacker_idx: usize) -> bool {
        let cloned_orders = &mut original_orders.to_vec();

        // 支援対象が輸送命令でなければカット回避失敗（S1 → C）
        let Some((c_idx, ck)) = cloned_orders.get_support_target_convoy_order_kind(s1_idx) else {
            return false;
        };

        // 輸送対象の移動先が支援命令でなければカット回避失敗（S1 → C → S2）
        let Some(s2_idx) = cloned_orders.iter().position(|o| o.location() == ck.target_dest) else {
            return false;
        };
        let OrderKind::Support(_) = &cloned_orders[s2_idx].kind.clone() else {
            return false;
        };

        // 自身の支援がなくても輸送海軍が撃退されない見込みなら本ルールによるカット回避不可
        if Self::is_target_safe_without_own_support(cloned_orders, c_idx) {
            return false;
        }

        // 該当の輸送海軍を除いても輸送経路が維持されるのであれば本ルールによるカット回避不可
        if Self::can_move_via_valid_convoy(cloned_orders, attacker_idx, Some(cloned_orders[c_idx].location())) {
            return false;
        }
        true
    }

    // DATC テストケース 6.F.22 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
    fn should_avoid_cut_due_to_datc_6_f_22(original_orders: &[Order], start_idx: usize, end_idx: usize) -> bool {
        let cloned_orders = &mut original_orders.to_vec();

        let Some(s2_idx) = Self::should_avoid_cut_due_to_datc_6_f_22_core(cloned_orders, start_idx) else {
            return false;
        };
        if s2_idx == end_idx {
            return true;
        }
        Self::should_avoid_cut_due_to_datc_6_f_22(original_orders, s2_idx, end_idx)
    }

    fn should_avoid_cut_due_to_datc_6_f_22_core(orders: &[Order], start_idx: usize) -> Option<usize> {
        // 支援対象が移動命令でなければカット回避失敗（S1 → M2）
        let m1k = orders.get_support_target_move_order_kind(start_idx)?;

        // 支援対象の移動先が輸送命令でなければカット回避失敗（S1 → M1 → C）
        let c_idx = orders.find_target_convoy_order_idx(m1k)?;

        // 支援対象の移動先の輸送命令の輸送対象の移動先が支援命令でなければカット回避失敗（S1 → M1 → C → M2 → S2）
        let m2k = orders.get_support_target_move_order_kind(c_idx)?;

        // 支援対象の移動先の輸送命令の輸送対象移動命令（m2）を取得
        let m2 = orders.iter().find(|o| orders[c_idx].is_matching_target(o))?;
        let m2_idx = orders.iter().position(|o| o == m2)?;

        // 支援対象の移動先の輸送命令の輸送対象移動命令の移動先（s2）のインデックスを取得
        let s2_idx = orders.find_target_support_order_idx(m2k)?;

        // m1 が c を撃退しても m2 の移動経路が維持されるならカット回避失敗
        if Self::can_move_via_valid_convoy(orders, m2_idx, Some(orders[c_idx].location())) {
            return None;
        };

        Some(s2_idx)
    }

    // DATC テストケース 6.F.23 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
    fn should_avoid_cut_due_to_datc_6_f_23(original_orders: &[Order], start_idx: usize, end_idx: usize) -> bool {
        let cloned_orders = &mut original_orders.to_vec();

        let Some(s2_idx) = Self::should_avoid_cut_due_to_datc_6_f_23_core(cloned_orders, start_idx, end_idx) else {
            return false;
        };
        if s2_idx == end_idx {
            return true;
        }

        Self::should_avoid_cut_due_to_datc_6_f_23(original_orders, s2_idx, end_idx)
    }

    fn should_avoid_cut_due_to_datc_6_f_23_core(orders: &[Order], start_idx: usize, end_idx: usize) -> Option<usize> {
        // 支援対象が輸送命令でなければカット回避失敗（S1 → C）
        let (c_idx, ck) = orders.get_support_target_convoy_order_kind(start_idx)?;

        // 自身の支援がなくても輸送海軍が撃退されない見込みなら本ルールによるカット回避不可
        if Self::is_target_safe_without_own_support(orders, c_idx) {
            return None;
        }

        // 輸送対象の移動先が支援命令でなければカット回避失敗（S1 → C → M1 → S2）
        let s2_idx = orders.iter().position(|o| o.location() == ck.target_dest)?;
        let OrderKind::Support(_) = &orders[s2_idx].kind.clone() else {
            return None;
        };

        // 支援対象の輸送対象を取得
        let attack_order_idx = orders.iter().position(|o| orders[c_idx].is_matching_target(o))?;

        // 該当の輸送海軍を除いても輸送経路が維持されるのであれば本ルールによるカット回避不可
        if Self::can_move_via_valid_convoy(orders, attack_order_idx, Some(orders[c_idx].location())) {
            return None;
        }

        if s2_idx == end_idx && orders[attack_order_idx].power == orders[end_idx].power {
            // 自軍輸送での循環攻撃偽装によるカット回避は認めない
            return None;
        }

        Some(s2_idx)
    }

    // DATC テストケース 6.F.24 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
    fn should_avoid_cut_due_to_datc_6_f_24_a(original_orders: &[Order], s1_idx: usize, end_idx: usize) -> bool {
        let cloned_orders = &mut original_orders.to_vec();

        let Some(s2_idx) = Self::should_avoid_cut_due_to_datc_6_f_22_core(cloned_orders, s1_idx) else {
            return false;
        };

        let Some(s3_idx) = Self::should_avoid_cut_due_to_datc_6_f_23_core(cloned_orders, s2_idx, end_idx) else {
            return false;
        };

        if s3_idx == end_idx {
            return true;
        }

        Self::should_avoid_cut_due_to_datc_6_f_24_a(original_orders, s3_idx, end_idx)
    }

    // DATC テストケース 6.F.24 の Szykman ルールに従ったカット回避が成立するかどうかを判定する。
    fn should_avoid_cut_due_to_datc_6_f_24_b(original_orders: &[Order], s1_idx: usize, end_idx: usize) -> bool {
        let cloned_orders = &mut original_orders.to_vec();

        let Some(s2_idx) = Self::should_avoid_cut_due_to_datc_6_f_23_core(cloned_orders, s1_idx, end_idx) else {
            return false;
        };

        let Some(s3_idx) = Self::should_avoid_cut_due_to_datc_6_f_22_core(cloned_orders, s2_idx) else {
            return false;
        };

        if s3_idx == end_idx {
            return true;
        }

        Self::should_avoid_cut_due_to_datc_6_f_24_b(original_orders, s3_idx, end_idx)
    }

    /// 対象への攻撃命令が支援を一つ減らした状態で撃退されない見込みかどうかを判定する
    fn is_target_safe_without_own_support(orders: &[Order], target_idx: usize) -> bool {
        // 輸送海軍を攻撃する移動命令を収集（自国軍を除く）
        let attacker_indicies = orders.collect_attacker_indicies(target_idx);
        if attacker_indicies.is_empty() {
            return true;
        }

        // 自身の支援がなくても輸送海軍が撃退されない見込みなら本ルールによるカット回避不可
        let convoy_supports_count = orders.count_supports(target_idx).saturating_sub(1);
        let max_attacker_supports = orders.count_max_supports_for_attackers(&attacker_indicies, target_idx);
        if max_attacker_supports <= convoy_supports_count {
            return true;
        }
        false
    }

    /// 戦闘解決
    fn handle_conflicting(
        original_orders: &mut [Order],
        target_location_code: &str,
        standoff_province_codes: &mut Vec<String>,
        enable_bg: bool,
    ) -> Option<usize> {
        // BELEAGUERED GARRISON が有効な場合は失敗判定された移動命令も conflicting_move_indicies に追加する
        let conflicting_move_indicies: Vec<usize> = if enable_bg {
            original_orders
                .collect_valid_order_indices()
                .into_iter()
                .filter(|&idx| {
                    if let OrderKind::Move(m) = original_orders[idx].kind {
                        m.dest.code()[..3] == target_location_code[..3]
                    } else {
                        false
                    }
                })
                .collect()
        } else {
            original_orders
                .collect_valid_move_indices()
                .into_iter()
                .filter(|&idx| {
                    if let OrderKind::Move(m) = original_orders[idx].kind {
                        m.dest.code()[..3] == target_location_code[..3]
                    } else {
                        false
                    }
                })
                .collect()
        };

        // 移動命令がなければ勝者なしで終了
        if conflicting_move_indicies.is_empty() {
            return None;
        }

        // 移動命令が 1 つなら即勝者確定で終了
        if conflicting_move_indicies.len() == 1 {
            let winner_idx = conflicting_move_indicies[0];
            return Some(winner_idx);
        }

        // 指定地点に非移動命令か失敗した移動命令があればその勢力を取得
        let target_power = if enable_bg {
            original_orders.occupant_power_on_target(target_location_code)
        } else {
            None
        };

        // DATC の解釈では、自軍撃退支援を有効とすることで逆にスタンドオフが発生して撃退を回避できるなら有効、
        // そうでなければ無効にするという判定が優先される。

        // 自己撃退支援有効で計算して、勝者なしならそれが正解。
        let winner1 = Self::handle_conflicting_core(
            original_orders,
            &conflicting_move_indicies,
            target_location_code,
            standoff_province_codes,
            target_power,
        )?;
        if target_power.is_none() {
            // 指定地点に駐留軍がいないなら自己撃退支援の有効無効は関係ないので winner1 が勝者で確定
            return Some(winner1);
        }

        // 自己撃退支援有効で計算して、勝者が出てもその勝者に有効な支援がなければ
        // 自己撃退含めて支援がなかったということでそれも正解。
        // 一方で一つ以上の支援の付いた勝者が出た場合、撃退回避の可能性を探すために
        // 自己撃退支援を無効にして計算しなおす必要がある。
        let support_orders = original_orders.collect_valid_support_orders();
        if target_power.is_some()
            && !support_orders
                .iter()
                .any(|s| s.is_matching_target(&original_orders[winner1]) && Some(s.power) != target_power)
        {
            // winner1 に有効な支援がついていないなら winner1 が勝者で確定
            return Some(winner1);
        }

        // 自己撃退支援無効で計算したらどんな結果が出ようとそれが正解。
        Self::handle_conflicting_core(
            original_orders,
            &conflicting_move_indicies,
            target_location_code,
            standoff_province_codes,
            None,
        )
    }

    // 支援数集計
    // - support_counts: (move_order の index, 支援数) の配列
    // - support_counts は 支援数降順（戦力順）にソートする
    fn handle_conflicting_core(
        original_orders: &mut [Order],
        conflicting_move_indicies: &[usize],
        target_location_code: &str,
        standoff_province_codes: &mut Vec<String>,
        target_power: Option<Power>,
    ) -> Option<usize> {
        let support_orders = original_orders.collect_valid_support_orders();
        let mut support_counts: Vec<(usize, usize)> = conflicting_move_indicies
            .iter()
            .map(|&idx| {
                let count = support_orders
                    .iter()
                    .filter(|s| {
                        s.is_matching_target(&original_orders[idx])
                            && match target_power {
                                Some(p) => s.power != p,
                                None => true,
                            }
                    })
                    .count();
                (idx, count)
            })
            .collect();
        support_counts.sort_by(|a, b| b.1.cmp(&a.1));

        // 戦闘解決： 単独勝利以外は移動失敗
        if support_counts
            .iter()
            .filter(|(_, count)| *count == support_counts[0].1)
            .count()
            > 1
        {
            // 戦力トップが複数なら勝者なしで終了
            for (idx, _) in support_counts {
                original_orders[idx].set_failure();
            }

            // スタンドオフ地点を記録
            standoff_province_codes.push(target_location_code[..3].to_string());
            return None;
        }

        // 支援数トップの単独勝利
        let winner_idx = support_counts[0].0;
        for (idx, _) in support_counts {
            if idx == winner_idx {
                continue;
            }
            original_orders[idx].set_failure();
        }
        Some(winner_idx)
    }

    /// 移動命令の移動先に対して、既存の有効な輸送命令群で海路到達可能か判定する。
    fn can_reach_via_convoy(original_orders: &[Order], move_order_idx: usize) -> bool {
        if original_orders[move_order_idx].unit.is_fleet() {
            // 陸軍以外は海路迂回不可
            return false;
        }

        let OrderKind::Move(m) = &original_orders[move_order_idx].kind else {
            unreachable!("expected Move");
        };
        if !m.via_convoy {
            return false;
        }

        Self::can_move_via_valid_convoy(original_orders, move_order_idx, None)
    }

    /// 交換移動命令双方の支援が生きている前提で支援数を比較し勝者のインデックスを返す。
    /// - 同点または同勢力の場合は `None` を返す
    fn decide_move_conflict_winner(original_orders: &[Order], a_idx: usize, b_idx: usize) -> Option<usize> {
        // 自軍同士は勝者なし
        if original_orders[a_idx].power == original_orders[b_idx].power {
            return None;
        }

        let support_orders = original_orders.collect_valid_support_orders();
        let a_supports = support_orders
            .iter()
            .filter(|s| s.is_matching_target(&original_orders[a_idx]) && s.power != original_orders[b_idx].power)
            .count();
        let b_supports = support_orders
            .iter()
            .filter(|s| s.is_matching_target(&original_orders[b_idx]) && s.power != original_orders[a_idx].power)
            .count();

        match a_supports.cmp(&b_supports) {
            Ordering::Greater => Some(a_idx),
            Ordering::Less => Some(b_idx),
            Ordering::Equal => None,
        }
    }

    /// 撃退された命令が攻撃側の元所在地に発生させたスタンドオフを無効化する。
    fn reset_standoff_failures_to_attacker_origin(
        original_orders: &mut [Order],
        attacker_idx: usize,
        defender_idx: usize,
        standoff_province_codes: &mut Vec<String>,
    ) {
        // 防御側が撃退されていなければ処理は不要
        if !original_orders[defender_idx].is_dislodged() {
            return;
        }

        let origin_code = &original_orders[attacker_idx].location().code()[..3];
        for order in original_orders.iter_mut() {
            if order.is_assumed() || !order.is_failure() {
                continue;
            }
            if let OrderKind::Move(m) = &order.kind
                && &m.dest.code()[..3] == origin_code
            {
                order.set_valid();
            }
        }

        standoff_province_codes.retain(|code| code != origin_code);
    }

    /// 移動命令 `attacker_idx` が非移動命令 `defender_idx` を攻撃したときの判定を行う。
    /// - attacker が勝てば attacker を success、defender を dislodged_from(attacker.location()) にする。
    /// - attacker が負ければ attacker を failure にする。
    /// - 同勢力の場合は attacker を failure にする。
    fn resolve_attack_against_non_move(original_orders: &mut [Order], attacker_idx: usize, defender_idx: usize) {
        // 自軍攻撃は失敗
        if original_orders[attacker_idx].power == original_orders[defender_idx].power {
            original_orders[attacker_idx].set_failure();
            return;
        }

        let support_orders = original_orders.collect_valid_support_orders();
        let attacker_supports = support_orders
            .iter()
            .filter(|s| s.is_matching_target(&original_orders[attacker_idx]) && s.power != original_orders[defender_idx].power)
            .count();
        let defender_supports = support_orders
            .iter()
            .filter(|s| s.is_matching_target(&original_orders[defender_idx]))
            .count();

        if attacker_supports > defender_supports {
            original_orders[attacker_idx].set_success();
            original_orders[defender_idx].set_dislodged_from(&original_orders[attacker_idx].location());
        } else {
            // 同点・守備優勢ともに攻撃失敗
            original_orders[attacker_idx].set_failure();
        }
    }

    /// attacker 進軍成功かつ defender 進軍失敗からの defender の防衛成否判定
    // - defender は進軍に失敗しているので支援は全て切れている
    // - attacker に有効な支援が残っていれば進軍成功で defender の敗退
    fn resolve_no_support_defense(original_orders: &mut [Order], attacker_idx: usize, defender_idx: usize) {
        let support_orders = original_orders.collect_valid_support_orders();

        if original_orders[attacker_idx].power == original_orders[defender_idx].power {
            // 自国軍同士の衝突は攻撃失敗
            original_orders[attacker_idx].set_failure();
            return;
        }
        if support_orders
            .iter()
            .any(|o| o.is_matching_target(&original_orders[attacker_idx]) && o.power != original_orders[defender_idx].power)
        {
            // defender 防衛失敗
            original_orders[attacker_idx].set_success();
            original_orders[defender_idx].set_dislodged_from(&original_orders[attacker_idx].location());
            return;
        }

        // attacker の進軍失敗確定で終了
        original_orders[attacker_idx].set_failure();
    }
}
