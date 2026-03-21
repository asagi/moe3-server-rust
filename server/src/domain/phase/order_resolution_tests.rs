//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6]
//!
//! ## 6.A. TEST CASES, BASIC CHECKS
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
    let order_e_3 = unit_e_3.support(unit_e_1, Some(p("yor")));
    phase.data.orders.push(order_e_1);
    phase.data.orders.push(order_e_2);
    phase.data.orders.push(order_e_3);

    let unit_g_1 = Unit::new_fleet(Power::Germany, p("lon"));
    let unit_g_2 = Unit::new_army(Power::Germany, p("wal"));
    let order_g_1 = unit_g_1.move_to(p("yor"));
    let order_g_2 = unit_g_2.support(unit_g_1, Some(p("yor")));
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
fn test_datc_6_a_8() {}

/// 6.A.9. TEST CASE, FLEETS MUST FOLLOW COAST IF NOT ON SEA
/// If two provinces are adjacent, that does not mean that a fleet can move between those two provinces.
/// An implementation that only holds one list of adjacent provinces for each province is incorrect.
///
/// Italy:
/// F Rome - Venice
/// Move fails. An army can go from Rome to Venice, but a fleet cannot.
#[test]
fn test_datc_6_a_9() {}

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
fn test_datc_6_a_10() {}

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
fn test_datc_6_a_11() {}

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
fn test_datc_6_a_12() {}
