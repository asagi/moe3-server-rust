// models
use super::AdjustmentPhase;
use super::DebriefPhase;
use super::FallMainPhase;
use super::FallRetreatPhase;
use super::Phase;
use super::PhaseContext;
use super::Power;
use super::Province;
use super::ReadyPhase;
use super::SpringMainPhase;
use super::SpringRetreatPhase;
use super::Territory;
use super::Unit;

// enums
use super::OrderKind;
use super::PhaseKind;

// adjudicators
use super::AdjustmentAdjudicator;
use super::MainAdjudicator;
use super::RetreatAdjudicator;

// helpers
use super::AdjustmentOrderHelper;
use super::MainOrderHelper;
use super::RetreatOrderHelper;
use super::UnitHelper;

// external crates
use strum::IntoEnumIterator;

// constants
use super::SUPPLY_CENTERS_FOR_SOLO;

/// フェイズのロジック
impl Phase {
    /// フェイズを締め切り命令を解決する。
    #[allow(dead_code)]
    pub(crate) fn close(mut self, context: &mut PhaseContext) {
        match self.kind {
            PhaseKind::Ready(r) => r.close(&mut self, context),
            PhaseKind::SpringMain(s) => s.close(&mut self, context),
            PhaseKind::SpringRetreat(s) => s.close(&mut self, context),
            PhaseKind::FallMain(f) => f.close(&mut self, context),
            PhaseKind::FallRetreat(f) => f.close(&mut self, context),
            PhaseKind::Adjustment(a) => a.close(&mut self, context),
            PhaseKind::Debrief(d) => d.close(&mut self, context),
        }
    }

    /// フェイズ初期化処理
    fn initialize(&mut self, prev_phase: &Phase) {
        self.territories = prev_phase.territories.clone();
        self.units = prev_phase.units.clone();

        // 初期命令生成
        match self.kind {
            PhaseKind::SpringMain(_) | PhaseKind::FallMain(_) => self.init_main_phase_orders(),
            PhaseKind::SpringRetreat(_) | PhaseKind::FallRetreat(_) => {
                self.init_retreat_phase_orders();
                self.standoff_codes = prev_phase.standoff_codes.clone();
            }
            PhaseKind::Adjustment(_) => self.init_adjustment_phase_orders(),
            _ => {}
        }
    }

    /// メインフェイズの初期命令を生成する
    fn init_main_phase_orders(&mut self) {
        for unit in &mut self.units {
            self.orders.push(unit.hold());
        }
    }

    /// 撤退フェイズの初期命令を生成する
    fn init_retreat_phase_orders(&mut self) {
        for unit in &mut self.units {
            if unit.dislodged {
                self.orders.push(unit.disband());
            }
        }
    }

    /// 調整フェイズの初期命令を生成する
    fn init_adjustment_phase_orders(&mut self) {
        for p in Power::iter() {
            // 解体必要数算出
            let sc_count = self.count_supply_centers(&p);
            let unit_count = self.count_units(&p);
            let adjustment_capacity = unit_count as isize - sc_count as isize;

            // 解体の必要がない場合はスキップ
            if adjustment_capacity < 1 {
                continue;
            }
            let mut remaining = adjustment_capacity as usize;

            // 解体命令登録
            let disband_candidates = &mut self.units.collect_units_for_civil_disorder(&p, &self.territories);
            disband_candidates.reverse();
            while remaining > 0 {
                let unit = disband_candidates.pop().unwrap();
                self.orders.push(unit.disband());
                remaining -= 1;
            }
        }
    }

    /// フェイズの生成処理
    fn new(year: i32, index: i32, kind: PhaseKind) -> Self {
        Self {
            game_number: None,
            index,
            year,
            kind,
            orders: Vec::new(),
            units: Vec::new(),
            territories: Vec::new(),
            standoff_codes: Vec::new(),
        }
    }

