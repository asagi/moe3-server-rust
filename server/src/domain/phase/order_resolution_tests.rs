//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6]
//!
//! * 6.A. TEST CASES, BASIC CHECKS
//! * 6.B. TEST CASES, COASTAL ISSUES
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
    // 不十分命令の類推に関するテスト（対応予定なし）
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
    // 不十分命令の類推に関するテスト（対応予定なし）
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
    // 不十分命令の類推に関するテスト（対応予定なし）
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
    // 命令対象の誤りに関するテスト（対応予定なし）
}

/// 6.B.11. TEST CASE, COAST CANNOT BE ORDERED TO CHANGE
/// The coast cannot change by just ordering the other coast.
///
/// France has a fleet on the north coast of Spain and orders:
///
/// France:
/// F Spain(sc) - Gulf of Lyon
/// The move fails.
#[test]
fn test_datc_6_b_11() {}

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
#[test]
fn test_datc_6_b_12() {}

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
fn test_datc_6_b_13() {}

/// 6.B.14. TEST CASE, BUILDING WITH UNSPECIFIED COAST
/// Coast must be specified in certain build cases:
///
/// Russia:
/// Build F St Petersburg
/// See issue 4.B.7. Build fails.
#[test]
fn test_datc_6_b_14() {}

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
#[test]
fn test_datc_6_b_15() {}
