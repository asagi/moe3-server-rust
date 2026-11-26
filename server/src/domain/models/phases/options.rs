// ============================================================================
// imports
// ============================================================================

use std::collections::HashSet;

use strum::IntoEnumIterator;

use super::Order;
use super::OrderKind;
use super::Path;
use super::Phase;
use super::PhaseKind;
use super::Power;
use super::Province;
use super::Unit;

// ============================================================================
// definitions
// ============================================================================

///
/// 支援候補（支援されるユニットとその支援先）の構造体
///
/// `dests` の `None` は維持の支援を表す。
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SupportTarget {
    pub(crate) target: Unit,
    pub(crate) dests: Vec<Option<Province>>,
}

///
/// 輸送候補（輸送される陸軍とその移動先）の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConvoyTarget {
    pub(crate) target: Unit,
    pub(crate) dests: Vec<Province>,
}

///
/// メインフェイズにおけるユニットごとの命令候補の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MainUnitOptions {
    pub(crate) unit: Unit,
    /// 陸路（海軍は海路）で移動できる地域
    pub(crate) move_dests: Vec<Province>,
    /// すべての海軍が輸送に同意した場合に海路移動できる海岸（陸軍のみ）
    pub(crate) convoy_dests: Vec<Province>,
    /// 支援できるユニットとその支援先
    pub(crate) support_targets: Vec<SupportTarget>,
    /// 輸送できる陸軍とその移動先（海にいる海軍のみ）
    pub(crate) convoy_targets: Vec<ConvoyTarget>,
}

///
/// 増設できる地域の構造体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BuildableSite {
    pub(crate) location: Province,
    pub(crate) army: bool,
    pub(crate) fleet: bool,
}

///
/// 調整フェイズにおける国ごとの命令候補の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AdjustmentOptions {
    /// 増設できる最大数
    pub(crate) build_count: usize,
    /// 解体しなければならない数
    pub(crate) disband_count: usize,
    /// 増設できる地域（海岸が複数ある地域は海岸ごと）
    pub(crate) buildable: Vec<BuildableSite>,
}

///
/// 命令入力（解釈前の命令）の構造体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OrderInput {
    pub(crate) unit_power: Power,
    pub(crate) is_fleet: bool,
    pub(crate) location: Province,
    pub(crate) kind: OrderInputKind,
}

///
/// 命令入力の種類の列挙体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrderInputKind {
    Hold,
    Move {
        dest: Province,
        via_convoy: bool,
    },
    Support {
        target_location: Province,
        target_dest: Option<Province>,
    },
    Convoy {
        target_location: Province,
        target_dest: Province,
    },
    Retreat {
        dest: Province,
    },
    Build,
    Disband,
}

///
/// 命令の却下理由の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderRejection {
    /// 却下された命令の対象地域（命令数の過不足など、特定の命令によらない場合は `None`）
    pub(crate) location: Option<Province>,
    pub(crate) message: String,
}

/// 命令の却下理由の構造体の実装
impl OrderRejection {
    fn at(location: Province, message: impl Into<String>) -> Self {
        Self {
            location: Some(location),
            message: message.into(),
        }
    }

    fn general(message: impl Into<String>) -> Self {
        Self {
            location: None,
            message: message.into(),
        }
    }
}

///
/// フェイズ構造体の実装（命令候補・命令確定）
///
impl Phase {
    ///
    /// メインフェイズかどうかを返す
    ///
    pub(crate) fn is_main(&self) -> bool {
        matches!(self.kind, PhaseKind::SpringMain(_) | PhaseKind::FallMain(_))
    }

    ///
    /// 撤退フェイズかどうかを返す
    ///
    pub(crate) fn is_retreat(&self) -> bool {
        matches!(self.kind, PhaseKind::SpringRetreat(_) | PhaseKind::FallRetreat(_))
    }

    ///
    /// 調整フェイズかどうかを返す
    ///
    pub(crate) fn is_adjustment(&self) -> bool {
        matches!(self.kind, PhaseKind::Adjustment(_))
    }

    ///
    /// このフェイズで操作が必要な国を返す
    ///
    /// - メイン：補給都市かユニットが残っている国（補給都市でない陸地の占領だけが残っている国は対象外）
    /// - 撤退：撃退されたユニットがある国
    /// - 調整：補給都市数とユニット数が異なる国
    /// - 開戦前夜・感想戦：なし
    ///
    pub(crate) fn powers_requiring_action(&self) -> HashSet<Power> {
        match self.kind {
            PhaseKind::SpringMain(_) | PhaseKind::FallMain(_) => Power::iter()
                .filter(|power| self.count_supply_centers(power) > 0 || self.count_units(power) > 0)
                .collect(),
            PhaseKind::SpringRetreat(_) | PhaseKind::FallRetreat(_) => {
                self.units.iter().filter(|u| u.dislodged).map(|u| u.power).collect()
            }
            PhaseKind::Adjustment(_) => Power::iter()
                .filter(|power| self.count_supply_centers(power) != self.count_units(power))
                .collect(),
            PhaseKind::Ready(_) | PhaseKind::Debrief(_) => HashSet::new(),
        }
    }