    /// 準備フェイズを生成する。
    #[allow(dead_code)]
    pub(crate) fn new_ready() -> Self {
        let mut phase = Self::new(1900, 0, PhaseKind::Ready(ReadyPhase {}));

        // 初期ユニット生成
        phase.units = vec![
            Unit::new_army(Power::Austria, Province::from_code("vie").unwrap()),
            Unit::new_army(Power::Austria, Province::from_code("bud").unwrap()),
            Unit::new_fleet(Power::Austria, Province::from_code("tri").unwrap()),
            Unit::new_fleet(Power::England, Province::from_code("lon").unwrap()),
            Unit::new_fleet(Power::England, Province::from_code("edi").unwrap()),
            Unit::new_army(Power::England, Province::from_code("lvp").unwrap()),
            Unit::new_army(Power::France, Province::from_code("par").unwrap()),
            Unit::new_army(Power::France, Province::from_code("mar").unwrap()),
            Unit::new_fleet(Power::France, Province::from_code("bre").unwrap()),
            Unit::new_army(Power::Germany, Province::from_code("ber").unwrap()),
            Unit::new_army(Power::Germany, Province::from_code("mun").unwrap()),
            Unit::new_fleet(Power::Germany, Province::from_code("kie").unwrap()),
            Unit::new_army(Power::Italy, Province::from_code("rom").unwrap()),
            Unit::new_army(Power::Italy, Province::from_code("ven").unwrap()),
            Unit::new_fleet(Power::Italy, Province::from_code("nap").unwrap()),
            Unit::new_army(Power::Russia, Province::from_code("mos").unwrap()),
            Unit::new_fleet(Power::Russia, Province::from_code("sev").unwrap()),
            Unit::new_army(Power::Russia, Province::from_code("war").unwrap()),
            Unit::new_fleet(Power::Russia, Province::from_code("stp_sc").unwrap()),
            Unit::new_fleet(Power::Turkey, Province::from_code("ank").unwrap()),
            Unit::new_army(Power::Turkey, Province::from_code("con").unwrap()),
            Unit::new_army(Power::Turkey, Province::from_code("smy").unwrap()),
        ];

        // 初期領土生成
        phase.territories = vec![
            Territory::new(Power::Austria, "vie"),
            Territory::new(Power::Austria, "bud"),
            Territory::new(Power::Austria, "tri"),
            Territory::new(Power::England, "lon"),
            Territory::new(Power::England, "edi"),
            Territory::new(Power::England, "lvp"),
            Territory::new(Power::France, "par"),
            Territory::new(Power::France, "mar"),
            Territory::new(Power::France, "bre"),
            Territory::new(Power::Germany, "ber"),
            Territory::new(Power::Germany, "mun"),
            Territory::new(Power::Germany, "kie"),
            Territory::new(Power::Italy, "rom"),
            Territory::new(Power::Italy, "ven"),
            Territory::new(Power::Italy, "nap"),
            Territory::new(Power::Russia, "mos"),
            Territory::new(Power::Russia, "sev"),
            Territory::new(Power::Russia, "war"),
            Territory::new(Power::Russia, "stp"),
            Territory::new(Power::Turkey, "ank"),
            Territory::new(Power::Turkey, "con"),
            Territory::new(Power::Turkey, "smy"),
        ];

        phase
    }

    /// 春メインフェイズを生成する。
    /// - 春命令は「次年の開始フェイズ」なので、必ず `year = prev_year + 1`。
    /// - このルールは Ready -> SpringOrder / Adjustment -> SpringOrder の両方で共通。
    pub(crate) fn new_spring_main(current_year: i32, current_index: i32) -> Self {
        Self::new(current_year + 1, current_index + 1, PhaseKind::SpringMain(SpringMainPhase {}))
    }

    /// 春撤退フェイズを生成する。
    pub(crate) fn new_spring_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::SpringRetreat(SpringRetreatPhase {}))
    }

    /// 秋メインフェイズを生成する。
    pub(crate) fn new_fall_main(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::FallMain(FallMainPhase {}))
    }

    /// 秋撤退フェイズを生成する。
    pub(crate) fn new_fall_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::FallRetreat(FallRetreatPhase {}))
    }

    /// 調整フェイズを生成する。
    pub(crate) fn new_adjustment(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::Adjustment(AdjustmentPhase {}))
    }

    /// 感想戦フェイズを生成する。
    #[allow(dead_code)]
    pub(crate) fn new_debrief(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::Debrief(DebriefPhase {}))
    }

    /// 指定した国が現在保有する補給都市数を取得する
    pub(crate) fn count_supply_centers(&self, power: &Power) -> usize {
        self.territories
            .iter()
            .filter(|t| t.power() == power)
            .filter(|t| Province::from_code(t.code_with_coast()).is_some_and(Province::is_supply_center))
            .count()
    }

    /// 指定した国が現在保有するユニット数を取得する
    pub(crate) fn count_units(&self, power: &Power) -> usize {
        self.units.iter().filter(|u| &u.power() == power).count()
    }
}

