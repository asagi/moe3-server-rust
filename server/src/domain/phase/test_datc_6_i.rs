//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6I]
//!
//! * 6.I. TEST CASES, BUILDING
//!
//! [DATC_6I]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.I

use super::super::order::*;
use super::super::phase::adjustment_order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.I.1. TEST CASE, TOO MANY BUILD ORDERS
/// Check how program reacts when someone orders too many builds.
///
/// Germany may build one:
///
/// Germany:
///     Build A Warsaw
///     Build A Kiel
///     Build A Munich
///
/// Program should not build all three, but handle it in another way. See issue 4.D.4.
/// I prefer that the build orders are just handled one by one until all allowed units are build.
/// According to this preference, the build in Warsaw fails,
/// the build in Kiel succeeds and the build in Munich fails.
#[test]
fn test_datc_6_i_1() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Germany, "kie"));
    phase.data.territories.push(Territory::new(Power::Germany, "mun"));
    phase.data.units.push(Unit::new_army(Power::Germany, p("ber")));
    let context = &mut PhaseContext::new();
    let unit_g_war = Unit::new_army(Power::Germany, p("war"));
    let unit_g_kie = Unit::new_army(Power::Germany, p("kie"));
    let unit_g_mun = Unit::new_army(Power::Germany, p("mun"));
    phase.data.orders.push(unit_g_war.build());
    phase.data.orders.push(unit_g_kie.build());
    phase.data.orders.push(unit_g_mun.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
}

/// 6.I.2. TEST CASE, FLEETS CANNOT BE BUILD IN LAND AREAS
/// Physical this is possible, but it is still not allowed.
///
/// Russia has one build and Moscow is empty.
///
/// Russia:
///     Build F Moscow
///
/// See issue 4.C.4. Some game masters will change the order and build an army in Moscow.
/// I prefer that the build fails.
#[test]
fn test_datc_6_i_2() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Russia, "mos"));
    let context = &mut PhaseContext::new();
    let unit_r_mos = Unit::new_fleet(Power::Russia, p("mos"));
    phase.data.orders.push(unit_r_mos.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.I.3. TEST CASE, SUPPLY CENTER MUST BE EMPTY FOR BUILDING
/// You can't have two units in a sector.
/// So, you can't build when there is a unit in the supply center.
///
/// Germany may build a unit but has an army in Berlin. Germany orders the following:
///
/// Germany:
///     Build A Berlin
///
/// Build fails.
#[test]
fn test_datc_6_i_3() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Germany, "ber"));
    phase.data.units.push(Unit::new_army(Power::Germany, p("ber")));
    let context = &mut PhaseContext::new();
    let unit_g_ber = Unit::new_army(Power::Germany, p("ber"));
    phase.data.orders.push(unit_g_ber.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.I.4. TEST CASE, BOTH COASTS MUST BE EMPTY FOR BUILDING
/// If a sector is occupied on one coast, the other coast cannot be used for building.
///
/// Russia may build a unit and has a fleet in St Petersburg(sc). Russia orders the following:
///
/// Russia:
///     Build A St Petersburg(nc)
///
/// Build fails.
#[test]
fn test_datc_6_i_4() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Russia, "stp"));
    phase.data.units.push(Unit::new_fleet(Power::Russia, p("stp_sc")));
    let context = &mut PhaseContext::new();
    let unit_r_stp_nc = Unit::new_army(Power::Russia, p("stp_nc"));
    phase.data.orders.push(unit_r_stp_nc.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.I.5. TEST CASE, BUILDING IN HOME SUPPLY CENTER THAT IS NOT OWNED
/// Building a unit is only allowed when supply center is a home supply center and is owned.
/// If not owned, build fails.
///
/// Russia captured Berlin in Fall, but left in the next year.
/// Germany captured other supply centers, but without recapturing Berling it may not build in Berlin.
///
/// Germany:
///     Build A Berlin
///
/// Build fails.
#[test]
fn test_datc_6_i_5() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Russia, "ber"));
    phase.data.territories.push(Territory::new(Power::Germany, "bel"));
    let context = &mut PhaseContext::new();
    let unit_g_ber = Unit::new_army(Power::Germany, p("ber"));
    phase.data.orders.push(unit_g_ber.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.I.6. TEST CASE, BUILDING IN OWNED SUPPLY CENTER THAT IS NOT A HOME SUPPLY CENTER
/// Building a unit is only allowed when supply center is a home supply center and is owned.
/// If it is not a home supply center, the build fails.
///
/// Germany owns Warsaw, Warsaw is empty and Germany may build one unit.
///
/// Germany:
///     Build A Warsaw
///
/// Build fails.
#[test]
fn test_datc_6_i_6() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Germany, "war"));
    let context = &mut PhaseContext::new();
    let unit_g_war = Unit::new_army(Power::Germany, p("war"));
    phase.data.orders.push(unit_g_war.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.I.7. TEST CASE, ONLY ONE BUILD IN A HOME SUPPLY CENTER
/// If you may build two units, you can still only build one in a supply center.
///
/// Russia owns Moscow, Moscow is empty and Russia may build two units.
///
/// Russia:
///     Build A Moscow
///     Build A Moscow
///
/// The second build should fail.
#[test]
fn test_datc_6_i_7() {
    let phase = &mut Phase::new_adjustment(1901, 5);
    phase.data.territories.push(Territory::new(Power::Russia, "mos"));
    let context = &mut PhaseContext::new();
    let unit_r_mos_1 = Unit::new_army(Power::Russia, p("mos"));
    let unit_r_mos_2 = Unit::new_army(Power::Russia, p("mos"));
    phase.data.orders.push(unit_r_mos_1.build());
    phase.data.orders.push(unit_r_mos_2.build());
    resolve_orders_for_adjustment_phase(phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
}
