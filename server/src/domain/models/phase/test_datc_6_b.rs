//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6B]
//!
//! * 6.B. TEST CASES, COASTAL ISSUES
//!
//! [DATC_6B]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.B

use crate::domain::models::order::*;
use crate::domain::models::phase::*;
use crate::domain::tests::f;
use crate::domain::tests::p;

/// 6.B.1. TEST CASE, MOVING WITH UNSPECIFIED COAST WHEN COAST IS NECESSARY
/// Coast is significant in this case:
///
/// France:
///     F Portugal - Spain
///
/// Move should fail.
#[test]
fn test_datc_6_b_1() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_por = f("f", "por");
    phase.data.units.push(unit_f_por);
    phase.data.orders.push(unit_f_por.move_to(p("spa")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_f_por));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.2. TEST CASE, MOVING WITH UNSPECIFIED COAST WHEN COAST IS NOT NECESSARY
/// There is only one coast possible in this case:
///
/// France:
///     F Gascony - Spain
///
/// Since the North Coast is the only coast that can be reached,
/// it seems logical that a move is attempted to the north coast of Spain. See issue 4.B.2.
/// I prefer that an attempt is made to the only possible coast, the north coast of Spain.
#[allow(unused)]
fn test_datc_6_b_2() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.3. TEST CASE, MOVING WITH WRONG COAST WHEN COAST IS NOT NECESSARY
/// If only one coast is possible, but the wrong coast can be specified.
///
/// France:
///     F Gascony - Spain(sc)
///
/// If the rules are given a lenient interpretation,
/// a move will be attempted to the north coast of Spain. However, this order is very precisely wrong.
/// The order should be declared illegal and fleet should hold. See issue 4.B.3.
#[test]
fn test_datc_6_b_3() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_gas = f("f", "gas");
    phase.data.units.push(unit_f_gas);
    phase.data.orders.push(unit_f_gas.move_to(p("spa_sc")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_f_gas));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.4. TEST CASE, SUPPORT TO UNREACHABLE COAST ALLOWED
/// A fleet can give support to a coast where it cannot go.
///
/// France:
///     F Gascony - Spain(nc)
///     F Marseilles Supports F Gascony - Spain(nc)
///
/// Italy:
///     F Western Mediterranean - Spain(sc)
///
/// Although the fleet in Marseilles cannot go to the north coast it can still support targeting the north coast.
/// So, the support is successful, the move of the fleet in Gascony succeeds and the move of the Italian fleet fails.
#[test]
fn test_datc_6_b_4() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_gas = f("f", "gas");
    let unit_f_mar = f("f", "mar");
    let unit_i_spa_nc = f("i", "wes");
    phase.data.units.push(unit_f_gas);
    phase.data.units.push(unit_f_mar);
    phase.data.units.push(unit_i_spa_nc);
    phase.data.orders.push(unit_f_gas.move_to(p("spa_nc")));
    phase.data.orders.push(unit_f_mar.support_move(unit_f_gas, p("spa_nc")));
    phase.data.orders.push(unit_i_spa_nc.move_to(p("spa_sc")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&f("f", "spa_nc")));
    assert!(phase.data.units.contains(&unit_f_mar));
    assert!(phase.data.units.contains(&unit_i_spa_nc));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.5. TEST CASE, SUPPORT FROM UNREACHABLE COAST NOT ALLOWED
/// A fleet cannot give support to an area that cannot be reached from the current coast of the fleet.
///
/// France:
///     F Marseilles - Gulf of Lyon
///     F Spain(nc) Supports F Marseilles - Gulf of Lyon
///
/// Italy:
///     F Gulf of Lyon Hold
///
/// The Gulf of Lyon cannot be reached from the North Coast of Spain.
/// Therefore, the support of Spain is illegal and the fleet in the Gulf of Lyon is not dislodged.
#[test]
fn test_datc_6_b_5() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_mar = f("f", "mar");
    let unit_f_spa_nc = f("f", "spa_nc");
    let unit_i_gol = f("i", "gol");
    phase.data.units.push(unit_f_mar);
    phase.data.units.push(unit_f_spa_nc);
    phase.data.units.push(unit_i_gol);
    phase.data.orders.push(unit_f_mar.move_to(p("gol")));
    phase.data.orders.push(unit_f_spa_nc.support_move(unit_f_mar, p("gol")));
    phase.data.orders.push(unit_i_gol.hold());
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_f_mar));
    assert!(phase.data.units.contains(&unit_f_spa_nc));
    assert!(phase.data.units.contains(&unit_i_gol));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.6. TEST CASE, SUPPORT CAN BE CUT WITH OTHER COAST