/// フェイズの終了ロジック
trait PhaseCloseLogic {
    /// フェイズ終了処理
    fn close(&self, current_phase: &mut Phase, context: &mut PhaseContext) {
        // context.active_powers から全滅した国を除外する
        for p in Power::iter() {
            if current_phase.count_supply_centers(&p) == 0 {
                context.remove_power(&p);
            }
        }

        // 和平判定
        if context.is_draw() && matches!(current_phase.kind, PhaseKind::SpringMain(_) | PhaseKind::FallMain(_)) {
            self.finish_on_draw(current_phase, context);
            return;
        }

        // 命令解決
        self.resolve_orders(current_phase);

        // 占領処理
        self.occupy(current_phase);

        // 制覇判定
        if self.check_solo_condition(current_phase) {
            self.finish_on_solo(current_phase, context);
            return;
        }

        // 次フェイズ生成
        context.push_phase(current_phase.clone());
        if let Some(next_phase) = self.create_next_phase(current_phase) {
            // スキップ判定と再帰
            if self.should_skip_next_phase(&next_phase, context) {
                next_phase.close(context);
                return;
            }

            context.push_phase(next_phase.clone());
        }
    }

    /// 和平合意による終了処理
    fn finish_on_draw(&self, current_phase: &Phase, context: &mut PhaseContext) {
        // メインフェイズ格納
        context.push_phase(current_phase.clone());

        // 撤退フェイズ格納
        let Some(retreat_phase) = self.create_next_phase(current_phase) else {
            unreachable!("draw: create_next_phase must not return None");
        };
        context.push_phase(retreat_phase.clone());

        let last_phase = if let PhaseKind::FallRetreat(f) = retreat_phase.kind {
            // 秋の場合のみ調整フェイズ格納
            let Some(adjustment_phase) = f.create_next_phase(&retreat_phase) else {
                unreachable!("draw: create_next_phase must not return None");
            };
            context.push_phase(adjustment_phase.clone());
            adjustment_phase
        } else {
            retreat_phase
        };

        // 感想戦フェイズ格納
        let mut debrief_phase = Phase::new_debrief(last_phase.year, last_phase.index);
        debrief_phase.initialize(&last_phase);
        context.push_phase(debrief_phase);
    }

    /// 命令解決処理
    fn resolve_orders(&self, _current_phase: &mut Phase) {
        // 準備フェイズと感想戦フェイズでは何もしない
    }

    /// 占領処理
    fn occupy(&self, _current_phase: &mut Phase) {
        // 秋の撤退フェイズ以外では何もしない
    }

    /// 制覇判定処理
    fn check_solo_condition(&self, _current_phase: &Phase) -> bool {
        // 秋の撤退フェイズ以外では常に false
        false
    }

    /// 制覇勝利による終了処理
    fn finish_on_solo(&self, current_phase: &mut Phase, context: &mut PhaseContext) {
        // 撤退フェイズ格納
        context.push_phase(current_phase.clone());

        // 調整フェイズ格納
        let Some(adjustment_phase) = self.create_next_phase(current_phase) else {
            unreachable!("solo: create_next_phase must not return None");
        };
        context.push_phase(adjustment_phase.clone());

        // 感想戦フェイズ格納
        let mut debrief_phase = Phase::new_debrief(adjustment_phase.year, adjustment_phase.index);
        debrief_phase.initialize(&adjustment_phase);
        context.push_phase(debrief_phase);
    }

    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase>;

    /// 次フェイズがスキップ可能な場合に true を返す
    /// true を返す可能性のある場合にのみオーバーライドする
    fn should_skip_next_phase(&self, _next_phase: &Phase, _context: &PhaseContext) -> bool {
        false
    }
}

/// 準備フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for ReadyPhase {
    /// 次フェイズ生成
    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase> {
        let mut phase = Phase::new_spring_main(current_phase.year, current_phase.index);
        phase.initialize(current_phase);
        Some(phase)
    }
}

/// 春メインフェイズの終了ロジックの差分実装
impl PhaseCloseLogic for SpringMainPhase {
    /// 命令解決処理
    fn resolve_orders(&self, current_phase: &mut Phase) {
        resolve_orders_for_main_phase(current_phase);
    }

    /// 次フェイズ生成
    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase> {
        let mut phase = Phase::new_spring_retreat(current_phase.year, current_phase.index);
        phase.initialize(current_phase);
        Some(phase)
    }

    /// 撤退指示が必要なユニットが存在しない場合に true を返す
    fn should_skip_next_phase(&self, next_phase: &Phase, context: &PhaseContext) -> bool {
        !next_phase
            .units
            .iter()
            .any(|u| u.dislodged && context.active_powers().contains(&u.power))
    }
}

/// 春撤退フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for SpringRetreatPhase {
    /// 命令解決処理
    fn resolve_orders(&self, current_phase: &mut Phase) {
        resolve_orders_for_retreat_phase(current_phase);
    }

    /// 次フェイズ生成
    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase> {
        let mut phase = Phase::new_fall_main(current_phase.year, current_phase.index);
        phase.initialize(current_phase);
        Some(phase)
    }
}

