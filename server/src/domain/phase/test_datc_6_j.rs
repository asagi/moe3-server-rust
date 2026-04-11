//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6J]
//!
//! * 6.J. TEST CASES, CIVIL DISORDER AND DISBANDS
//!
//! [DATC_6J]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.J

use super::super::order::*;
use super::super::phase::adjustment_order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.J.1. TEST CASE, TOO MANY DISBAND ORDERS
/// Check how program reacts when someone orders too many disbands.
/// France has to disband one and has an army in Paris and Picardy.
///
/// France:
///     Remove F Gulf of Lyon
///     Remove A Picardy
///     Remove A Paris
///
/// Program should not disband both Paris and Picardy, but should handle it in a different way.
/// See also issue 4.D.6.
/// I prefer that the disband orders are handled one by one.
/// According to the preference, the removal of the fleet in the Gulf of Lyon fails (no fleet),
/// the removal of the army in Picardy succeeds and the removal of the army in Paris fails (too many disbands).
#[test]
fn test_datc_6_j_1() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::France, "par"));
    phase.data.units.push(Unit::new_army(Power::France, p("par")));
    phase.data.units.push(Unit::new_army(Power::France, p("pic")));
    let context = &mut PhaseContext::new();
    let unit_f_lyo = Unit::new_fleet(Power::France, p("lyo"));
    let unit_a_pic = Unit::new_army(Power::France, p("pic"));
    let unit_a_par = Unit::new_army(Power::France, p("par"));
    phase.data.orders.push(unit_f_lyo.disband());
    phase.data.orders.push(unit_a_pic.disband());
    phase.data.orders.push(unit_a_par.disband());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
}

/// 6.J.2. TEST CASE, REMOVING THE SAME UNIT TWICE
/// If you have to remove two units,
/// you can always try to trick the computer by removing the same unit twice.
///
/// France has to disband two and has an army in Paris.
///
/// France:
///     Remove A Paris
///     Remove A Paris
///
/// Program should remove army in Paris and remove another unit by using the civil disorder rules.
#[test]
fn test_datc_6_j_2() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.units.push(Unit::new_army(Power::France, p("par")));
    phase.data.units.push(Unit::new_army(Power::France, p("bre")));
    let context = &mut PhaseContext::new();
    let unit_a_par = Unit::new_army(Power::France, p("par"));
    phase.data.orders.push(unit_a_par.disband());
    phase.data.orders.push(unit_a_par.disband());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
}

/// 6.J.3. TEST CASE, CIVIL DISORDER TWO ARMIES WITH DIFFERENT DISTANCE
/// When a player forgets to disband a unit, the civil disorder rules must be applied.
/// When two armies have different distance from the home supply centers,
/// then the army with the greatest distance has to be removed.
///
/// Russia has to remove one.
/// Russia owns supply center St Petersburg.
/// Russia has armies in Livonia and Sweden.
/// Russia does not order a disband.
///
/// The army in Sweden is removed.
#[test]
fn test_datc_6_j_3() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Russia, "stp"));
    phase.data.units.push(Unit::new_army(Power::Russia, p("lvp")));
    phase.data.units.push(Unit::new_army(Power::Russia, p("swe")));
    let context = &mut PhaseContext::new();
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders.len(), 1);
    assert_eq!(phase.data.orders[0].to_string(), "Remove A swe");
}

/// 6.J.4. TEST CASE, CIVIL DISORDER TWO ARMIES WITH EQUAL DISTANCE
/// If two armies have equal distance from the home supply centers,
/// then alphabetical order is used.
///
/// Russia has to remove one.
/// Russia owns Moscow.
/// Russia has armies in Livonia and Ukraine.
/// Russia does not order a disband.
///
/// Both armies have distance one. The Livonia army is removed,
/// because it appears first in alphabetical order.
#[test]
fn test_datc_6_j_4() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Russia, "mos"));
    phase.data.units.push(Unit::new_army(Power::Russia, p("lvn")));
    phase.data.units.push(Unit::new_army(Power::Russia, p("ukr")));
    let context = &mut PhaseContext::new();
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders.len(), 1);
    assert_eq!(phase.data.orders[0].to_string(), "Remove A lvn");
}