/// Support can be cut from the other coast.
///
/// England:
///     F Irish Sea Supports F North Atlantic Ocean - Mid-Atlantic Ocean
///     F North Atlantic Ocean - Mid-Atlantic Ocean
///
/// France:
///     F Spain(nc) Supports F Mid-Atlantic Ocean
///     F Mid-Atlantic Ocean Hold
///
/// Italy:
///     F Gulf of Lyon - Spain(sc)
///
/// The Italian fleet in the Gulf of Lyon will cut the support in Spain.
/// That means that the French fleet in the Mid Atlantic Ocean will be dislodged
/// by the English fleet in the North Atlantic Ocean.
#[test]
fn test_datc_6_b_6() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_iri = f("e", "iri");
    let unit_e_nat = f("e", "nat");
    let unit_f_spa_nc = f("f", "spa_nc");
    let mut unit_f_mid = f("f", "mid");
    let unit_i_gol = f("i", "gol");
    phase.data.units.push(unit_e_iri);
    phase.data.units.push(unit_e_nat);
    phase.data.units.push(unit_f_spa_nc);
    phase.data.units.push(unit_f_mid);
    phase.data.units.push(unit_i_gol);
    phase.data.orders.push(unit_e_iri.support_move(unit_e_nat, p("mid")));
    phase.data.orders.push(unit_e_nat.move_to(p("mid")));
    phase.data.orders.push(unit_f_spa_nc.support_hold(unit_f_mid));
    phase.data.orders.push(unit_f_mid.hold());
    phase.data.orders.push(unit_i_gol.move_to(p("spa_sc")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_e_iri));
    assert!(phase.data.units.contains(&f("e", "mid")));
    assert!(phase.data.units.contains(&unit_f_spa_nc));
    assert!(phase.data.units.contains(&unit_f_mid.set_dislodged_from(Some(p("nat")))));
    assert!(phase.data.units.contains(&unit_i_gol));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.7. TEST CASE, SUPPORTING OWN UNIT WITH UNSPECIFIED COAST