/// 秋メインフェイズの終了ロジックの差分実装
impl PhaseCloseLogic for FallMainPhase {
    /// 命令解決処理
    fn resolve_orders(&self, current_phase: &mut Phase) {
        resolve_orders_for_main_phase(current_phase);
    }

    /// 次フェイズ生成
    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase> {
        let mut phase = Phase::new_fall_retreat(current_phase.year, current_phase.index);
        phase.initialize(current_phase);
        Some(phase)
    }

    /// 撤退指示が必要なユニットが存在しない場合に true を返す
    fn should_skip_next_phase(&self, next_phase: &Phase, context: &PhaseContext) -> bool {
        !next_phase
            .units
            .iter()
            .any(|u| u.dislodged && context.active_powers().contains(&u.power))
    }
}

/// 秋撤退フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for FallRetreatPhase {
    /// 命令解決処理
    fn resolve_orders(&self, current_phase: &mut Phase) {
        resolve_orders_for_retreat_phase(current_phase);
    }

    /// 占領処理
    fn occupy(&self, current_phase: &mut Phase) {
        for idx in current_phase.units.collect_all_idxs() {
            let code = current_phase.units[idx].location.code().to_string();

            if let Some(occupied_idx) = current_phase.territories.iter().position(|t| t.code() == code) {
                // すでに占領されている場合は占領国を更新
                current_phase.territories[occupied_idx].set_power(current_phase.units[idx].power);
                continue;
            } else if !current_phase.units[idx].location.is_water() {
                // 水域以外の未占領地は占領情報を新規に登録
                current_phase
                    .territories
                    .push(Territory::new(current_phase.units[idx].power, &code));
            }
        }
    }

    /// 制覇判定処理
    fn check_solo_condition(&self, current_phase: &Phase) -> bool {
        for p in Power::iter() {
            if current_phase.count_supply_centers(&p) >= SUPPLY_CENTERS_FOR_SOLO {
                return true;
            }
        }
        false
    }

    /// 次フェイズ生成
    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase> {
        let mut phase = Phase::new_adjustment(current_phase.year, current_phase.index);
        phase.initialize(current_phase);
        Some(phase)
    }

    /// 調整が不要な場合に true を返す
    fn should_skip_next_phase(&self, next_phase: &Phase, context: &PhaseContext) -> bool {
        for p in Power::iter() {
            if !context.active_powers().contains(&p) {
                // 全滅した国や無政府の国は考慮しない
                continue;
            }

            let sc_count = next_phase.count_supply_centers(&p);
            let unit_count = next_phase.count_units(&p);
            if unit_count == sc_count {
                continue;
            }
            return false;
        }
        true
    }
}

/// 調整フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for AdjustmentPhase {
    fn resolve_orders(&self, current_phase: &mut Phase) {
        resolve_orders_for_adjustment_phase(current_phase);
    }

    /// 次フェイズ生成
    fn create_next_phase(&self, current_phase: &Phase) -> Option<Phase> {
        let mut phase = Phase::new_spring_main(current_phase.year, current_phase.index);
        phase.initialize(current_phase);
        Some(phase)
    }
}

/// 感想戦フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for DebriefPhase {
    fn create_next_phase(&self, _current_phase: &Phase) -> Option<Phase> {
        None
    }
}

/// メインフェイズの命令解決処理
fn resolve_orders_for_main_phase(current_phase: &mut Phase) {
    let orders = &mut current_phase.orders;
    let standoff_province_codes = &mut current_phase.standoff_codes;

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
            if let Some(idx) = current_phase.units.iter().position(|u| u == &order.unit) {
                // 対象ユニットを削除
                current_phase.units.remove(idx);

                // 所在地を移動先に変更して保存
                current_phase.units.push(Unit {
                    location: m.dest,
                    ..order.unit
                });
                continue;
            }
            unreachable!("A move order succeeded, but the target unit could not be found.");
        }

        // 撃退された軍に攻撃元情報を記録
        if !order.is_dislodged() {
            continue;
        }
        let Some(dislodged_unit) = current_phase
            .units
            .iter_mut()
            .find(|u| u.power == order.unit.power && u.kind == order.unit.kind && u.location == order.unit.location)
        else {
            continue;
        };
        if order.unit.dislodged_from.is_some() {
            dislodged_unit.set_dislodged_from(order.unit.dislodged_from);
        } else {
            dislodged_unit.set_dislodged_via_convoy();
        }
    }
}

