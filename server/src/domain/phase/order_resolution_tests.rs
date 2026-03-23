//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6]
//!
//! * 6.A. TEST CASES, BASIC CHECKS
//! * 6.B. TEST CASES, COASTAL ISSUES
//! * 6.C. TEST CASES, CIRCULAR MOVEMENT
//! * 6.D. TEST CASES, SUPPORTS AND DISLODGES
//!
//! [DATC_6]: https://webdiplomacy.net/doc/DATC_v3_0.html#6

use super::super::order::*;
use super::super::phase::order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.A.1. TEST CASE, MOVING TO AN AREA THAT IS NOT A NEIGHBOUR
///  Check if an illegal move (without convoy) will fail.
///
///  England:
///  F North Sea - Picardy
///  Order should fail.
#[test]
fn test_datc_6_a_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit = Unit::new_fleet(Power::England, p("nth"));
    let order = unit.move_to(p("pic"));
    phase.data.orders.push(order);
    resolve_orders_for_order_phase(&mut phase);
    assert!(phase.data.orders[0].is_invalid());
}

/// 6.A.2. TEST CASE, MOVE ARMY TO SEA
/// Check if an army could not be moved to open sea.
///
/// England:
/// A Liverpool - Irish Sea
/// Order should fail.
#[test]
fn test_datc_6_a_2() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit = Unit::new_army(Power::England, p("lvp"));
    let order = unit.move_to(p("iri"));
    phase.data.orders.push(order);
    resolve_orders_for_order_phase(&mut phase);
    assert!(phase.data.orders[0].is_invalid());
}

/// 6.A.3. TEST CASE, MOVE FLEET TO LAND
/// Check whether a fleet cannot move to land.
///
/// Germany:
/// F Kiel - Munich
/// Order should fail.
#[test]
fn test_datc_6_a_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit = Unit::new_fleet(Power::Germany, p("kie"));
    let order = unit.move_to(p("mun"));
    phase.data.orders.push(order);
    resolve_orders_for_order_phase(&mut phase);
    assert!(phase.data.orders[0].is_invalid());
}

/// 6.A.4. TEST CASE, MOVE TO OWN SECTOR
/// Moving to the same sector is an illegal move (2023 rulebook, page 7,
/// "An Army can be ordered to move into an adjacent inland or coastal province.").
///
/// Germany:
/// F Kiel - Kiel
/// Program should not crash.
#[test]
fn test_datc_6_a_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit = Unit::new_fleet(Power::Germany, p("kie"));
    let order = unit.move_to(p("kie"));
    phase.data.orders.push(order);
    resolve_orders_for_order_phase(&mut phase);
    assert!(phase.data.orders[0].is_invalid());
}

/// 6.A.5. TEST CASE, MOVE TO OWN SECTOR WITH CONVOY
/// Moving to the same sector is still illegal with convoy (2023 rulebook, page 7,
/// "Note: An Army can move across water provinces from one coastal province to another...").
///
/// England:
/// F North Sea Convoys A Yorkshire - Yorkshire
/// A Yorkshire - Yorkshire
/// A Liverpool Supports A Yorkshire - Yorkshire
///
/// Germany:
/// F London - Yorkshire
/// A Wales Supports F London - Yorkshire
/// The move of the army in Yorkshire is illegal.
/// This makes the support of Liverpool also illegal and without the support,
/// the Germans have a stronger force. The army in London dislodges the army in Yorkshire.
#[test]
fn test_datc_6_a_5() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_army(Power::England, p("yor"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_3 = Unit::new_army(Power::England, p("lvp"));
    let order_e_1 = unit_e_1.move_to(p("yor"));
    let order_e_2 = unit_e_2.convoy(unit_e_1, p("yor"));
    let order_e_3 = unit_e_3.support_move(unit_e_1, p("yor"));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    phase.data.orders.push(order_e_3);
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("lon"));
    let unit_g_2 = Unit::new_army(Power::Germany, p("wal"));
    let order_g_1 = unit_g_1.move_to(p("yor"));
    let order_g_2 = unit_g_2.support_move(unit_g_1, p("yor"));
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].unit, order_e_1.unit);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].unit, order_e_2.unit);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].unit, order_e_3.unit);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[3].unit, order_g_1.unit);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[4].unit, order_g_2.unit);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
}

/// 6.A.6. TEST CASE, ORDERING A UNIT OF ANOTHER COUNTRY
/// Check whether someone cannot order a unit that is not his own unit.
///
/// England has a fleet in London.
///
/// Germany:
/// F London - North Sea
/// Order should fail.
#[test]
fn test_datc_6_a_6() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit = Unit::new_fleet(Power::England, p("lon"));
    let order = unit.move_to(p("nth")).assumed_by(Power::Germany);
    phase.data.orders.push(order);
    resolve_orders_for_order_phase(&mut phase);
    assert!(phase.data.orders[0].is_unresolved());
}

/// 6.A.7. TEST CASE, ONLY ARMIES CAN BE CONVOYED
/// A fleet cannot be convoyed.
///
/// England:
/// F London - Belgium
/// F North Sea Convoys A London - Belgium
/// Move from London to Belgium should fail.
#[test]
fn test_datc_6_a_7() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_fleet(Power::England, p("lon"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("nth"));
    let order_e_1 = unit_e_1.move_to(p("bel"));
    let order_e_2 = unit_e_2.convoy(unit_e_1, p("bel"));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].unit, order_e_1.unit);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].unit, order_e_2.unit);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
}

/// 6.A.8. TEST CASE, SUPPORT TO HOLD YOURSELF IS NOT POSSIBLE
/// An army cannot get an additional hold power by supporting itself.
///
/// Italy:
/// A Venice - Trieste
/// A Tyrolia Supports A Venice - Trieste
///
/// Austria:
/// F Trieste Supports F Trieste
/// The army in Trieste should be dislodged.
#[test]
fn test_datc_6_a_8() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_army(Power::Italy, p("tyr"));
    let order_i_1 = unit_i_1.move_to(p("tri"));
    let order_i_2 = unit_i_2.support_move(unit_i_1, p("tri"));
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    let unit_a_1 = Unit::new_fleet(Power::Austria, p("tri"));
    let order_a_1 = unit_a_1.support_hold(unit_a_1);
    phase.data.orders.push(order_a_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].unit, order_i_1.unit);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].unit, order_i_2.unit);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].unit, order_a_1.unit);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
}

/// 6.A.9. TEST CASE, FLEETS MUST FOLLOW COAST IF NOT ON SEA
/// If two provinces are adjacent, that does not mean that a fleet can move between those two provinces.
/// An implementation that only holds one list of adjacent provinces for each province is incorrect.
///
/// Italy:
/// F Rome - Venice
/// Move fails. An army can go from Rome to Venice, but a fleet cannot.
#[test]
fn test_datc_6_a_9() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("rom"));
    let order_i_1 = unit_i_1.move_to(p("ven"));
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.A.10. TEST CASE, SUPPORT ON UNREACHABLE DESTINATION NOT POSSIBLE
/// The destination of the move that is supported must be reachable by the supporting unit.
///
/// Austria:
/// A Venice Hold
///
/// Italy:
/// F Rome Supports A Apulia - Venice
/// A Apulia - Venice
/// The support of Rome is illegal, because Venice cannot be reached from Rome by a fleet.
/// Venice is not dislodged.
#[test]
fn test_datc_6_a_10() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("ven"));
    let order_a_1 = unit_a_1.hold();
    phase.data.orders.push(order_a_1);
    let unit_i_1 = Unit::new_army(Power::Italy, p("apu"));
    let order_i_1 = unit_i_1.move_to(p("ven"));
    let unit_i_2 = Unit::new_fleet(Power::Italy, p("rom"));
    let order_i_2 = unit_i_2.support_move(unit_i_1, p("ven"));
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].unit, order_a_1.unit);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].unit, order_i_1.unit);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].unit, order_i_2.unit);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
}