    ///
    /// 海にいる海軍の所在地コードの集合を返す
    ///
    fn fleet_water_codes(&self) -> HashSet<&'static str> {
        self.units
            .iter()
            .filter(|u| u.is_fleet() && u.location.is_water())
            .map(|u| u.location.code_with_coast())
            .collect()
    }

    ///
    /// ユニットが陸路（海軍は海路）で移動できる地域を返す
    ///
    fn move_dests_of(unit: &Unit) -> Vec<Province> {
        Path::move_dest_codes(unit)
            .into_iter()
            .filter_map(Province::from_code)
            .collect()
    }

    ///
    /// 陸軍が、指定した海域の海軍すべての輸送によって海路移動できる海岸を返す
    ///
    /// 海軍と、内陸にいる陸軍には空配列を返す。
    ///
    fn convoy_dests_of(unit: &Unit, waters: &HashSet<&str>) -> Vec<Province> {
        if unit.is_fleet() || !unit.location.is_coast() {
            return Vec::new();
        }
        let origin = unit.location.code();
        Province::all()
            .filter(|p| p.is_coast() && p.code_with_coast() == p.code() && p.code() != origin)
            .filter(|p| Path::is_reachable_by_sea(origin, p.code(), waters))
            .collect()
    }

    ///
    /// メインフェイズで、盤面のすべてのユニットについて命令候補を返す
    ///
    pub(crate) fn main_options(&self) -> Vec<MainUnitOptions> {
        let waters = self.fleet_water_codes();

        // 各ユニットの移動先（陸路・海路）を先に求めておく
        let reachable: Vec<(Unit, Vec<Province>, Vec<Province>)> = self
            .units
            .iter()
            .map(|u| (*u, Self::move_dests_of(u), Self::convoy_dests_of(u, &waters)))
            .collect();

        reachable
            .iter()
            .map(|(unit, move_dests, convoy_dests)| {
                let support_targets = reachable
                    .iter()
                    .filter(|(target, _, _)| target.location.code() != unit.location.code())
                    .filter_map(|(target, target_moves, target_convoys)| {
                        let mut dests: Vec<Option<Province>> = Vec::new();
                        // 維持の支援
                        if Path::can_unit_support_to(unit, unit.location.code_with_coast(), target.location.code_with_coast()) {
                            dests.push(None);
                        }
                        // 移動の支援（支援するユニットが移動できる地域と、支援されるユニットの移動先が重なるもの）
                        for dest in target_moves.iter().chain(target_convoys.iter()) {
                            if dest.code() == unit.location.code() || dests.contains(&Some(*dest)) {
                                continue;
                            }
                            if Path::can_unit_support_to(unit, unit.location.code_with_coast(), dest.code_with_coast()) {
                                dests.push(Some(*dest));
                            }
                        }
                        (!dests.is_empty()).then_some(SupportTarget { target: *target, dests })
                    })
                    .collect();

                let convoy_targets = if unit.is_fleet() && unit.location.is_water() {
                    reachable
                        .iter()
                        .filter(|(target, _, _)| !target.is_fleet())
                        .filter_map(|(target, _, target_convoys)| {
                            // この海軍を通る経路で輸送できる移動先
                            let fleet_origin = unit.location.code_with_coast();
                            if !Path::is_reachable_by_sea(fleet_origin, target.location.code(), &waters) {
                                return None;
                            }
                            let dests: Vec<Province> = target_convoys
                                .iter()
                                .filter(|d| Path::is_reachable_by_sea(fleet_origin, d.code(), &waters))
                                .copied()
                                .collect();
                            (!dests.is_empty()).then_some(ConvoyTarget { target: *target, dests })
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                MainUnitOptions {
                    unit: *unit,
                    move_dests: move_dests.clone(),
                    convoy_dests: convoy_dests.clone(),
                    support_targets,
                    convoy_targets,
                }
            })
            .collect()
    }

    ///
    /// 撤退フェイズで、撃退されたユニットが撤退できる地域を返す
    ///
    /// 攻撃元（海路で攻められた場合を除く）・スタンドオフが起きた地域・ほかのユニットがいる地域を除く。
    ///
    pub(crate) fn retreat_dests(&self, unit: &Unit) -> Vec<Province> {
        Self::move_dests_of(unit)
            .into_iter()
            .filter(|dest| unit.dislodged_from.is_none_or(|from| from.code() != dest.code()))
            .filter(|dest| !self.standoff_codes.iter().any(|code| code.get(..3) == Some(dest.code())))
            .filter(|dest| !self.units.iter().any(|u| u.location.code() == dest.code()))
            .collect()
    }

    ///
    /// 調整フェイズで、指定した国の増設・解体の候補を返す
    ///
    pub(crate) fn adjustment_options(&self, power: &Power) -> AdjustmentOptions {
        let sc_count = self.count_supply_centers(power);
        let unit_count = self.count_units(power);
        let build_count = sc_count.saturating_sub(unit_count);
        let disband_count = unit_count.saturating_sub(sc_count);

        let mut buildable = Vec::new();
        if build_count > 0 {
            for province in Province::all().filter(|p| p.code_with_coast() == p.code()) {
                let code = province.code();
                if !Province::is_home_sc(code, power) {
                    continue;
                }
                if !self.territories.iter().any(|t| t.code() == code && &t.power == power) {
                    continue;
                }
                if self.units.iter().any(|u| u.location.code() == code) {
                    continue;
                }
                if province.has_coast_variants() {
                    // 海岸が複数ある地域は、陸軍は地域に、海軍は海岸ごとに分ける
                    buildable.push(BuildableSite {
                        location: province,
                        army: true,
                        fleet: false,
                    });
                    for coast in Province::all().filter(|p| p.code() == code && p.code_with_coast() != code) {
                        buildable.push(BuildableSite {
                            location: coast,
                            army: false,
                            fleet: true,
                        });
                    }
                } else {
                    let army = Path::can_unit_exist_at(&Unit::new_army(*power, province), code);
                    let fleet = Path::can_unit_exist_at(&Unit::new_fleet(*power, province), code);
                    buildable.push(BuildableSite {
                        location: province,
                        army,
                        fleet,
                    });
                }
            }
        }

        AdjustmentOptions {
            build_count,
            disband_count,
            buildable,
        }
    }

    ///
    /// 指定した国の命令を、入力された命令で丸ごと置き換える
    ///
    /// 入力に候補にない命令が 1 つでもあれば、何も変えずに却下理由を返す。
    ///
    /// - メイン：入力に含まれない自国ユニットは維持。自国の仮定命令は入力のものだけにする（維持の仮定命令は保存しない）
    /// - 撤退：入力に含まれない自国の撃退されたユニットは解体
    /// - 調整：入力の増設・解体だけにする（増設は増設可能数以下、解体はちょうど解体必要数）
    ///
    pub(crate) fn replace_orders(&mut self, power: Power, inputs: &[OrderInput]) -> Result<(), OrderRejection> {
        if self.is_main() {
            self.replace_main_orders(power, inputs)
        } else if self.is_retreat() {
            self.replace_retreat_orders(power, inputs)
        } else if self.is_adjustment() {
            self.replace_adjustment_orders(power, inputs)
        } else {
            Err(OrderRejection::general("orders cannot be submitted in this phase"))
        }
    }

    /// 命令入力の対象ユニットを盤面から探す
    fn find_input_unit(&self, input: &OrderInput) -> Result<Unit, OrderRejection> {
        self.units
            .iter()
            .find(|u| u.power == input.unit_power && u.is_fleet() == input.is_fleet && u.location == input.location)
            .copied()
            .ok_or_else(|| OrderRejection::at(input.location, "unit not found at the location"))
    }

    /// メインフェイズの命令を置き換える
    fn replace_main_orders(&mut self, power: Power, inputs: &[OrderInput]) -> Result<(), OrderRejection> {
        let options = self.main_options();
        let mut own_orders: Vec<Order> = Vec::new();
        let mut assumed_orders: Vec<Order> = Vec::new();
        let mut seen: Vec<Province> = Vec::new();

        for input in inputs {
            let unit = self.find_input_unit(input)?;
            if seen.contains(&unit.location) {
                return Err(OrderRejection::at(input.location, "duplicate orders for the same unit"));
            }
            seen.push(unit.location);

            let option = options
                .iter()
                .find(|o| o.unit == unit)
                .ok_or_else(|| OrderRejection::at(input.location, "unit cannot be ordered"))?;
            let find_target = |location: Province| -> Result<Unit, OrderRejection> {
                self.units
                    .iter()
                    .find(|u| u.location.code() == location.code())
                    .copied()
                    .ok_or_else(|| OrderRejection::at(input.location, "target unit not found"))
            };

            let order = match input.kind {
                OrderInputKind::Hold => Order::new_hold(power, unit),
                OrderInputKind::Move { dest, via_convoy } => {
                    let candidates = if via_convoy {
                        &option.convoy_dests
                    } else {
                        &option.move_dests
                    };
                    if !candidates.contains(&dest) {
                        return Err(OrderRejection::at(input.location, "destination is not reachable"));
                    }
                    let mut order = Order::new_move(power, unit, dest);
                    if via_convoy {
                        order.set_via_convoy();
                    }
                    order
                }
                OrderInputKind::Support {
                    target_location,
                    target_dest,
                } => {
                    let target = find_target(target_location)?;
                    let valid = option
                        .support_targets
                        .iter()
                        .any(|s| s.target == target && s.dests.contains(&target_dest));
                    if !valid {
                        return Err(OrderRejection::at(input.location, "support target is not available"));
                    }
                    Order::new_support(power, unit, target, target_dest)
                }
                OrderInputKind::Convoy {
                    target_location,
                    target_dest,
                } => {
                    let target = find_target(target_location)?;
                    let valid = option
                        .convoy_targets
                        .iter()
                        .any(|c| c.target == target && c.dests.contains(&target_dest));
                    if !valid {
                        return Err(OrderRejection::at(input.location, "convoy target is not available"));
                    }
                    Order::new_convoy(power, unit, target, target_dest)
                }
                OrderInputKind::Retreat { .. } | OrderInputKind::Build | OrderInputKind::Disband => {
                    return Err(OrderRejection::at(
                        input.location,
                        "order kind is not available in a main phase",
                    ));
                }
            };

            if unit.power == power {
                own_orders.push(order);
            } else if !matches!(order.kind, OrderKind::Hold(_)) {
                assumed_orders.push(order);
            }
        }

        // 仮定命令は命令列の末尾に置く必要がある（裁定処理が非仮定命令を先頭から数えるため）
        let mut non_assumed: Vec<Order> = Vec::new();
        let mut assumed: Vec<Order> = Vec::new();
        let mut ordered_units: Vec<Province> = Vec::new();
        for order in &self.orders {
            if order.is_assumed() {
                if order.power != power {
                    assumed.push(*order);
                }
                continue;
            }
            if order.unit.power != power {
                non_assumed.push(*order);
                continue;
            }
            // 自国ユニットの命令は、入力があれば置き換え、なければ維持にする
            if ordered_units.contains(&order.unit.location) {
                continue;
            }
            ordered_units.push(order.unit.location);
            let replaced = own_orders
                .iter()
                .find(|o| o.unit.location == order.unit.location)
                .copied()
                .unwrap_or_else(|| Order::new_hold(power, order.unit));
            non_assumed.push(replaced);
        }
        // 命令が未登録の自国ユニットがあれば追加する
        for unit in self.units.iter().filter(|u| u.power == power) {
            if ordered_units.contains(&unit.location) {
                continue;
            }
            ordered_units.push(unit.location);
            let order = own_orders
                .iter()
                .find(|o| o.unit.location == unit.location)
                .copied()
                .unwrap_or_else(|| Order::new_hold(power, *unit));
            non_assumed.push(order);
        }
        assumed.extend(assumed_orders);
        non_assumed.extend(assumed);
        self.orders = non_assumed;
        Ok(())
    }

    /// 撤退フェイズの命令を置き換える
    fn replace_retreat_orders(&mut self, power: Power, inputs: &[OrderInput]) -> Result<(), OrderRejection> {
        let mut new_orders: Vec<Order> = Vec::new();

        for input in inputs {
            if input.unit_power != power {
                return Err(OrderRejection::at(
                    input.location,
                    "only own units can be ordered in a retreat phase",
                ));
            }
            let unit = self
                .units
                .iter()
                .find(|u| u.power == power && u.is_fleet() == input.is_fleet && u.location == input.location && u.dislodged)
                .copied()
                .ok_or_else(|| OrderRejection::at(input.location, "dislodged unit not found at the location"))?;
            if new_orders.iter().any(|o| o.unit == unit) {
                return Err(OrderRejection::at(input.location, "duplicate orders for the same unit"));
            }

            let order = match input.kind {
                OrderInputKind::Retreat { dest } => {
                    if !self.retreat_dests(&unit).contains(&dest) {
                        return Err(OrderRejection::at(input.location, "retreat destination is not available"));
                    }
                    Order::new_retreat(power, unit, dest)
                }
                OrderInputKind::Disband => Order::new_disband(power, unit),
                _ => {
                    return Err(OrderRejection::at(
                        input.location,
                        "order kind is not available in a retreat phase",
                    ));
                }
            };
            new_orders.push(order);
        }

        let mut orders: Vec<Order> = self.orders.iter().filter(|o| o.unit.power != power).copied().collect();
        for unit in self.units.iter().filter(|u| u.power == power && u.dislodged) {
            let order = new_orders
                .iter()
                .find(|o| o.unit == *unit)
                .copied()
                .unwrap_or_else(|| Order::new_disband(power, *unit));
            orders.push(order);
        }
        self.orders = orders;
        Ok(())
    }

    /// 調整フェイズの命令を置き換える
    fn replace_adjustment_orders(&mut self, power: Power, inputs: &[OrderInput]) -> Result<(), OrderRejection> {
        let options = self.adjustment_options(&power);
        let mut new_orders: Vec<Order> = Vec::new();
        let mut build_codes: Vec<&'static str> = Vec::new();
        let mut disbanded: Vec<Unit> = Vec::new();

        for input in inputs {
            if input.unit_power != power {
                return Err(OrderRejection::at(
                    input.location,
                    "only own units can be ordered in an adjustment phase",
                ));
            }
            match input.kind {
                OrderInputKind::Build => {
                    let site = options
                        .buildable
                        .iter()
                        .find(|s| s.location == input.location)
                        .ok_or_else(|| OrderRejection::at(input.location, "location is not buildable"))?;
                    if (input.is_fleet && !site.fleet) || (!input.is_fleet && !site.army) {
                        return Err(OrderRejection::at(
                            input.location,
                            "unit kind cannot be built at the location",
                        ));
                    }
                    if build_codes.contains(&input.location.code()) {
                        return Err(OrderRejection::at(input.location, "duplicate builds at the same location"));
                    }
                    build_codes.push(input.location.code());
                    let unit = if input.is_fleet {
                        Unit::new_fleet(power, input.location)
                    } else {
                        Unit::new_army(power, input.location)
                    };
                    new_orders.push(Order::new_build(power, unit));
                }
                OrderInputKind::Disband => {
                    let unit = self.find_input_unit(input)?;
                    if disbanded.contains(&unit) {
                        return Err(OrderRejection::at(input.location, "duplicate orders for the same unit"));
                    }
                    disbanded.push(unit);
                    new_orders.push(Order::new_disband(power, unit));
                }
                _ => {
                    return Err(OrderRejection::at(
                        input.location,
                        "order kind is not available in an adjustment phase",
                    ));
                }
            }
        }

        if build_codes.len() > options.build_count {
            return Err(OrderRejection::general(format!(
                "too many builds: at most {} allowed",
                options.build_count
            )));
        }
        if disbanded.len() != options.disband_count {
            return Err(OrderRejection::general(format!(
                "exactly {} disbands required",
                options.disband_count
            )));
        }

        let mut orders: Vec<Order> = self.orders.iter().filter(|o| o.power != power).copied().collect();
        orders.extend(new_orders);
        self.orders = orders;
        Ok(())
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test::a;
    use crate::domain::test::f;
    use crate::domain::test::p;
    use crate::domain::test::t;

    fn main_phase(units: Vec<Unit>) -> Phase {
        let mut phase = Phase::new_spring_main(1900, 0);
        phase.orders = units.iter().map(|u| u.hold()).collect();
        phase.units = units;
        phase
    }

    fn options_of(phase: &Phase, location: &str) -> MainUnitOptions {
        phase
            .main_options()
            .into_iter()
            .find(|o| o.unit.location == p(location))
            .expect("options exist")
    }

    fn codes(provinces: &[Province]) -> Vec<&'static str> {
        let mut codes: Vec<&'static str> = provinces.iter().map(|p| p.code_with_coast()).collect();
        codes.sort_unstable();
        codes
    }

    fn input(power: &str, is_fleet: bool, location: &str, kind: OrderInputKind) -> OrderInput {
        OrderInput {
            unit_power: Power::from_symbol(power).expect("valid power"),
            is_fleet,
            location: p(location),
            kind,
        }
    }

    #[test]
    fn move_dests_distinguish_land_and_sea() {
        let phase = main_phase(vec![a("f", "bre"), f("f", "mid")]);
        assert_eq!(codes(&options_of(&phase, "bre").move_dests), vec!["gas", "par", "pic"]);
        assert_eq!(
            codes(&options_of(&phase, "mid").move_dests),
            vec!["bre", "eng", "gas", "iri", "naf", "nat", "por", "spa_nc", "spa_sc", "wes"]
        );
    }

    #[test]
    fn convoy_dests_are_empty_for_inland_army() {
        let phase = main_phase(vec![a("f", "par"), f("f", "eng")]);
        assert!(options_of(&phase, "par").convoy_dests.is_empty());
    }

    #[test]
    fn convoy_dests_are_empty_without_fleets() {
        let phase = main_phase(vec![a("f", "bre"), a("e", "lon"), f("e", "edi")]);
        assert!(options_of(&phase, "bre").convoy_dests.is_empty());
        assert!(options_of(&phase, "lon").convoy_dests.is_empty());
    }

    #[test]
    fn convoy_dests_are_empty_for_fleet() {
        let phase = main_phase(vec![f("f", "bre"), f("f", "eng")]);
        assert!(options_of(&phase, "bre").convoy_dests.is_empty());
    }

    #[test]
    fn convoy_dests_follow_chained_fleets() {
        // ロンドン → 北海 → ノルウェー海 → ノルウェー海の先の海岸
        let phase = main_phase(vec![a("e", "lon"), f("e", "nth"), f("g", "nrg")]);
        let dests = codes(&options_of(&phase, "lon").convoy_dests);
        for code in ["bel", "den", "edi", "hol", "nwy", "yor", "cly"] {
            assert!(dests.contains(&code), "{} should be included: {:?}", code, dests);
        }
        assert!(!dests.contains(&"lon"));
        assert!(!dests.contains(&"bre"));
    }

    #[test]
    fn support_targets_include_hold_and_move_support() {
        let phase = main_phase(vec![a("f", "par"), a("f", "bur"), a("g", "mun")]);
        let options = options_of(&phase, "par");
        let bur = options
            .support_targets
            .iter()
            .find(|s| s.target.location == p("bur"))
            .expect("bur is supportable");
        assert_eq!(bur.dests[0], None);
        assert!(bur.dests.contains(&Some(p("pic"))));
        assert!(bur.dests.contains(&Some(p("gas"))));
        assert!(!bur.dests.contains(&Some(p("mun"))));
        // 支援するユニット自身の地域への移動は支援できない
        assert!(!bur.dests.contains(&Some(p("par"))));
        let mun = options
            .support_targets
            .iter()
            .find(|s| s.target.location == p("mun"))
            .expect("mun is supportable");
        assert_eq!(mun.dests, vec![Some(p("bur"))]);
    }

    #[test]
    fn support_targets_include_convoy_destinations() {
        let phase = main_phase(vec![a("e", "lon"), f("e", "nth"), a("g", "ruh")]);
        let options = options_of(&phase, "ruh");
        let lon = options
            .support_targets
            .iter()
            .find(|s| s.target.location == p("lon"))
            .expect("lon is supportable via convoy");
        assert!(lon.dests.contains(&Some(p("bel"))));
        assert!(lon.dests.contains(&Some(p("hol"))));
    }

    #[test]
    fn army_can_support_fleet_on_split_coast() {
        let phase = main_phase(vec![a("f", "gas"), f("f", "spa_nc")]);
        let options = options_of(&phase, "gas");
        let spa = options
            .support_targets
            .iter()
            .find(|s| s.target.location == p("spa_nc"))
            .expect("spa is supportable");
        assert!(spa.dests.contains(&None));
    }

    #[test]
    fn convoy_targets_pass_through_the_fleet() {
        let phase = main_phase(vec![a("e", "lon"), f("e", "nth"), f("e", "eng"), a("e", "wal")]);
        let nth = options_of(&phase, "nth");
        let lon = nth
            .convoy_targets
            .iter()
            .find(|c| c.target.location == p("lon"))
            .expect("lon is convoyable");
        assert!(lon.dests.contains(&p("nwy")));
        assert!(lon.dests.contains(&p("bre")));
        // ウェールズは北海に面していないが、イギリス海峡経由で北海を通れる
        let wal = nth
            .convoy_targets
            .iter()
            .find(|c| c.target.location == p("wal"))
            .expect("wal is convoyable through nth");
        assert!(wal.dests.contains(&p("nwy")));
        // 海岸にいる海軍は輸送できない
        let phase = main_phase(vec![a("e", "lon"), f("e", "yor"), f("e", "nth")]);
        assert!(options_of(&phase, "yor").convoy_targets.is_empty());
    }

    #[test]
    fn retreat_dests_exclude_attacker_origin_standoff_and_occupied() {
        let mut phase = Phase::new_spring_retreat(1901, 2);
        let mut dislodged = a("f", "bur");
        dislodged.set_dislodged_from(Some(p("mun")));
        phase.units = vec![dislodged, a("g", "bur"), a("g", "par")];
        phase.standoff_codes = vec!["pic".to_string()];

        let dests = codes(&phase.retreat_dests(&phase.units[0].clone()));
        assert_eq!(dests, vec!["bel", "gas", "mar", "ruh"]);
    }

    #[test]
    fn retreat_dests_allow_attacker_origin_when_dislodged_via_convoy() {
        let mut phase = Phase::new_spring_retreat(1901, 2);
        let mut dislodged = a("f", "bre");
        dislodged.set_dislodged_via_convoy();
        phase.units = vec![dislodged, a("e", "bre")];
        assert_eq!(
            codes(&phase.retreat_dests(&phase.units[0].clone())),
            vec!["gas", "par", "pic"]
        );
    }

    #[test]
    fn adjustment_options_split_coasts() {
        let mut phase = Phase::new_adjustment(1901, 4);
        phase.units = vec![a("r", "war")];
        phase.territories = vec![t("r", "stp"), t("r", "mos"), t("r", "war"), t("r", "sev")];
        let options = phase.adjustment_options(&Power::Russia);
        assert_eq!(options.build_count, 3);
        assert_eq!(options.disband_count, 0);
        let sites: Vec<(&str, bool, bool)> = options
            .buildable
            .iter()
            .map(|s| (s.location.code_with_coast(), s.army, s.fleet))
            .collect();
        assert!(sites.contains(&("stp", true, false)));
        assert!(sites.contains(&("stp_nc", false, true)));
        assert!(sites.contains(&("stp_sc", false, true)));
        assert!(sites.contains(&("mos", true, false)));
        assert!(sites.contains(&("sev", true, true)));
        assert!(!sites.iter().any(|s| s.0 == "war"));
    }

    #[test]
    fn adjustment_options_exclude_lost_or_foreign_supply_centers() {
        let mut phase = Phase::new_adjustment(1901, 4);
        phase.units = vec![];
        phase.territories = vec![t("g", "par"), t("f", "bel"), t("f", "bre")];
        let options = phase.adjustment_options(&Power::France);
        let sites: Vec<&str> = options.buildable.iter().map(|s| s.location.code_with_coast()).collect();
        assert_eq!(sites, vec!["bre"]);
    }

    #[test]
    fn powers_requiring_action_in_retreat_includes_convoy_dislodged_units() {
        let mut phase = Phase::new_spring_retreat(1901, 2);
        let mut dislodged = a("f", "bre");
        dislodged.set_dislodged_via_convoy();
        phase.units = vec![dislodged, a("e", "bre")];
        assert_eq!(phase.powers_requiring_action(), HashSet::from([Power::France]));
    }

    #[test]
    fn replace_main_orders_replaces_own_and_assumed_orders() {
        let mut phase = main_phase(vec![a("f", "par"), a("f", "mar"), a("g", "mun"), a("e", "lon")]);
        // イギリスの既存の仮定命令は残る
        phase.orders.push(a("g", "mun").move_to(p("bur")).assumed_by(Power::England));
        // フランスの既存の仮定命令は置き換えで消える
        phase.orders.push(a("g", "mun").move_to(p("ruh")).assumed_by(Power::France));
        phase.orders[0] = a("f", "par").move_to(p("bur"));

        phase
            .replace_orders(
                Power::France,
                &[
                    input(
                        "f",
                        false,
                        "mar",
                        OrderInputKind::Move {
                            dest: p("bur"),
                            via_convoy: false,
                        },
                    ),
                    input(
                        "g",
                        false,
                        "mun",
                        OrderInputKind::Move {
                            dest: p("bur"),
                            via_convoy: false,
                        },
                    ),
                ],
            )
            .expect("valid orders");

        let french: Vec<String> = phase
            .orders
            .iter()
            .filter(|o| o.power == Power::France)
            .map(|o| o.to_string())
            .collect();
        assert_eq!(french, vec!["A Par Holds", "A Mar - Bur", "A Mun - Bur"]);
        // 仮定命令は末尾に並ぶ
        let first_assumed = phase.orders.iter().position(|o| o.is_assumed()).expect("assumed exists");
        assert!(phase.orders[first_assumed..].iter().all(|o| o.is_assumed()));
        assert!(
            phase
                .orders
                .iter()
                .any(|o| o.power == Power::England && o.is_assumed() && o.to_string() == "A Mun - Bur")
        );
        // ほかの国の命令はそのまま
        assert!(phase.orders.iter().any(|o| o.power == Power::Germany && !o.is_assumed()));
    }

    #[test]
    fn replace_main_orders_rejects_unavailable_order_without_changes() {
        let mut phase = main_phase(vec![a("f", "par"), a("f", "mar")]);
        let before = phase.orders.clone();
        let result = phase.replace_orders(
            Power::France,
            &[
                input(
                    "f",
                    false,
                    "mar",
                    OrderInputKind::Move {
                        dest: p("bur"),
                        via_convoy: false,
                    },
                ),
                input(
                    "f",
                    false,
                    "par",
                    OrderInputKind::Move {
                        dest: p("mun"),
                        via_convoy: false,
                    },
                ),
            ],
        );
        assert_eq!(result.unwrap_err().location, Some(p("par")));
        assert_eq!(phase.orders, before);
    }

    #[test]
    fn replace_main_orders_accepts_convoy_route() {
        let mut phase = main_phase(vec![a("e", "lon"), f("e", "nth")]);
        phase
            .replace_orders(
                Power::England,
                &[
                    input(
                        "e",
                        false,
                        "lon",
                        OrderInputKind::Move {
                            dest: p("nwy"),
                            via_convoy: true,
                        },
                    ),
                    input(
                        "e",
                        true,
                        "nth",
                        OrderInputKind::Convoy {
                            target_location: p("lon"),
                            target_dest: p("nwy"),
                        },
                    ),
                ],
            )
            .expect("valid orders");
        assert!(phase.orders.iter().any(|o| o.via_convoy()));
        // 陸路で移動できない地域への陸路移動は却下される
        let result = phase.replace_orders(
            Power::England,
            &[input(
                "e",
                false,
                "lon",
                OrderInputKind::Move {
                    dest: p("nwy"),
                    via_convoy: false,
                },
            )],
        );
        assert!(result.is_err());
    }

    #[test]
    fn replace_retreat_orders_defaults_to_disband() {
        let mut phase = Phase::new_spring_retreat(1901, 2);
        let mut bur = a("f", "bur");
        bur.set_dislodged_from(Some(p("mun")));
        let mut bre = f("f", "bre");
        bre.set_dislodged_from(Some(p("eng")));
        phase.units = vec![bur, bre, a("g", "bur"), f("e", "bre")];
        phase.orders = vec![bur.disband(), bre.disband()];

        phase
            .replace_orders(
                Power::France,
                &[input("f", false, "bur", OrderInputKind::Retreat { dest: p("gas") })],
            )
            .expect("valid orders");
        let orders: Vec<String> = phase.orders.iter().map(|o| o.to_string()).collect();
        assert_eq!(orders, vec!["A Bur - Gas", "Remove F Bre"]);

        // 攻撃元への撤退は却下される
        let result = phase.replace_orders(
            Power::France,
            &[input("f", false, "bur", OrderInputKind::Retreat { dest: p("mun") })],
        );
        assert!(result.is_err());
    }

    #[test]
    fn replace_adjustment_orders_validates_counts() {
        let mut phase = Phase::new_adjustment(1901, 4);
        phase.units = vec![a("f", "par")];
        phase.territories = vec![t("f", "par"), t("f", "bre"), t("f", "mar")];

        // 増設が多すぎる
        let result = phase.replace_orders(
            Power::France,
            &[
                input("f", true, "bre", OrderInputKind::Build),
                input("f", false, "mar", OrderInputKind::Build),
                input("f", false, "bre", OrderInputKind::Build),
            ],
        );
        assert!(result.is_err());

        // 海軍は内陸に増設できない
        let result = phase.replace_orders(Power::France, &[input("f", true, "par", OrderInputKind::Build)]);
        assert!(result.is_err());

        phase
            .replace_orders(Power::France, &[input("f", true, "bre", OrderInputKind::Build)])
            .expect("valid orders");
        assert_eq!(phase.orders.len(), 1);
        assert_eq!(phase.orders[0].to_string(), "Build F Bre");

        // 解体の数が合わない
        let mut phase = Phase::new_adjustment(1901, 4);
        phase.units = vec![a("f", "par"), a("f", "bur"), a("f", "gas")];
        phase.territories = vec![t("f", "par")];
        let result = phase.replace_orders(Power::France, &[input("f", false, "bur", OrderInputKind::Disband)]);
        assert!(result.is_err());
        phase
            .replace_orders(
                Power::France,
                &[
                    input("f", false, "bur", OrderInputKind::Disband),
                    input("f", false, "gas", OrderInputKind::Disband),
                ],
            )
            .expect("valid orders");
        assert_eq!(phase.orders.len(), 2);
    }
}