/// 撤退フェイズの命令解決処理
fn resolve_orders_for_retreat_phase(current_phase: &mut Phase) {
    let orders = &mut current_phase.orders;
    let units = &current_phase.units;
    let standoff_codes = &current_phase.standoff_codes;

    // # 01. 撤退命令検証
    RetreatAdjudicator::validate_retreat_orders(orders, units, standoff_codes);

    // # 02. 撤退命令処理
    RetreatAdjudicator::handle_retreat_orders(orders);

    // # 03. 命令解決後のユニット配置情報をフェイズに反映
    for order in orders.collect_not_assumed_retreats() {
        // 撤退フェイズでの処理対象の軍をいったん削除
        if matches!(&order.kind, OrderKind::Retreat(_) | OrderKind::Disband(_))
            && let Some(idx) = current_phase
                .units
                .iter()
                .position(|u| u.power == order.unit.power && u.kind == order.unit.kind && u.location == order.unit.location)
        {
            current_phase.units.remove(idx);
        }

        if let OrderKind::Retreat(r) = &order.kind
            && order.is_success()
        {
            // 撤退に成功した軍のみ所在を移動先に変更して再配置
            current_phase.units.push(Unit {
                location: r.dest,
                dislodged_from: None,
                dislodged: false,
                ..order.unit
            });
        }
    }
}