/// It is a little bit harsh to reject this.
///
/// France:
///     F Portugal Supports F Mid-Atlantic Ocean - Spain
///     F Mid-Atlantic Ocean - Spain(nc)
///
/// Italy:
///     F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
///     F Western Mediterranean - Spain(sc)
///
/// See issue 4.B.4.
/// I prefer that the support succeeds and the Italian fleet in the Western Mediterranean bounces.
/// However, if orders are checked on submission (such as in webbased play),
/// support without coast should not be given as an option.
#[allow(unused)]
fn test_datc_6_b_7() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.8. TEST CASE, SUPPORTING WITH UNSPECIFIED COAST WHEN ONLY ONE COAST IS POSSIBLE
/// If coast is omitted while only coast is possible,
/// it should be considered a poorly written order, that should be followed.
///
/// France:
///     F Portugal Supports F Gascony - Spain
///     F Gascony - Spain(nc)
///
/// Italy:
///     F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
///     F Western Mediterranean - Spain(sc)
///
/// Support of Portugal is successful
/// and the Italian fleet in the Western Mediterranean bounces with the French fleet from Gascony.
#[allow(unused)]
fn test_datc_6_b_8() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.9. TEST CASE, SUPPORTING WITH WRONG COAST
/// It should be possible to specify a coast and that coast should match.
///
/// France:
///     F Portugal Supports F Mid-Atlantic Ocean - Spain(nc)
///     F Mid-Atlantic Ocean - Spain(sc)
///
/// Italy:
///     F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
///     F Western Mediterranean - Spain(sc)
///
/// See issue 4.B.4. Support of Portugal is invalid
/// and the Italian fleet in the Western Mediterranean moves successfully.
#[test]
fn test_datc_6_b_9() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_mid = f("f", "mid");
    let unit_f_por = f("f", "por");
    let unit_i_gol = f("i", "gol");
    let unit_i_wes = f("i", "wes");
    phase.data.units.push(unit_f_mid);
    phase.data.units.push(unit_f_por);
    phase.data.units.push(unit_i_gol);
    phase.data.units.push(unit_i_wes);
    phase.data.orders.push(unit_f_por.support_move(unit_f_mid, p("spa_nc")));
    phase.data.orders.push(unit_f_mid.move_to(p("spa_sc")));
    phase.data.orders.push(unit_i_gol.support_move(unit_i_wes, p("spa_sc")));
    phase.data.orders.push(unit_i_wes.move_to(p("spa_sc")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_f_mid));
    assert!(phase.data.units.contains(&unit_f_por));
    assert!(phase.data.units.contains(&unit_i_gol));
    assert!(phase.data.units.contains(&f("i", "spa_sc")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.10. TEST CASE, UNIT ORDERED WITH WRONG COAST
/// A player might specify the wrong coast for the ordered unit.
/// France has a fleet on the south coast of Spain and orders:
///
/// France:
///     F Spain(nc) - Gulf of Lyon
///
/// If only perfect orders are accepted, then the move will fail,
/// but since the coast for the ordered unit has no purpose,
/// it might also be ignored (see issue 4.B.5).
/// I prefer that a move will be attempted.
#[allow(unused)]
fn test_datc_6_b_10() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.11. TEST CASE, COAST CANNOT BE ORDERED TO CHANGE
/// The coast cannot change by just ordering the other coast.
/// France has a fleet on the north coast of Spain and orders:
///
/// France:
///     F Spain(sc) - Gulf of Lyon
///
/// The move fails.
#[allow(unused)]
fn test_datc_6_b_11() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.12. TEST CASE, ARMY MOVEMENT WITH COASTAL SPECIFICATION
/// For armies the coasts are irrelevant:
///
/// France:
///     A Gascony - Spain(nc)
///
/// If only perfect orders are accepted, then the move will fail.
/// But it is also possible that coasts are ignored in this case
/// and a move will be attempted (see issue 4.B.6).
/// I prefer that a move will be attempted.
#[allow(unused)]
fn test_datc_6_b_12() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.13. TEST CASE, COASTAL CRAWL NOT ALLOWED
/// If a fleet is leaving a sector from a certain coast while in the opposite direction another fleet
/// is moving to another coast of the sector, it is still a head-to-head battle.
/// This has been decided in the great revision of the 1961 rules that resulted in the 1971 rules.
///
/// Turkey:
///     F Bulgaria(sc) - Constantinople
///     F Constantinople - Bulgaria(ec)
///
/// Both moves fail.
#[test]
fn test_datc_6_b_13() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_t_bul_sc = f("t", "bul_sc");
    let unit_t_con = f("t", "con");
    phase.data.units.push(unit_t_bul_sc);
    phase.data.units.push(unit_t_con);
    phase.data.orders.push(unit_t_bul_sc.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("bul_ec")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 2);
    assert!(phase.data.units.contains(&unit_t_bul_sc));
    assert!(phase.data.units.contains(&unit_t_con));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.B.14. TEST CASE, BUILDING WITH UNSPECIFIED COAST
/// Coast must be specified in certain build cases:
///
/// Russia:
///     Build F St Petersburg
///
/// See issue 4.B.7. Build fails.
#[allow(unused)]
fn test_datc_6_b_14() {
    // 建造命令に関するテストは対象外
}

/// 6.B.15. TEST CASE, SUPPORTING FOREIGN UNIT WITH UNSPECIFIED COAST
/// Opinions differ on this.
///
/// France:
///     F Portugal Supports F Mid-Atlantic Ocean - Spain
///
/// England:
///     F Mid-Atlantic Ocean - Spain(nc)
///
/// Italy:
///     F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
///     F Western Mediterranean - Spain(sc)
///
/// See issue 4.B.4.
/// Although the move to the north coast of Spain might be a surprise for France,
/// it is hard to believe that England somehow tricked France. Therefore,
/// I prefer that the support succeeds and the Italian fleet in the Western Mediterranean bounces.
/// However, if orders are checked on submission (such as in webbased play),
/// support without coast should not be given as an option.
#[allow(unused)]
fn test_datc_6_b_15() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}