/// 6.A.11. TEST CASE, SIMPLE BOUNCE
/// Two armies bouncing on each other.
///
/// Austria:
/// A Vienna - Tyrolia
///
/// Italy:
/// A Venice - Tyrolia
/// The two units bounce.
#[test]
fn test_datc_6_a_11() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("vie"));
    let order_a_1 = unit_a_1.move_to(p("tyr"));
    phase.data.orders.push(order_a_1);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let order_i_1 = unit_i_1.move_to(p("tyr"));
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].unit, order_a_1.unit);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].unit, order_i_1.unit);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
}

/// 6.A.12. TEST CASE, BOUNCE OF THREE UNITS
/// If three units move to the same area,
/// the adjudicator should not bounce the first two units and then let the third unit go to the now open area.
///
/// Austria:
/// A Vienna - Tyrolia
///
/// Germany:
/// A Munich - Tyrolia
///
/// Italy:
/// A Venice - Tyrolia
/// The three units bounce.
#[test]
fn test_datc_6_a_12() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("vie"));
    let order_a_1 = unit_a_1.move_to(p("tyr"));
    phase.data.orders.push(order_a_1);
    let unit_g_1 = Unit::new_army(Power::Italy, p("mun"));
    let order_g_1 = unit_g_1.move_to(p("tyr"));
    phase.data.orders.push(order_g_1);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let order_i_1 = unit_i_1.move_to(p("tyr"));
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].unit, order_a_1.unit);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].unit, order_g_1.unit);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].unit, order_i_1.unit);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.B.1. TEST CASE, MOVING WITH UNSPECIFIED COAST WHEN COAST IS NECESSARY
/// Coast is significant in this case:
///
/// France:
/// F Portugal - Spain
/// Move should fail.
#[test]
fn test_datc_6_b_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_1 = Unit::new_fleet(Power::France, p("por"));
    let order_f_1 = unit_f_1.move_to(p("spa"));
    phase.data.orders.push(order_f_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.B.2. TEST CASE, MOVING WITH UNSPECIFIED COAST WHEN COAST IS NOT NECESSARY
/// There is only one coast possible in this case:
///
/// France:
/// F Gascony - Spain
/// Since the North Coast is the only coast that can be reached,
/// it seems logical that a move is attempted to the north coast of Spain. See issue 4.B.2.
///
/// I prefer that an attempt is made to the only possible coast, the north coast of Spain.
#[allow(unused)]
fn test_datc_6_b_2() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.3. TEST CASE, MOVING WITH WRONG COAST WHEN COAST IS NOT NECESSARY
/// If only one coast is possible, but the wrong coast can be specified.
///
/// France:
/// F Gascony - Spain(sc)
/// If the rules are given a lenient interpretation,
/// a move will be attempted to the north coast of Spain. However, this order is very precisely wrong.
/// The order should be declared illegal and fleet should hold. See issue 4.B.3.
#[test]
fn test_datc_6_b_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_1 = Unit::new_fleet(Power::France, p("gas"));
    let order_f_1 = unit_f_1.move_to(p("spa_sc"));
    phase.data.orders.push(order_f_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.B.4. TEST CASE, SUPPORT TO UNREACHABLE COAST ALLOWED
/// A fleet can give support to a coast where it cannot go.
///
/// France:
/// F Gascony - Spain(nc)
/// F Marseilles Supports F Gascony - Spain(nc)
///
/// Italy:
/// F Western Mediterranean - Spain(sc)
/// Although the fleet in Marseilles cannot go to the north coast it can still support targeting the north coast.
/// So, the support is successful, the move of the fleet in Gascony succeeds and the move of the Italian fleet fails.
#[test]
fn test_datc_6_b_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_1 = Unit::new_fleet(Power::France, p("gas"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("mar"));
    let order_f_1 = unit_f_1.move_to(p("spa_nc"));
    let order_f_2 = unit_f_2.support_move(unit_f_1, p("spa_nc"));
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("wes"));
    let order_i_1 = unit_i_1.move_to(p("spa_sc"));
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.B.5. TEST CASE, SUPPORT FROM UNREACHABLE COAST NOT ALLOWED
/// A fleet cannot give support to an area that cannot be reached from the current coast of the fleet.
///
/// France:
/// F Marseilles - Gulf of Lyon
/// F Spain(nc) Supports F Marseilles - Gulf of Lyon
///
/// Italy:
/// F Gulf of Lyon Hold
/// The Gulf of Lyon cannot be reached from the North Coast of Spain.
/// Therefore, the support of Spain is illegal and the fleet in the Gulf of Lyon is not dislodged.
#[test]
fn test_datc_6_b_5() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_1 = Unit::new_fleet(Power::France, p("mar"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("spa_nc"));
    let order_f_1 = unit_f_1.move_to(p("lyo"));
    let order_f_2 = unit_f_2.support_move(unit_f_1, p("lyo"));
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("lyo"));
    let order_i_1 = unit_i_1.hold();
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
}

/// 6.B.6. TEST CASE, SUPPORT CAN BE CUT WITH OTHER COAST
/// Support can be cut from the other coast.
///
/// England:
/// F Irish Sea Supports F North Atlantic Ocean - Mid-Atlantic Ocean
/// F North Atlantic Ocean - Mid-Atlantic Ocean
///
/// France:
/// F Spain(nc) Supports F Mid-Atlantic Ocean
/// F Mid-Atlantic Ocean Hold
///
/// Italy:
/// F Gulf of Lyon - Spain(sc)
/// The Italian fleet in the Gulf of Lyon will cut the support in Spain.
/// That means that the French fleet in the Mid Atlantic Ocean will be dislodged
/// by the English fleet in the North Atlantic Ocean.
#[test]
fn test_datc_6_b_6() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_fleet(Power::England, p("nao"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("iri"));
    let order_e_1 = unit_e_1.move_to(p("mao"));
    let order_e_2 = unit_e_2.support_move(unit_e_1, p("mao"));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    let unit_f_1 = Unit::new_fleet(Power::France, p("mao"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("spa_nc"));
    let order_f_1 = unit_f_1.hold();
    let order_f_2 = unit_f_2.support_hold(unit_f_1);
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("lyo"));
    let order_i_1 = unit_i_1.move_to(p("spa_sc"));
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.B.7. TEST CASE, SUPPORTING OWN UNIT WITH UNSPECIFIED COAST
/// It is a little bit harsh to reject this.
///
/// France:
/// F Portugal Supports F Mid-Atlantic Ocean - Spain
/// F Mid-Atlantic Ocean - Spain(nc)
///
/// Italy:
/// F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
/// F Western Mediterranean - Spain(sc)
/// See issue 4.B.4.
///
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
/// F Portugal Supports F Gascony - Spain
/// F Gascony - Spain(nc)
///
/// Italy:
/// F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
/// F Western Mediterranean - Spain(sc)
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
/// F Portugal Supports F Mid-Atlantic Ocean - Spain(nc)
/// F Mid-Atlantic Ocean - Spain(sc)
///
/// Italy:
/// F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
/// F Western Mediterranean - Spain(sc)
/// See issue 4.B.4. Support of Portugal is invalid
/// and the Italian fleet in the Western Mediterranean moves successfully.
#[test]
fn test_datc_6_b_9() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_1 = Unit::new_fleet(Power::France, p("mao"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("por"));
    let order_f_1 = unit_f_1.move_to(p("spa_sc"));
    let order_f_2 = unit_f_2.support_move(unit_f_1, p("spa_nc"));
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("wes"));
    let unit_i_2 = Unit::new_fleet(Power::Italy, p("lyo"));
    let order_i_1 = unit_i_1.move_to(p("spa_sc"));
    let order_i_2 = unit_i_2.support_move(unit_i_1, p("spa_sc"));
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
}

/// 6.B.10. TEST CASE, UNIT ORDERED WITH WRONG COAST
/// A player might specify the wrong coast for the ordered unit.
///
/// France has a fleet on the south coast of Spain and orders:
///
/// France:
/// F Spain(nc) - Gulf of Lyon
/// If only perfect orders are accepted, then the move will fail,
/// but since the coast for the ordered unit has no purpose,
/// it might also be ignored (see issue 4.B.5).
///
/// I prefer that a move will be attempted.
#[allow(unused)]
fn test_datc_6_b_10() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.11. TEST CASE, COAST CANNOT BE ORDERED TO CHANGE
/// The coast cannot change by just ordering the other coast.
///
/// France has a fleet on the north coast of Spain and orders:
///
/// France:
/// F Spain(sc) - Gulf of Lyon
/// The move fails.
#[allow(unused)]
fn test_datc_6_b_11() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.B.12. TEST CASE, ARMY MOVEMENT WITH COASTAL SPECIFICATION
/// For armies the coasts are irrelevant:
///
/// France:
/// A Gascony - Spain(nc)
/// If only perfect orders are accepted, then the move will fail.
/// But it is also possible that coasts are ignored in this case
/// and a move will be attempted (see issue 4.B.6).
///
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
/// F Bulgaria(sc) - Constantinople
/// F Constantinople - Bulgaria(ec)
/// Both moves fail.
#[test]
fn test_datc_6_b_13() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("bul_sc"));
    let unit_t_2 = Unit::new_fleet(Power::Turkey, p("con"));
    let order_t_1 = unit_t_1.move_to(p("con"));
    let order_t_2 = unit_t_2.move_to(p("bul_ec"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
}

/// 6.B.14. TEST CASE, BUILDING WITH UNSPECIFIED COAST
/// Coast must be specified in certain build cases:
///
/// Russia:
/// Build F St Petersburg
/// See issue 4.B.7. Build fails.
#[allow(unused)]
fn test_datc_6_b_14() {
    // 建造命令に関するテストは対象外
}

/// 6.B.15. TEST CASE, SUPPORTING FOREIGN UNIT WITH UNSPECIFIED COAST
/// Opinions differ on this.
///
/// France:
/// F Portugal Supports F Mid-Atlantic Ocean - Spain
///
/// England:
/// F Mid-Atlantic Ocean - Spain(nc)
///
/// Italy:
/// F Gulf of Lyon Supports F Western Mediterranean - Spain(sc)
/// F Western Mediterranean - Spain(sc)
/// See issue 4.B.4.
///
/// Although the move to the north coast of Spain might be a surprise for France,
/// it is hard to believe that England somehow tricked France. Therefore,
/// I prefer that the support succeeds and the Italian fleet in the Western Mediterranean bounces.
/// However, if orders are checked on submission (such as in webbased play),
/// support without coast should not be given as an option.
#[allow(unused)]
fn test_datc_6_b_15() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.C.1. TEST CASE, THREE ARMY CIRCULAR MOVEMENT
/// Three units can change place, even in spring 1901.
///
/// Turkey:
/// F Ankara - Constantinople
/// A Constantinople - Smyrna
/// A Smyrna - Ankara
/// All three units will move.
#[test]
fn test_datc_6_c_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_t_1 = Unit::new_army(Power::Turkey, p("ank"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("con"));
    let unit_t_3 = Unit::new_army(Power::Turkey, p("smy"));
    let order_t_1 = unit_t_1.move_to(p("con"));
    let order_t_2 = unit_t_2.move_to(p("smy"));
    let order_t_3 = unit_t_3.move_to(p("ank"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    phase.data.orders.push(order_t_3);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
}

/// 6.C.2. TEST CASE, THREE ARMY CIRCULAR MOVEMENT WITH SUPPORT
/// Three units can change place, even when one gets support.
///
/// Turkey:
/// F Ankara - Constantinople
/// A Constantinople - Smyrna
/// A Smyrna - Ankara
/// A Bulgaria Supports F Ankara - Constantinople
/// Of course, the three units will move, but knowing how programs are written,
/// this can confuse the adjudicator.
#[test]
fn test_datc_6_c_2() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("ank"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("con"));
    let unit_t_3 = Unit::new_army(Power::Turkey, p("smy"));
    let unit_t_4 = Unit::new_army(Power::Turkey, p("bul"));
    let order_t_1 = unit_t_1.move_to(p("con"));
    let order_t_2 = unit_t_2.move_to(p("smy"));
    let order_t_3 = unit_t_3.move_to(p("ank"));
    let order_t_4 = unit_t_4.support_move(unit_t_1, p("con"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    phase.data.orders.push(order_t_3);
    phase.data.orders.push(order_t_4);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
}

/// 6.C.3. TEST CASE, A DISRUPTED THREE ARMY CIRCULAR MOVEMENT
/// When one of the units bounces, the whole circular movement will hold.
///
/// Turkey:
/// F Ankara - Constantinople
/// A Constantinople - Smyrna
/// A Smyrna - Ankara
/// A Bulgaria - Constantinople
/// Every unit will keep its place.
#[test]
fn test_datc_6_c_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("ank"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("con"));
    let unit_t_3 = Unit::new_army(Power::Turkey, p("smy"));
    let unit_t_4 = Unit::new_army(Power::Turkey, p("bul"));
    let order_t_1 = unit_t_1.move_to(p("con"));
    let order_t_2 = unit_t_2.move_to(p("smy"));
    let order_t_3 = unit_t_3.move_to(p("ank"));
    let order_t_4 = unit_t_4.move_to(p("con"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    phase.data.orders.push(order_t_3);
    phase.data.orders.push(order_t_4);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
}

/// 6.C.4. TEST CASE, A CIRCULAR MOVEMENT WITH ATTACKED CONVOY
/// When the circular movement contains an attacked convoy, the circular movement succeeds.
/// The adjudication algorithm should handle attack of convoys before calculating circular movement.
///
/// Austria:
/// A Trieste - Serbia
/// A Serbia - Bulgaria
///
/// Turkey:
/// A Bulgaria - Trieste
/// F Aegean Sea Convoys A Bulgaria - Trieste
/// F Ionian Sea Convoys A Bulgaria - Trieste
/// F Adriatic Sea Convoys A Bulgaria - Trieste
///
/// Italy:
/// F Naples - Ionian Sea
/// The fleet in the Ionian Sea is attacked but not dislodged. The circular movement succeeds.
/// The Austrian and Turkish armies will advance.
#[test]
fn test_datc_6_c_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("ser"));
    let order_a_1 = unit_a_1.move_to(p("ser"));
    let order_a_2 = unit_a_2.move_to(p("bul"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    let unit_t_1 = Unit::new_army(Power::Turkey, p("bul"));
    let unit_t_2 = Unit::new_fleet(Power::Turkey, p("aeg"));
    let unit_t_3 = Unit::new_fleet(Power::Turkey, p("ion"));
    let unit_t_4 = Unit::new_fleet(Power::Turkey, p("adr"));
    let order_t_1 = unit_t_1.move_to(p("tri"));
    let order_t_2 = unit_t_2.convoy(unit_t_1, p("tri"));
    let order_t_3 = unit_t_3.convoy(unit_t_1, p("tri"));
    let order_t_4 = unit_t_4.convoy(unit_t_1, p("tri"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    phase.data.orders.push(order_t_3);
    phase.data.orders.push(order_t_4);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("nap"));
    let order_i_1 = unit_i_1.move_to(p("ion"));
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
}

/// 6.C.5. TEST CASE, A DISRUPTED CIRCULAR MOVEMENT DUE TO DISLODGED CONVOY
/// When the circular movement contains a convoy, the circular movement is disrupted when the convoying fleet is dislodged. The adjudication algorithm should disrupt convoys before calculating circular movement.
///
/// Austria:
/// A Trieste - Serbia
/// A Serbia - Bulgaria
///
/// Turkey:
/// A Bulgaria - Trieste
/// F Aegean Sea Convoys A Bulgaria - Trieste
/// F Ionian Sea Convoys A Bulgaria - Trieste
/// F Adriatic Sea Convoys A Bulgaria - Trieste
///
/// Italy:
/// F Naples - Ionian Sea
/// F Tunis Supports F Naples - Ionian Sea
/// Due to the dislodged convoying fleet, all Austrian and Turkish armies will not move.
#[test]
fn test_datc_6_c_5() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("ser"));
    let order_a_1 = unit_a_1.move_to(p("ser"));
    let order_a_2 = unit_a_2.move_to(p("bul"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    let unit_t_1 = Unit::new_army(Power::Turkey, p("bul"));
    let unit_t_2 = Unit::new_fleet(Power::Turkey, p("aeg"));
    let unit_t_3 = Unit::new_fleet(Power::Turkey, p("ion"));
    let unit_t_4 = Unit::new_fleet(Power::Turkey, p("adr"));
    let order_t_1 = unit_t_1.move_to(p("tri"));
    let order_t_2 = unit_t_2.convoy(unit_t_1, p("tri"));
    let order_t_3 = unit_t_3.convoy(unit_t_1, p("tri"));
    let order_t_4 = unit_t_4.convoy(unit_t_1, p("tri"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    phase.data.orders.push(order_t_3);
    phase.data.orders.push(order_t_4);
    let unit_i_1 = Unit::new_fleet(Power::Italy, p("nap"));
    let unit_i_2 = Unit::new_fleet(Power::Italy, p("tun"));
    let order_i_1 = unit_i_1.move_to(p("ion"));
    let order_i_2 = unit_i_2.support_move(unit_i_1, p("ion"));
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Valid);
}

/// 6.C.6. TEST CASE, TWO ARMIES WITH TWO CONVOYS
/// Two armies can swap places even when they are not adjacent.
///
/// England:
/// F North Sea Convoys A London - Belgium
/// A London - Belgium
///
/// France:
/// F English Channel Convoys A Belgium - London
/// A Belgium - London
/// Both convoys should succeed.
#[test]
fn test_datc_6_c_6() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_army(Power::England, p("lon"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("nth"));
    let order_e_1 = unit_e_1.move_to(p("bel"));
    let order_e_2 = unit_e_2.convoy(unit_e_1, p("bel"));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    let unit_f_1 = Unit::new_army(Power::France, p("bel"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("eng"));
    let order_f_1 = unit_f_1.move_to(p("lon"));
    let order_f_2 = unit_f_2.convoy(unit_f_1, p("lon"));
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
}

/// 6.C.7. TEST CASE, DISRUPTED UNIT SWAP
/// If in a swap one of the unit bounces, then the swap fails.
///
/// England:
/// F North Sea Convoys A London - Belgium
/// A London - Belgium
///
/// France:
/// F English Channel Convoys A Belgium - London
/// A Belgium - London
/// A Burgundy - Belgium
/// None of the units will succeed to move.
#[test]
fn test_datc_6_c_7() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_army(Power::England, p("lon"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("nth"));
    let order_e_1 = unit_e_1.move_to(p("bel"));
    let order_e_2 = unit_e_2.convoy(unit_e_1, p("bel"));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    let unit_f_1 = Unit::new_army(Power::France, p("bel"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_3 = Unit::new_army(Power::France, p("bur"));
    let order_f_1 = unit_f_1.move_to(p("lon"));
    let order_f_2 = unit_f_2.convoy(unit_f_1, p("lon"));
    let order_f_3 = unit_f_3.move_to(p("bel"));
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    phase.data.orders.push(order_f_3);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.C.8. TEST CASE, NO SELF DISLODGEMENT IN DISRUPTED CIRCULAR MOVEMENT
/// Self dislodgement is prohibited as usual in circular movement.
///
/// Turkey:
/// F Constantinople - Black Sea
/// A Bulgaria - Constantinople
/// A Smyrna Supports A Bulgaria - Constantinople
///
/// Russia:
/// F Black Sea - Bulgaria(ec)
///
/// Austria
/// A Serbia - Bulgaria
/// None of the units will succeed to move.
#[test]
fn test_datc_6_c_8() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("con"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("bul"));
    let unit_t_3 = Unit::new_army(Power::Turkey, p("smy"));
    let order_t_1 = unit_t_1.move_to(p("bla"));
    let order_t_2 = unit_t_2.move_to(p("con"));
    let order_t_3 = unit_t_3.support_move(unit_t_2, p("con"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    phase.data.orders.push(order_t_3);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("bla"));
    let order_r_1 = unit_r_1.move_to(p("bul_ec"));
    phase.data.orders.push(order_r_1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("ser"));
    let order_a_1 = unit_a_1.move_to(p("bul"));
    phase.data.orders.push(order_a_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.C.9. TEST CASE, NO HELP IN DISLODGEMENT OF OWN UNIT IN DISRUPTED CIRCULAR MOVEMENT
/// Helping to dislodge your own unit is prohibited as usual in circular movement.
///
/// Turkey:
/// F Constantinople - Black Sea
/// A Smyrna Supports A Bulgaria - Constantinople
///
/// Russia:
/// F Black Sea - Bulgaria(ec)
///
/// Austria
/// A Serbia - Bulgaria
/// A Bulgaria - Constantinople
/// None of the units will succeed to move.
#[test]
fn test_datc_6_c_9() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("ser"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("bul"));
    let order_a_1 = unit_a_1.move_to(p("bul"));
    let order_a_2 = unit_a_2.move_to(p("con"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("con"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("smy"));
    let order_t_1 = unit_t_1.move_to(p("bla"));
    let order_t_2 = unit_t_2.support_move(unit_a_2, p("con"));
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("bla"));
    let order_r_1 = unit_r_1.move_to(p("bul_ec"));
    phase.data.orders.push(order_r_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.D.1. TEST CASE, SUPPORTED HOLD CAN PREVENT DISLODGEMENT
/// The simplest support to hold order.
///
/// Austria:
/// F Adriatic Sea Supports A Trieste - Venice
/// A Trieste - Venice
///
/// Italy:
/// A Venice Hold
/// A Tyrolia Supports A Venice
/// The support of Tyrolia prevents the army in Venice from being dislodged.
/// The army in Trieste will not move.
#[test]
fn test_datc_6_d_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_fleet(Power::Austria, p("adr"));
    let order_a_1 = unit_a_1.move_to(p("ven"));
    let order_a_2 = unit_a_2.support_move(unit_a_1, p("ven"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_army(Power::Italy, p("tyr"));
    let order_i_1 = unit_i_1.hold();
    let order_i_2 = unit_i_2.support_hold(unit_i_1);
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
}

/// 6.D.2. TEST CASE, A MOVE CUTS SUPPORT ON HOLD
/// The simplest support on hold cut.
///
/// Austria:
/// F Adriatic Sea Supports A Trieste - Venice
/// A Trieste - Venice
/// A Vienna - Tyrolia
///
/// Italy:
/// A Venice Hold
/// A Tyrolia Supports A Venice
/// The support of Tyrolia is cut by the army in Vienna.
/// That means that the army in Venice is dislodged by the army from Trieste.
#[test]
fn test_datc_6_d_2() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_fleet(Power::Austria, p("adr"));
    let unit_a_3 = Unit::new_army(Power::Austria, p("vie"));
    let order_a_1 = unit_a_1.move_to(p("ven"));
    let order_a_2 = unit_a_2.support_move(unit_a_1, p("ven"));
    let order_a_3 = unit_a_3.move_to(p("tyr"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    phase.data.orders.push(order_a_3);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_army(Power::Italy, p("tyr"));
    let order_i_1 = unit_i_1.hold();
    let order_i_2 = unit_i_2.support_hold(unit_i_1);
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Cut);
}

/// 6.D.3. TEST CASE, A MOVE CUTS SUPPORT ON MOVE
/// The simplest support on move cut.
///
/// Austria:
/// F Adriatic Sea Supports A Trieste - Venice
/// A Trieste - Venice
///
/// Italy:
/// A Venice Hold
/// F Ionian Sea - Adriatic Sea
/// The support of the fleet in the Adriatic Sea is cut.
/// That means that the army in Venice will not be dislodged and the army in Trieste stays in Trieste.
#[test]
fn test_datc_6_d_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_fleet(Power::Austria, p("adr"));
    let order_a_1 = unit_a_1.move_to(p("ven"));
    let order_a_2 = unit_a_2.support_move(unit_a_1, p("ven"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_fleet(Power::Italy, p("ion"));
    let order_i_1 = unit_i_1.hold();
    let order_i_2 = unit_i_2.move_to(p("adr"));
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
}

/// 6.D.4. TEST CASE, SUPPORT TO HOLD ON UNIT SUPPORTING A HOLD ALLOWED
/// A unit that is supporting a hold, can receive a hold support.
///
/// Germany:
/// A Berlin Supports F Kiel
/// F Kiel Supports A Berlin
///
/// Russia:
/// F Baltic Sea Supports A Prussia - Berlin
/// A Prussia - Berlin
/// The Russian move from Prussia to Berlin fails.
#[test]
fn test_datc_6_d_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Germany, p("ber"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("kie"));
    let order_g_1 = unit_g_1.support_hold(unit_g_2);
    let order_g_2 = unit_g_2.support_hold(unit_g_1);
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("bal"));
    let unit_r_2 = Unit::new_army(Power::Russia, p("pru"));
    let order_r_1 = unit_r_1.support_move(unit_r_2, p("ber"));
    let order_r_2 = unit_r_2.move_to(p("ber"));
    phase.data.orders.push(order_r_1);
    phase.data.orders.push(order_r_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
}

/// 6.D.5. TEST CASE, SUPPORT TO HOLD ON UNIT SUPPORTING A MOVE ALLOWED
/// A unit that is supporting a move, can receive a hold support.
///
/// Germany:
/// A Berlin Supports A Munich - Silesia
/// F Kiel Supports A Berlin
/// A Munich - Silesia
///
/// Russia:
/// F Baltic Sea Supports A Prussia - Berlin
/// A Prussia - Berlin
/// The Russian move from Prussia to Berlin fails.
#[test]
fn test_datc_6_d_5() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Germany, p("ber"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("kie"));
    let unit_g_3 = Unit::new_army(Power::Germany, p("mun"));
    let order_g_1 = unit_g_1.support_move(unit_g_3, p("sil"));
    let order_g_2 = unit_g_2.support_hold(unit_g_1);
    let order_g_3 = unit_g_3.move_to(p("sil"));
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    phase.data.orders.push(order_g_3);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("bal"));
    let unit_r_2 = Unit::new_army(Power::Russia, p("pru"));
    let order_r_1 = unit_r_1.support_move(unit_r_2, p("ber"));
    let order_r_2 = unit_r_2.move_to(p("ber"));
    phase.data.orders.push(order_r_1);
    phase.data.orders.push(order_r_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.D.6. TEST CASE, SUPPORT TO HOLD ON CONVOYING UNIT ALLOWED
/// A unit that is convoying, can receive a hold support.
///
/// Germany:
/// A Berlin - Sweden
/// F Baltic Sea Convoys A Berlin - Sweden
/// F Prussia Supports F Baltic Sea
///
/// Russia:
/// F Livonia - Baltic Sea
/// F Gulf of Bothnia Supports F Livonia - Baltic Sea
/// The Russian move from Livonia to the Baltic Sea fails. The convoy from Berlin to Sweden succeeds.
#[test]
fn test_datc_6_d_6() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Germany, p("ber"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("bal"));
    let unit_g_3 = Unit::new_fleet(Power::Germany, p("pru"));
    let order_g_1 = unit_g_1.move_to(p("swe"));
    let order_g_2 = unit_g_2.convoy(unit_g_1, p("swe"));
    let order_g_3 = unit_g_3.support_hold(unit_g_2);
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    phase.data.orders.push(order_g_3);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("lvn"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("bot"));
    let order_r_1 = unit_r_1.move_to(p("bal"));
    let order_r_2 = unit_r_2.support_move(unit_r_1, p("bal"));
    phase.data.orders.push(order_r_1);
    phase.data.orders.push(order_r_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
}

/// 6.D.7. TEST CASE, SUPPORT TO HOLD ON MOVING UNIT NOT ALLOWED
/// A unit that is moving, cannot receive a hold support for the situation that the move fails.
///
/// Germany:
/// F Baltic Sea - Sweden
/// F Prussia Supports F Baltic Sea
///
/// Russia:
/// F Livonia - Baltic Sea
/// F Gulf of Bothnia Supports F Livonia - Baltic Sea
/// A Finland - Sweden
/// The support of the fleet in Prussia fails.
/// The fleet in Baltic Sea will bounce on the Russian army in Finland
/// and will be dislodged by the Russian fleet from Livonia when it returns to the Baltic Sea.
#[test]
fn test_datc_6_d_7() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("bal"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("pru"));
    let order_g_1 = unit_g_1.move_to(p("swe"));
    let order_g_2 = unit_g_2.support_hold(unit_g_1);
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("lvn"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("bot"));
    let unit_r_3 = Unit::new_army(Power::Russia, p("fin"));
    let order_r_1 = unit_r_1.move_to(p("bal"));
    let order_r_2 = unit_r_2.support_move(unit_r_1, p("bal"));
    let order_r_3 = unit_r_3.move_to(p("swe"));
    phase.data.orders.push(order_r_1);
    phase.data.orders.push(order_r_2);
    phase.data.orders.push(order_r_3);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.D.8. TEST CASE, FAILED CONVOY CANNOT RECEIVE HOLD SUPPORT
/// If a convoy fails because of disruption of the convoy or when the right convoy orders are not given,
/// then the army to be convoyed cannot receive support in hold, since it still tried to move.
///
/// Austria:
/// F Ionian Sea Hold
/// A Serbia Supports A Albania - Greece
/// A Albania - Greece
///
/// Turkey:
/// A Greece - Naples
/// A Bulgaria Supports A Greece
/// There was a possible convoy from Greece to Naples, before the orders were made public (via the Ionian Sea).
/// This means that the order of Greece to Naples should never be treated as illegal order
/// and be changed in a hold order able to receive hold support (see also issue 4.E.1).
/// Therefore, the support in Bulgaria fails and the army in Greece is dislodged by the army in Albania.
#[test]
fn test_datc_6_d_8() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_fleet(Power::Austria, p("ion"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("ser"));
    let unit_a_3 = Unit::new_army(Power::Austria, p("alb"));
    let order_a_1 = unit_a_1.hold();
    let order_a_2 = unit_a_2.support_move(unit_a_3, p("gre"));
    let order_a_3 = unit_a_3.move_to(p("gre"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    phase.data.orders.push(order_a_3);
    let unit_t_1 = Unit::new_army(Power::Turkey, p("gre"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("bul"));
    let order_t_1 = unit_t_1.move_to(p("nap"));
    let order_t_2 = unit_t_2.support_hold(unit_t_1);
    phase.data.orders.push(order_t_1);
    phase.data.orders.push(order_t_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Invalid);
}

/// 6.D.9. TEST CASE, SUPPORT TO MOVE ON HOLDING UNIT NOT ALLOWED
/// A unit that is holding cannot receive a support in moving.
///
/// Italy:
/// A Venice - Trieste
/// A Tyrolia Supports A Venice - Trieste
///
/// Austria:
/// A Albania Supports A Trieste - Serbia
/// A Trieste Hold
/// The support of the army in Albania fails and the army in Trieste is dislodged by the army from Venice.
#[test]
fn test_datc_6_d_9() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_army(Power::Italy, p("tyr"));
    let order_i_1 = unit_i_1.move_to(p("tri"));
    let order_i_2 = unit_i_2.support_move(unit_i_1, p("tri"));
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    let unit_a_1 = Unit::new_army(Power::Austria, p("alb"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("tri"));
    let order_a_1 = unit_a_1.support_move(unit_a_2, p("ser"));
    let order_a_2 = unit_a_2.hold();
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
}

/// 6.D.10. TEST CASE, SELF DISLODGMENT PROHIBITED
/// A unit may not dislodge a unit of the same great power.
///
/// Germany:
/// A Berlin Hold
/// F Kiel - Berlin
/// A Munich Supports F Kiel - Berlin
/// Move to Berlin fails.
#[test]
fn test_datc_6_d_10() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Italy, p("ber"));
    let unit_g_2 = Unit::new_army(Power::Italy, p("kie"));
    let unit_g_3 = Unit::new_army(Power::Italy, p("mun"));
    let order_g_1 = unit_g_1.hold();
    let order_g_2 = unit_g_2.move_to(p("ber"));
    let order_g_3 = unit_g_3.support_move(unit_g_2, p("ber"));
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    phase.data.orders.push(order_g_3);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
}

/// 6.D.11. TEST CASE, NO SELF DISLODGMENT OF RETURNING UNIT
/// Idem.
///
/// Germany:
/// A Berlin - Prussia
/// F Kiel - Berlin
/// A Munich Supports F Kiel - Berlin
///
/// Russia:
/// A Warsaw - Prussia
/// Army in Berlin bounces, but is not dislodged by own unit.
#[test]
fn test_datc_6_d_11() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Italy, p("ber"));
    let unit_g_2 = Unit::new_fleet(Power::Italy, p("kie"));
    let unit_g_3 = Unit::new_army(Power::Italy, p("mun"));
    let order_g_1 = unit_g_1.move_to(p("pru"));
    let order_g_2 = unit_g_2.move_to(p("ber"));
    let order_g_3 = unit_g_3.support_move(unit_g_2, p("ber"));
    phase.data.orders.push(order_g_1);
    phase.data.orders.push(order_g_2);
    phase.data.orders.push(order_g_3);
    let unit_r_1 = Unit::new_army(Power::Austria, p("war"));
    let order_r_1 = unit_r_1.move_to(p("pru"));
    phase.data.orders.push(order_r_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
}

/// 6.D.12. TEST CASE, SUPPORTING A FOREIGN UNIT TO DISLODGE OWN UNIT PROHIBITED
/// You may not help another power in dislodging your own unit.
///
/// Austria:
/// F Trieste Hold
/// A Vienna Supports A Venice - Trieste
///
/// Italy:
/// A Venice - Trieste
/// No dislodgment of fleet in Trieste.
#[test]
fn test_datc_6_d_12() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_fleet(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("vie"));
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let order_a_1 = unit_a_1.hold();
    let order_a_2 = unit_a_2.support_move(unit_i_1, p("tri"));
    let order_i_1 = unit_i_1.move_to(p("tri"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    phase.data.orders.push(order_i_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.D.13. TEST CASE, SUPPORTING A FOREIGN UNIT TO DISLODGE A RETURNING OWN UNIT PROHIBITED
/// Idem.
///
/// Austria:
/// F Trieste - Adriatic Sea
/// A Vienna Supports A Venice - Trieste
///
/// Italy:
/// A Venice - Trieste
/// F Apulia - Adriatic Sea
/// No dislodgment of fleet in Trieste.
#[test]
fn test_datc_6_d_13() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_fleet(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("vie"));
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_fleet(Power::Italy, p("apu"));
    let order_a_1 = unit_a_1.move_to(p("adr"));
    let order_a_2 = unit_a_2.support_move(unit_i_1, p("tri"));
    let order_i_1 = unit_i_1.move_to(p("tri"));
    let order_i_2 = unit_i_2.move_to(p("adr"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
}

/// 6.D.14. TEST CASE, SUPPORTING A FOREIGN UNIT IS NOT ENOUGH TO PREVENT DISLODGEMENT
/// If a foreign unit has enough support to dislodge your unit,
/// you may not prevent that dislodgement by supporting the attack.
///
/// Austria:
/// F Trieste Hold
/// A Vienna Supports A Venice - Trieste
///
/// Italy:
/// A Venice - Trieste
/// A Tyrolia Supports A Venice - Trieste
/// F Adriatic Sea Supports A Venice - Trieste
/// The fleet in Trieste is dislodged.
#[test]
fn test_datc_6_d_14() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_a_1 = Unit::new_fleet(Power::Austria, p("tri"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("vie"));
    let unit_i_1 = Unit::new_army(Power::Italy, p("ven"));
    let unit_i_2 = Unit::new_army(Power::Italy, p("tyr"));
    let unit_i_3 = Unit::new_fleet(Power::Italy, p("adr"));
    let order_a_1 = unit_a_1.hold();
    let order_a_2 = unit_a_2.support_move(unit_i_1, p("tri"));
    let order_i_1 = unit_i_1.move_to(p("tri"));
    let order_i_2 = unit_i_2.support_move(unit_i_1, p("tri"));
    let order_i_3 = unit_i_3.support_move(unit_i_1, p("tri"));
    phase.data.orders.push(order_a_1);
    phase.data.orders.push(order_a_2);
    phase.data.orders.push(order_i_1);
    phase.data.orders.push(order_i_2);
    phase.data.orders.push(order_i_3);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
}

/// 6.D.15. TEST CASE, DEFENDER CANNOT CUT SUPPORT FOR ATTACK ON ITSELF
/// A unit that is attacked by a supported unit cannot prevent dislodgement
/// by guessing which of the units will do the support.
///
/// Russia:
/// F Constantinople Supports F Black Sea - Ankara
/// F Black Sea - Ankara
///
/// Turkey:
/// F Ankara - Constantinople
/// The support of Constantinople is not cut
/// and the fleet in Ankara is dislodged by the fleet in the Black Sea.
#[test]
fn test_datc_6_d_15() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("con"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("bla"));
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("ank"));
    let order_r_1 = unit_r_1.support_move(unit_r_2, p("ank"));
    let order_r_2 = unit_r_2.move_to(p("ank"));
    let order_t_1 = unit_t_1.move_to(p("con"));
    phase.data.orders.push(order_r_1);
    phase.data.orders.push(order_r_2);
    phase.data.orders.push(order_t_1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
}

/// 6.D.16. TEST CASE, CONVOYING A UNIT DISLODGING A UNIT OF SAME POWER IS ALLOWED
/// It is allowed to convoy a foreign unit that dislodges your own unit is allowed.
///
/// England:
/// A London Hold
/// F North Sea Convoys A Belgium - London
///
/// France:
/// F English Channel Supports A Belgium - London
/// A Belgium - London
/// The English army in London is dislodged by the French army coming from Belgium.
#[test]
fn test_datc_6_d_16() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_army(Power::England, p("lon"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("nth"));
    let unit_f_1 = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_2 = Unit::new_army(Power::France, p("bel"));
    let order_e_1 = unit_e_1.hold();
    let order_e_2 = unit_e_2.convoy(unit_f_2, p("lon"));
    let order_f_1 = unit_f_1.support_move(unit_f_2, p("lon"));
    let order_f_2 = unit_f_2.move_to(p("lon"));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    phase.data.orders.push(order_f_1);
    phase.data.orders.push(order_f_2);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
}

/// 6.D.17. TEST CASE, DISLODGEMENT CUTS SUPPORTS
/// The famous dislodge rule.
///
/// Russia:
/// F Constantinople Supports F Black Sea - Ankara
/// F Black Sea - Ankara
///
/// Turkey:
/// F Ankara - Constantinople
/// A Smyrna Supports F Ankara - Constantinople
/// A Armenia - Ankara
/// The Russian fleet in Constantinople is dislodged.
/// This cuts the support to from Black Sea to Ankara.
/// Black Sea will bounce with the army from Armenia.
#[test]
fn test_datc_6_d_17() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("con"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("bla"));
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("ank"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("smy"));
    let unit_t_3 = Unit::new_army(Power::Turkey, p("arm"));
    phase.data.orders.push(unit_r_1.support_move(unit_r_2, p("ank")));
    phase.data.orders.push(unit_r_2.move_to(p("ank")));
    phase.data.orders.push(unit_t_1.move_to(p("con")));
    phase.data.orders.push(unit_t_2.support_move(unit_t_1, p("con")));
    phase.data.orders.push(unit_t_3.move_to(p("ank")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.D.18. TEST CASE, A SURVIVING UNIT WILL SUSTAIN SUPPORT
/// Idem. But now with an additional hold that prevents dislodgement.
///
/// Russia:
/// F Constantinople Supports F Black Sea - Ankara
/// F Black Sea - Ankara
/// A Bulgaria Supports F Constantinople
///
/// Turkey:
/// F Ankara - Constantinople
/// A Smyrna Supports F Ankara - Constantinople
/// A Armenia - Ankara
/// The Russian fleet in the Black Sea will dislodge the Turkish fleet in Ankara.
#[test]
fn test_datc_6_d_18() {}

/// 6.D.19. TEST CASE, EVEN WHEN SURVIVING IS IN ALTERNATIVE WAY
/// Now, the dislodgement is prevented because the support comes from a Russian army:
///
/// Russia:
/// F Constantinople Supports F Black Sea - Ankara
/// F Black Sea - Ankara
/// A Smyrna Supports F Ankara - Constantinople
///
/// Turkey:
/// F Ankara - Constantinople
/// The Russian fleet in Constantinople is not dislodged,
/// because one of the supports is of Russian origin.
/// The support from Black Sea to Ankara will sustain and the fleet in Ankara will be dislodged.
#[test]
fn test_datc_6_d_19() {}

/// 6.D.20. TEST CASE, UNIT CANNOT CUT SUPPORT OF ITS OWN COUNTRY
/// Although this is not mentioned in all rulebooks,
/// it is generally accepted that when a unit attacks another unit of the same Great Power,
/// it will not cut support.
///
/// England:
/// F London Supports F North Sea - English Channel
/// F North Sea - English Channel
/// A Yorkshire - London
///
/// France:
/// F English Channel Hold
/// The army in York does not cut support.
/// This means that the fleet in the English Channel is dislodged by the fleet in the North Sea.
#[test]
fn test_datc_6_d_20() {}

/// 6.D.21. TEST CASE, DISLODGING DOES NOT CANCEL A SUPPORT CUT
/// Sometimes there is the question whether a dislodged moving unit does not cut support
/// (similar to the dislodge rule).
/// This is not the case.
///
/// Austria:
/// F Trieste Hold
///
/// Italy:
/// A Venice - Trieste
/// A Tyrolia Supports A Venice - Trieste
///
/// Germany:
/// A Munich - Tyrolia
///
/// Russia:
/// A Silesia - Munich
/// A Berlin Supports A Silesia - Munich
/// Although the German army is dislodged, it still cuts the Italian support.
/// That means that the Austrian Fleet is not dislodged.
#[test]
fn test_datc_6_d_21() {}

/// 6.D.22. TEST CASE, IMPOSSIBLE FLEET MOVE CANNOT BE SUPPORTED
/// If a fleet tries moves to a land area it seems pointless to support the fleet,
/// since the move will fail anyway. However, in such case,
/// the support is also invalid for defense purposes.
///
/// Germany:
/// F Kiel - Munich
/// A Burgundy Supports F Kiel - Munich
///
/// Russia:
/// A Munich - Kiel
/// A Berlin Supports A Munich - Kiel
/// The German move from Kiel to Munich is illegal (fleets cannot go to Munich).
/// Illegal orders are fully ignored which makes the support from Burgundy also illegal.
/// The Russian army in Munich will dislodge the fleet in Kiel.
#[test]
fn test_datc_6_d_22() {}

/// 6.D.23. TEST CASE, IMPOSSIBLE COAST MOVE CANNOT BE SUPPORTED
/// Comparable with the previous test case, but now the fleet move is impossible for coastal reasons.
///
/// Italy:
/// F Gulf of Lyon - Spain(sc)
/// F Western Mediterranean Supports F Gulf of Lyon - Spain(sc)
///
/// France:
/// F Spain(nc) - Gulf of Lyon
/// F Marseilles Supports F Spain(nc) - Gulf of Lyon
/// The French move from Spain North Coast to Gulf of Lyon is illegal (wrong coast).
/// Therefore, the support from Marseilles fails and the fleet in Spain is dislodged.
#[test]
fn test_datc_6_d_23() {}

/// 6.D.24. TEST CASE, IMPOSSIBLE ARMY MOVE CANNOT BE SUPPORTED
/// Comparable with the previous test case,
/// but now an army tries to move into sea and the support is used in a beleaguered garrison.
///
/// France:
/// A Marseilles - Gulf of Lyon
/// F Spain(sc) Supports A Marseilles - Gulf of Lyon
///
/// Italy:
/// F Gulf of Lyon Hold
///
/// Turkey:
/// F Tyrrhenian Sea Supports F Western Mediterranean - Gulf of Lyon
/// F Western Mediterranean - Gulf of Lyon
/// The French move from Marseilles to Gulf of Lyon is illegal (an army cannot go to sea).
/// Therefore,/// the support from Spain fails and there is no beleaguered garrison.
/// The fleet in the Gulf of Lyon is dislodged by the Turkish fleet in the Western Mediterranean.
#[test]
fn test_datc_6_d_24() {}

/// 6.D.25. TEST CASE, FAILING HOLD SUPPORT CAN BE SUPPORTED
/// If an adjudicator fails on one of the previous three test cases,
/// then the bug should be removed with care.
/// A failing move cannot be supported, but a failing hold support,
/// because of some preconditions (unmatching order) can still be supported.
///
/// Germany:
/// A Berlin Supports A Prussia
/// F Kiel Supports A Berlin
///
/// Russia:
/// F Baltic Sea Supports A Prussia - Berlin
/// A Prussia - Berlin
/// Although the support of Berlin on Prussia fails (because of unmatching orders),
/// the support of Kiel on Berlin is still valid. So, Berlin will not be dislodged.
#[test]
fn test_datc_6_d_25() {}

/// 6.D.26. TEST CASE, FAILING MOVE SUPPORT CAN BE SUPPORTED
/// Similar as the previous test case, but now with an unmatched support to move.
///
/// Germany:
/// A Berlin Supports A Prussia - Silesia
/// F Kiel Supports A Berlin
///
/// Russia:
/// F Baltic Sea Supports A Prussia - Berlin
/// A Prussia - Berlin
/// Again, Berlin will not be dislodged.
#[test]
fn test_datc_6_d_26() {}

/// 6.D.27. TEST CASE, FAILING CONVOY CAN BE SUPPORTED
/// Similar as the previous test case, but now with an unmatched convoy.
///
/// England:
/// F Sweden - Baltic Sea
/// F Denmark Supports F Sweden - Baltic Sea
///
/// Germany:
/// A Berlin Hold
///
/// Russia:
/// F Baltic Sea Convoys A Berlin - Livonia
/// F Prussia Supports F Baltic Sea
/// The convoy order in the Baltic Sea is unmatched and fails. However,
/// the support of Prussia on the Baltic Sea is still valid
/// and the fleet in the Baltic Sea is not dislodged.
#[test]
fn test_datc_6_d_27() {}

/// 6.D.28. TEST CASE, IMPOSSIBLE MOVE AND SUPPORT
/// An impossible move is "illegal" and should be ignored.
///
/// Austria:
/// A Budapest Supports F Rumania
///
/// Russia:
/// F Rumania - Holland
///
/// Turkey:
/// F Black Sea - Rumania
/// A Bulgaria Supports F Black Sea - Rumania
/// See issue 4.E.1. Illegal orders are ignored. Without an order,
/// Rumania holds and receives support.
/// The fleet in Rumania is not dislodged.
#[test]
fn test_datc_6_d_28() {}

/// 6.D.29. TEST CASE, MOVE TO IMPOSSIBLE COAST AND SUPPORT
/// Similar to the previous test case, but now the move "illegal" due the wrong coast.
///
/// Austria:
/// A Budapest Supports F Rumania
///
/// Russia:
/// F Rumania - Bulgaria(sc)
///
/// Turkey:
/// F Black Sea - Rumania
/// A Bulgaria Supports F Black Sea - Rumania
/// See issue 4.E.1. Illegal orders are ignored. Without an order,
/// Rumania holds and receives support.
/// The fleet in Rumania is not dislodged.
#[test]
fn test_datc_6_d_29() {}

/// 6.D.30. TEST CASE, MOVE WITHOUT COAST AND SUPPORT
/// Similar to the previous test case, but now the move is "illegal" due to missing coast.
///
/// Italy:
/// F Aegean Sea Supports F Constantinople
///
/// Russia:
/// F Constantinople - Bulgaria
///
/// Turkey:
/// F Black Sea - Constantinople
/// A Bulgaria Supports F Black Sea - Constantinople
/// See issue 4.E.1. Illegal orders are ignored. Without an order,
/// Constantinople holds and receives support.
/// The fleet in Constantinople is not dislodged.
#[test]
fn test_datc_6_d_30() {}

/// 6.D.31. TEST CASE, A TRICKY IMPOSSIBLE SUPPORT
/// A support order can be impossible for complex reasons.
///
/// Austria:
/// A Rumania - Armenia
///
/// Turkey:
/// F Black Sea Supports A Rumania - Armenia
/// Although the army in Rumania can move to Armenia
/// and the fleet in the Black Sea can also go to Armenia, the support is still not possible.
/// The reason is that the only possible convoy is through the Black Sea
/// and a fleet cannot convoy and support at the same time.
///
/// This is relevant for computer programs that show only the possible orders.
/// In the list of possible orders,
/// the support as given to the fleet in the Black Sea, should not be listed.
///
/// Furthermore, the support order should be judged to be illegal,
/// meaning that it is completely ignored.
/// If there is a second order for the Black Sea, that order should be executed (see issue 4.E.1).
#[test]
fn test_datc_6_d_31() {}

/// 6.D.32. TEST CASE, A MISSING FLEET
/// The previous test cases contained an order that was impossible
/// even when some other pieces on the board where changed.
/// In this test case, the order is impossible, but only for that situation.
///
/// England:
/// F Edinburgh Supports A Liverpool - Yorkshire
/// A Liverpool - Yorkshire
///
/// France:
/// F London Supports A Yorkshire
///
/// Germany:
/// A Yorkshire - Holland
/// The German order to Yorkshire cannot be executed,
/// because there is no fleet in the North Sea.
/// In other situations (where there is a fleet in the North Sea),
/// the exact same order would be possible.
/// This is considered "illegal" (see issue 4.E.1).
/// The order should be ignored and the support of the French fleet in London succeeds.
/// This means that the army in Yorkshire is not dislodged.
#[test]
fn test_datc_6_d_32() {}

/// 6.D.33. TEST CASE, UNWANTED SUPPORT ALLOWED
/// A self standoff can be broken by an unwanted support.
///
/// Austria:
/// A Serbia - Budapest
/// A Vienna - Budapest
///
/// Russia:
/// A Galicia Supports A Serbia - Budapest
///
/// Turkey:
/// A Bulgaria - Serbia
/// Due to the Russian support, the army in Serbia advances to Budapest.
/// This enables Turkey to capture Serbia with the army in Bulgaria.
#[test]
fn test_datc_6_d_33() {}

/// 6.D.34. TEST CASE, SUPPORT TARGETING OWN AREA NOT ALLOWED
/// Support targeting the area where the supporting unit is standing, is illegal.
///
/// Germany:
/// A Berlin - Prussia
/// A Silesia Supports A Berlin - Prussia
/// F Baltic Sea Supports A Berlin - Prussia
///
/// Italy:
/// A Prussia Supports Livonia - Prussia
///
/// Russia:
/// A Warsaw Supports A Livonia - Prussia
/// A Livonia - Prussia
/// Russia and Italy wanted to get rid of the Italian army in Prussia
/// (to build an Italian fleet somewhere else).
/// However, they didn't want a possible German attack on Prussia to succeed.
/// They invented this odd order of Italy.
/// It was intended that the attack of the army in Livonia would have strength three,
/// so it would be capable to prevent the possible German attack to succeed. However,
/// the order of Italy is illegal,
/// because a unit may only support to an area where the unit can go by itself.
/// A unit can't go to the area it is already standing,
/// so the Italian order is illegal and the German move from Berlin succeeds.
/// Even if it would be legal, the German move from Berlin would still succeed,
/// because the support of Prussia is cut by Livonia and Berlin.
#[test]
fn test_datc_6_d_34() {}