/// 調整フェイズの命令解決処理
fn resolve_orders_for_adjustment_phase(current_phase: &mut Phase) {
    // # 01. 増設命令検証
    AdjustmentAdjudicator::validate_build_orders(current_phase);

    // # 02. 解体命令検証
    AdjustmentAdjudicator::validate_disband_orders(current_phase);

    // # 03. 未処理命令をすべて無効判定
    AdjustmentAdjudicator::invalidate_unresolved_orders(current_phase);

    // # 04. 命令解決後のユニット配置情報をフェイズに反映
    for idx in current_phase.orders.collect_valid_adjustment_idxs() {
        match current_phase.orders[idx].kind {
            OrderKind::Build(_) => {
                // 増設命令が有効な場合はユニットを追加
                current_phase.units.push(current_phase.orders[idx].unit);
            }
            OrderKind::Disband(_) => {
                // 解体命令が有効な場合はユニットを削除
                if let Some(idx) = current_phase.units.iter().position(|u| u == &current_phase.orders[idx].unit) {
                    current_phase.units.remove(idx);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tests::a;
    use crate::domain::tests::f;
    use crate::domain::tests::p;
    use crate::domain::tests::t;

    /// Ready → SpringMain: ユニットと領土が引き継がれ、全ユニットにホールド命令が生成される
    #[test]
    fn test_initialize_ready_to_spring_main() {
        let ready = Phase::new_ready();
        let expected_units = ready.units.clone();
        let expected_territories = ready.territories.clone();

        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        spring_main.initialize(&ready);

        assert_eq!(spring_main.units, expected_units);
        assert_eq!(spring_main.territories, expected_territories);
        assert_eq!(spring_main.orders.len(), expected_units.len());
        assert!(spring_main.orders.iter().all(|o| matches!(o.kind, OrderKind::Hold(_))));
        assert!(spring_main.standoff_codes.is_empty());
    }

    /// SpringMain → SpringRetreat: 撃退ユニットにのみ解体命令が生成され standoff_codes が引き継がれる
    #[test]
    fn test_initialize_spring_main_to_spring_retreat() {
        let mut spring_main = Phase::new_spring_main(1901, 1);
        let unit_normal = a("f", "par");
        let mut unit_dislodged = a("a", "vie");
        unit_dislodged.set_dislodged_from(Some(p("boh")));
        spring_main.units = vec![unit_normal, unit_dislodged];
        spring_main.territories = vec![Territory::new(Power::France, "par")];
        spring_main.standoff_codes = vec!["boh".to_string()];

        let mut spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        spring_retreat.initialize(&spring_main);

        assert_eq!(spring_retreat.units, spring_main.units);
        assert_eq!(spring_retreat.territories, spring_main.territories);
        assert_eq!(spring_retreat.standoff_codes, spring_main.standoff_codes);
        // 撃退ユニットにのみ解体命令が生成される
        assert_eq!(spring_retreat.orders.len(), 1);
        assert!(matches!(spring_retreat.orders[0].kind, OrderKind::Disband(_)));
        assert_eq!(spring_retreat.orders[0].unit, unit_dislodged);
    }

    /// SpringRetreat → FallMain: ユニットと領土が引き継がれ、standoff_codes は引き継がれない
    #[test]
    fn test_initialize_spring_retreat_to_fall_main() {
        let mut spring_retreat = Phase::new_spring_retreat(1901, 2);
        spring_retreat.units = vec![a("f", "par")];
        spring_retreat.territories = vec![Territory::new(Power::France, "par")];
        spring_retreat.standoff_codes = vec!["boh".to_string()];

        let mut fall_main = Phase::new_fall_main(spring_retreat.year, spring_retreat.index);
        fall_main.initialize(&spring_retreat);

        assert_eq!(fall_main.units, spring_retreat.units);
        assert_eq!(fall_main.territories, spring_retreat.territories);
        assert!(fall_main.standoff_codes.is_empty());
        assert_eq!(fall_main.orders.len(), 1);
        assert!(matches!(fall_main.orders[0].kind, OrderKind::Hold(_)));
    }

    /// FallMain → FallRetreat: 撃退ユニットにのみ解体命令が生成され standoff_codes が引き継がれる
    #[test]
    fn test_initialize_fall_main_to_fall_retreat() {
        let mut fall_main = Phase::new_fall_main(1901, 3);
        let unit_normal = f("e", "nth");
        let mut unit_dislodged = a("g", "ber");
        unit_dislodged.set_dislodged_from(Some(p("sil")));
        fall_main.units = vec![unit_normal, unit_dislodged];
        fall_main.territories = vec![Territory::new(Power::England, "lon")];
        fall_main.standoff_codes = vec!["sil".to_string()];

        let mut fall_retreat = Phase::new_fall_retreat(fall_main.year, fall_main.index);
        fall_retreat.initialize(&fall_main);

        assert_eq!(fall_retreat.units, fall_main.units);
        assert_eq!(fall_retreat.territories, fall_main.territories);
        assert_eq!(fall_retreat.standoff_codes, fall_main.standoff_codes);
        // 撃退ユニットにのみ解体命令が生成される
        assert_eq!(fall_retreat.orders.len(), 1);
        assert!(matches!(fall_retreat.orders[0].kind, OrderKind::Disband(_)));
        assert_eq!(fall_retreat.orders[0].unit, unit_dislodged);
    }

    /// FallRetreat → Adjustment: 余剰ユニットに市民的混乱解体命令が生成される
    #[test]
    fn test_initialize_fall_retreat_to_adjustment() {
        let mut fall_retreat = Phase::new_fall_retreat(1901, 4);
        // Austria has 1 supply center but 2 units → needs 1 civil disorder disband
        fall_retreat.units = vec![a("a", "vie"), a("a", "boh")];
        fall_retreat.territories = vec![Territory::new(Power::Austria, "vie")];

        let mut adjustment = Phase::new_adjustment(fall_retreat.year, fall_retreat.index);
        adjustment.initialize(&fall_retreat);

        assert_eq!(adjustment.units, fall_retreat.units);
        assert_eq!(adjustment.territories, fall_retreat.territories);
        // ユニット数 (2) が補給都市数 (1) を超えるため 1 つ解体命令が生成される
        assert_eq!(adjustment.orders.len(), 1);
        assert!(matches!(adjustment.orders[0].kind, OrderKind::Disband(_)));
    }

    /// Adjustment → SpringMain: ユニットと領土が引き継がれ、全ユニットにホールド命令が生成される
    #[test]
    fn test_initialize_adjustment_to_spring_main() {
        let mut adjustment = Phase::new_adjustment(1901, 5);
        adjustment.units = vec![a("f", "par"), f("e", "lon")];
        adjustment.territories = vec![Territory::new(Power::France, "par"), Territory::new(Power::England, "lon")];

        let mut spring_main = Phase::new_spring_main(adjustment.year, adjustment.index);
        spring_main.initialize(&adjustment);

        assert_eq!(spring_main.units, adjustment.units);
        assert_eq!(spring_main.territories, adjustment.territories);
        assert_eq!(spring_main.orders.len(), 2);
        assert!(spring_main.orders.iter().all(|o| matches!(o.kind, OrderKind::Hold(_))));
        assert!(spring_main.standoff_codes.is_empty());
    }

    #[test]
    fn test_new_spring_order_always_increments_year() {
        let p = Phase::new_spring_main(1900, 7);
        assert_eq!(p.year, 1901);
        assert_eq!(p.index, 8);
        assert!(matches!(p.kind, PhaseKind::SpringMain(_)));
    }

    #[test]
    fn close_on_spring_main_skips_empty_retreat_and_advances_to_fall_main() {
        let current_phase = Phase::new_spring_main(1900, 0);
        let mut context = PhaseContext::new();
        current_phase.close(&mut context);
        let tail_phase = &context.pop_phase().unwrap();
        assert_eq!(tail_phase.year, 1901);
        assert_eq!(tail_phase.index, 3);
        assert!(matches!(tail_phase.kind, PhaseKind::FallMain(_)));
    }

    #[test]
    fn close_on_fall_main_skips_empty_retreat_and_advances_past_retreat_and_adjustment() {
        let current_phase = Phase::new_fall_main(1901, 3);
        let mut context = PhaseContext::new();
        current_phase.close(&mut context);
        let tail_phase = &context.pop_phase().unwrap();
        assert_eq!(tail_phase.year, 1902);
        assert_eq!(tail_phase.index, 7);
        assert!(matches!(tail_phase.kind, PhaseKind::SpringMain(_)));
    }

    #[test]
    fn close_on_spring_main_with_inactive_dislodged_power_skips_retreat() {
        let mut current_phase = Phase::new_spring_main(1900, 0);
        let mut dislodged = a("f", "par");
        dislodged.set_dislodged_from(Some(p("gas")));
        current_phase.units = vec![dislodged];

        let mut context = PhaseContext::new();
        context.remove_power(&Power::France);
        current_phase.close(&mut context);

        let tail_phase = context.pop_phase().unwrap();
        assert_eq!(tail_phase.year, 1901);
        assert_eq!(tail_phase.index, 3);
        assert!(matches!(tail_phase.kind, PhaseKind::FallMain(_)));
    }

    #[test]
    fn close_on_spring_main_with_active_dislodged_power_keeps_retreat() {
        let mut current_phase = Phase::new_spring_main(1900, 0);
        let mut dislodged = a("f", "par");
        dislodged.set_dislodged_from(Some(p("gas")));
        current_phase.units = vec![dislodged];
        current_phase.territories = vec![t("f", "par")];

        let mut context = PhaseContext::new();
        current_phase.close(&mut context);

        let tail_phase = context.pop_phase().unwrap();
        assert_eq!(tail_phase.year, 1901);
        assert_eq!(tail_phase.index, 2);
        assert!(matches!(tail_phase.kind, PhaseKind::SpringRetreat(_)));
    }

    #[test]
    fn close_pushes_spring_draw_phase_sequence_into_context() {
        let current_phase = Phase::new_spring_main(1901, 0);
        let mut context = PhaseContext::new();
        context.set_draw();
        current_phase.close(&mut context);
        let phases = context.phases();
        assert_eq!(phases.len(), 3);
        assert!(matches!(phases[0].kind, PhaseKind::SpringMain(_)));
        assert!(matches!(phases[1].kind, PhaseKind::SpringRetreat(_)));
        assert!(matches!(phases[2].kind, PhaseKind::Debrief(_)));
    }

    #[test]
    fn close_pushes_fall_draw_phase_sequence_into_context() {
        let current_phase = Phase::new_fall_main(1901, 0);
        let mut context = PhaseContext::new();
        context.set_draw();
        current_phase.close(&mut context);
        let phases = context.phases();
        assert_eq!(phases.len(), 4);
        assert!(matches!(phases[0].kind, PhaseKind::FallMain(_)));
        assert!(matches!(phases[1].kind, PhaseKind::FallRetreat(_)));
        assert!(matches!(phases[2].kind, PhaseKind::Adjustment(_)));
        assert!(matches!(phases[3].kind, PhaseKind::Debrief(_)));
    }

    #[test]
    fn close_on_fall_retreat_with_solo_pushes_adjustment_and_debrief() {
        let mut current_phase = Phase::new_fall_retreat(1901, 4);
        current_phase.units = vec![a("f", "par")];
        current_phase.territories = vec![
            Territory::new(Power::France, "par"),
            Territory::new(Power::France, "bre"),
            Territory::new(Power::France, "mar"),
            Territory::new(Power::France, "lon"),
            Territory::new(Power::France, "edi"),
            Territory::new(Power::France, "lvp"),
            Territory::new(Power::France, "ber"),
            Territory::new(Power::France, "mun"),
            Territory::new(Power::France, "kie"),
            Territory::new(Power::France, "vie"),
            Territory::new(Power::France, "bud"),
            Territory::new(Power::France, "tri"),
            Territory::new(Power::France, "rom"),
            Territory::new(Power::France, "ven"),
            Territory::new(Power::France, "nap"),
            Territory::new(Power::France, "mos"),
            Territory::new(Power::France, "war"),
            Territory::new(Power::France, "sev"),
        ];

        let mut context = PhaseContext::new();
        current_phase.close(&mut context);

        let phases = context.phases();
        assert_eq!(phases.len(), 3);
        assert!(matches!(phases[0].kind, PhaseKind::FallRetreat(_)));
        assert!(matches!(phases[1].kind, PhaseKind::Adjustment(_)));
        assert!(matches!(phases[2].kind, PhaseKind::Debrief(_)));
    }

    #[test]
    fn close_on_fall_retreat_without_solo_follows_normal_transition() {
        let mut current_phase = Phase::new_fall_retreat(1901, 4);
        current_phase.units = vec![a("f", "par")];
        current_phase.territories = vec![
            Territory::new(Power::France, "par"),
            Territory::new(Power::France, "bre"),
            Territory::new(Power::France, "mar"),
            Territory::new(Power::France, "lon"),
            Territory::new(Power::France, "edi"),
            Territory::new(Power::France, "lvp"),
            Territory::new(Power::France, "ber"),
            Territory::new(Power::France, "mun"),
            Territory::new(Power::France, "kie"),
            Territory::new(Power::France, "vie"),
            Territory::new(Power::France, "bud"),
            Territory::new(Power::France, "tri"),
            Territory::new(Power::France, "rom"),
            Territory::new(Power::France, "ven"),
            Territory::new(Power::France, "nap"),
            Territory::new(Power::France, "mos"),
            Territory::new(Power::France, "war"),
        ];

        let mut context = PhaseContext::new();
        current_phase.close(&mut context);

        let phases = context.phases();
        assert_eq!(phases.len(), 2);
        assert!(matches!(phases[0].kind, PhaseKind::FallRetreat(_)));
        assert!(matches!(phases[1].kind, PhaseKind::Adjustment(_)));
    }

    #[test]
    fn close_on_fall_retreat_skips_adjustment_for_inactive_power_only() {
        let mut current_phase = Phase::new_fall_retreat(1901, 4);
        current_phase.units = vec![a("f", "par"), a("f", "gas")];
        current_phase.territories = vec![Territory::new(Power::France, "par")];

        let mut context = PhaseContext::new();
        context.remove_power(&Power::France);
        current_phase.close(&mut context);

        let tail_phase = context.pop_phase().unwrap();
        assert_eq!(tail_phase.year, 1902);
        assert_eq!(tail_phase.index, 7);
        assert!(matches!(tail_phase.kind, PhaseKind::SpringMain(_)));
    }
    #[test]
    fn occupy_overwrites_existing_territory_owner() {
        let mut fall_retreat = Phase::new_fall_retreat(1901, 4);
        fall_retreat.units = vec![a("g", "par")];
        fall_retreat.territories = vec![Territory::new(Power::France, "par")];

        FallRetreatPhase {}.occupy(&mut fall_retreat);

        assert_eq!(fall_retreat.territories.len(), 1);
        assert_eq!(fall_retreat.territories[0], Territory::new(Power::Germany, "par"));
    }

    #[test]
    fn occupy_adds_unoccupied_land_territory() {
        let mut fall_retreat = Phase::new_fall_retreat(1901, 4);
        fall_retreat.units = vec![a("g", "gas")];

        FallRetreatPhase {}.occupy(&mut fall_retreat);

        assert_eq!(fall_retreat.territories, vec![Territory::new(Power::Germany, "gas")]);
    }

    #[test]
    fn occupy_does_not_add_water_territory() {
        let mut fall_retreat = Phase::new_fall_retreat(1901, 4);
        fall_retreat.units = vec![f("e", "nth")];

        FallRetreatPhase {}.occupy(&mut fall_retreat);

        assert!(fall_retreat.territories.is_empty());
    }

    #[test]
    fn count_supply_centers_excludes_non_supply_territories() {
        let mut phase = Phase::new_fall_retreat(1901, 4);
        phase.territories = vec![
            Territory::new(Power::France, "par"),
            Territory::new(Power::France, "gas"),
            Territory::new(Power::France, "pic"),
        ];

        assert_eq!(phase.count_supply_centers(&Power::France), 1);
    }
}

#[cfg(test)]
#[path = "test_datc_6_a.rs"]
mod test_datc_6_a;
#[cfg(test)]
#[path = "test_datc_6_b.rs"]
mod test_datc_6_b;
#[cfg(test)]
#[path = "test_datc_6_c.rs"]
mod test_datc_6_c;
#[cfg(test)]
#[path = "test_datc_6_d.rs"]
mod test_datc_6_d;
#[cfg(test)]
#[path = "test_datc_6_e.rs"]
mod test_datc_6_e;
#[cfg(test)]
#[path = "test_datc_6_f.rs"]
mod test_datc_6_f;
#[cfg(test)]
#[path = "test_datc_6_g.rs"]
mod test_datc_6_g;
#[cfg(test)]
#[path = "test_datc_6_h.rs"]
mod test_datc_6_h;
#[cfg(test)]
#[path = "test_datc_6_i.rs"]
mod test_datc_6_i;
#[cfg(test)]
#[path = "test_datc_6_j.rs"]
mod test_datc_6_j;