/// 6.J.5 TEST CASE, CIVIL DISORDER TWO FLEETS WITH DIFFERENT DISTANCE
/// If two fleets have different distance from the home supply centers, then the fleet with the greatest distance has to be removed. Note that fleets cannot go over land.
///
/// Russia has to remove one.
/// Russia owns St Petersburg.
/// Russia has fleets in Skagerrak and Berlin.
/// Russia does not order a disband.
/// The distance of the fleet in Berlin is three, the fleet in Skagerrak has distance two (via Norway). So, the fleet in Berlin has to be removed.
#[test]
fn test_datc_6_j_5() {}

/// 6.J.6. TEST CASE, CIVIL DISORDER TWO FLEETS WITH EQUAL DISTANCE
/// Alphabetical order is used, when two fleets have equal distance to the home supply centers.
///
/// Russia has to remove one.
/// Russia owns Munich.
/// Russia has fleets in Gulf of Bothnia and North Sea.
/// Russia does not order a disband.
/// Note, that in 2023 rules distance is calculated to owned supply centers (instead of home supply centers). Also, for distance calculations both armies and fleets can take both land and sea. Both distances are three. The fleet in Gulf of Bothnia is removed, because it appears first in alphabetical order.
#[test]
fn test_datc_6_j_6() {}

/// 6.J.7. TEST CASE, CIVIL DISORDER TWO FLEETS AND ARMY WITH EQUAL DISTANCE
/// In removal, the fleet has precedence over an army. In this case there are two fleets, to make the test more complex.
///
/// Russia has to remove one.
/// Russia owns St Petersburg and Warsaw.
/// Russia has an army in Bohemia, a fleet in Skagerrak and a fleet in the North Sea.
/// Russia does not order a disband.
/// The distances of the army and the fleets to one of the owned supply centers are two. The fleets take precedence above the army (although the army is alphabetical first). The fleet in the North Sea is alphabetical first, compared to Skagerrak and has to be removed.
#[test]
fn test_datc_6_j_7() {}

/// 6.J.8. TEST CASE, CIVIL DISORDER A FLEET WITH SHORTER DISTANCE THEN THE ARMY
/// If the fleet has a shorter distance than the army, the army is removed.
///
/// Russia has to remove one.
/// Russia has an army in Tyrolia and a fleet in the Baltic Sea.
/// Russia owns Warsaw.
/// Russia does not order a disband.
/// The distance of the army to Warsaw is three while the distance of the fleet is two. So, the army is removed.
#[test]
fn test_datc_6_j_8() {}

/// 6.J.9. TEST CASE, CIVIL DISORDER MUST BE COUNTED FROM BOTH COASTS
/// Distance must be calculated from both coasts.
///
/// Russia has to remove one.
/// Russia owns St Petersburg and Sevastopol.
/// Russia has armies in Greece and Sevastopol and a fleet in the Baltic Sea.
/// Russia does not order a disband.
/// The distance of the fleet to St Petersburg(nc) is three but to St Petersburg(sc) is two. So, the army in Greece must be removed.
///
/// Russia has to remove one.
/// Russia owns St Petersburg and Sevastopol.
/// Russia has armies in Greece and Sevastopol and a fleet in Skagerrak.
/// Russia does not order a disband.
/// The distance of the fleet to St Petersburg(sc) is three but to St Petersburg(nc) is two. So, the army in Greece must be removed.
#[test]
fn test_datc_6_j_9() {}

/// 6.J.10. TEST CASE, CIVIL DISORDER COUNTING CONVOYING DISTANCE
/// For calculating the distance for armies all areas must be considered.
///
/// Italy has to remove one.
/// Italy owns Naples.
/// Italy has armies in Greece and Piedmont.
/// Italy does not order a disband.
/// The distance from Greece to owned supply center is five over land. However, for distance calculation it can go over water and arrive in two steps. The army in Piedmont has to be removed.
#[test]
fn test_datc_6_j_10() {}

/// 6.J.11. TEST CASE, DISTANCE TO OWNED SUPPLY CENTER
/// The 2023 rules say that distance must be calculated to owned supply center instead of home supply center (as it was in the older rulebooks).
///
/// Italy has to remove one.
/// Italy owns Warsaw.
/// Italy has armies in Warsaw and Tuscany.
/// Italy does not order a disband.
/// The army in Tuscany is removed and Italy will continue defending its supply center in Warsaw. Under older rulebooks the army in Tuscany was kept.
#[test]
fn test_datc_6_j_11() {}
