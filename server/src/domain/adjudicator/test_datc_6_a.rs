//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6A]
//!
//! * 6.A. TEST CASES, BASIC CHECKS
//!
//! [DATC_6A]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.A

use crate::domain::models::order::*;
use crate::domain::models::phase::*;
use crate::domain::models::power::*;
use crate::domain::tests::a;
use crate::domain::tests::f;
use crate::domain::tests::p;

/// 6.A.1. TEST CASE, MOVING TO AN AREA THAT IS NOT A NEIGHBOUR
///  Check if an illegal move (without convoy) will fail.
///
///  England:
///     F North Sea - Picardy
///
///  Order should fail.
#[test]
fn test_datc_6_a_1() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    phase.data.units.push(unit_e_nth);
    phase.data.orders.push(unit_e_nth.move_to(p("pic")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.2. TEST CASE, MOVE ARMY TO SEA
/// Check if an army could not be moved to open sea.
///
/// England:
///     A Liverpool - Irish Sea
///
/// Order should fail.
#[test]
fn test_datc_6_a_2() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_lvp = a("e", "lvp");
    phase.data.units.push(unit_e_lvp);
    phase.data.orders.push(unit_e_lvp.move_to(p("iri")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_e_lvp));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.3. TEST CASE, MOVE FLEET TO LAND
/// Check whether a fleet cannot move to land.
///
/// Germany:
///     F Kiel - Munich
///
/// Order should fail.
#[test]
fn test_datc_6_a_3() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_kie = f("g", "kie");
    phase.data.units.push(unit_g_kie);
    phase.data.orders.push(unit_g_kie.move_to(p("mun")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.4. TEST CASE, MOVE TO OWN SECTOR
/// Moving to the same sector is an illegal move (2023 rulebook, page 7,
/// "An Army can be ordered to move into an adjacent inland or coastal province.").
///
/// Germany:
///     F Kiel - Kiel
///
/// Program should not crash.
#[test]
fn test_datc_6_a_4() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_kie = f("g", "kie");
    phase.data.units.push(unit_g_kie);
    phase.data.orders.push(unit_g_kie.move_to(p("kie")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.5. TEST CASE, MOVE TO OWN SECTOR WITH CONVOY
/// Moving to the same sector is still illegal with convoy (2023 rulebook, page 7,
/// "Note: An Army can move across water provinces from one coastal province to another...").
///
/// England:
///     F North Sea Convoys A Yorkshire - Yorkshire
///     A Yorkshire - Yorkshire
///     A Liverpool Supports A Yorkshire - Yorkshire
///
/// Germany:
///     F London - Yorkshire
///     A Wales Supports F London - Yorkshire
///
/// The move of the army in Yorkshire is illegal.
/// This makes the support of Liverpool also illegal and without the support,
/// the Germans have a stronger force. The army in London dislodges the army in Yorkshire.
#[test]
fn test_datc_6_a_5() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let mut unit_e_yor = a("e", "yor");
    let unit_e_lvp = a("e", "lvp");
    let unit_g_lon = f("g", "lon");
    let unit_g_wal = a("g", "wal");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_e_lvp);
    phase.data.units.push(unit_g_lon);
    phase.data.units.push(unit_g_wal);
    phase.data.orders.push(unit_e_nth.convoy(unit_e_yor, p("yor")));
    phase.data.orders.push(unit_e_yor.move_to(p("yor")));
    phase.data.orders.push(unit_e_lvp.support_move(unit_e_yor, p("yor")));
    phase.data.orders.push(unit_g_lon.move_to(p("yor")));
    phase.data.orders.push(unit_g_wal.support_move(unit_g_lon, p("yor")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&unit_e_yor.dislodged_from(p("lon"))));
    assert!(phase.data.units.contains(&unit_e_lvp));
    assert!(phase.data.units.contains(&f("g", "yor")));
    assert!(phase.data.units.contains(&unit_g_wal));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.6. TEST CASE, ORDERING A UNIT OF ANOTHER COUNTRY
/// Check whether someone cannot order a unit that is not his own unit.
/// England has a fleet in London.
///
/// Germany:
///     F London - North Sea
///
/// Order should fail.
#[test]
fn test_datc_6_a_6() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_lon = f("e", "lon");
    phase.data.units.push(unit_e_lon);
    phase
        .data
        .orders
        .push(unit_e_lon.move_to(p("nth")).assumed_by(Power::Germany));
    resolve_orders_for_main_phase(&mut phase);
    assert!(phase.data.orders[0].is_unresolved());
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_e_lon));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.7. TEST CASE, ONLY ARMIES CAN BE CONVOYED
/// A fleet cannot be convoyed.
///
/// England:
///     F London - Belgium
///     F North Sea Convoys A London - Belgium
///
/// Move from London to Belgium should fail.
#[test]
fn test_datc_6_a_7() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_lon = f("e", "lon");
    let unit_e_nth = f("e", "nth");
    phase.data.units.push(unit_e_lon);
    phase.data.units.push(unit_e_nth);
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 2);
    assert!(phase.data.units.contains(&unit_e_lon));
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.8. TEST CASE, SUPPORT TO HOLD YOURSELF IS NOT POSSIBLE
/// An army cannot get an additional hold power by supporting itself.
///
/// Italy:
///     A Venice - Trieste
///     A Tyrolia Supports A Venice - Trieste
///
/// Austria:
///     F Trieste Supports F Trieste
///
/// The army in Trieste should be dislodged.
#[test]
fn test_datc_6_a_8() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_i_ven = a("i", "ven");
    let unit_i_tyr = a("i", "tyr");
    let mut unit_a_tri = f("a", "tri");
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_tyr);
    phase.data.units.push(unit_a_tri);
    phase.data.orders.push(unit_i_ven.move_to(p("tri")));
    phase.data.orders.push(unit_i_tyr.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_a_tri.support_hold(unit_a_tri));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&a("i", "tri")));
    assert!(phase.data.units.contains(&unit_i_tyr));
    assert!(phase.data.units.contains(&unit_a_tri.dislodged_from(p("ven"))));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.9. TEST CASE, FLEETS MUST FOLLOW COAST IF NOT ON SEA
/// If two provinces are adjacent, that does not mean that a fleet can move between those two provinces.
/// An implementation that only holds one list of adjacent provinces for each province is incorrect.
///
/// Italy:
///     F Rome - Venice
///
/// Move fails. An army can go from Rome to Venice, but a fleet cannot.
#[test]
fn test_datc_6_a_9() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_i_rom = f("i", "rom");
    phase.data.units.push(unit_i_rom);
    phase.data.orders.push(unit_i_rom.move_to(p("ven")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 1);
    assert!(phase.data.units.contains(&unit_i_rom));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.10. TEST CASE, SUPPORT ON UNREACHABLE DESTINATION NOT POSSIBLE
/// The destination of the move that is supported must be reachable by the supporting unit.
///
/// Austria:
///     A Venice Hold
///
/// Italy:
///     F Rome Supports A Apulia - Venice
///     A Apulia - Venice
///
/// The support of Rome is illegal, because Venice cannot be reached from Rome by a fleet.
/// Venice is not dislodged.
#[test]
fn test_datc_6_a_10() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_ven = a("a", "ven");
    let unit_i_rom = f("i", "rom");
    let unit_i_apu = a("i", "apu");
    phase.data.units.push(unit_a_ven);
    phase.data.units.push(unit_i_rom);
    phase.data.units.push(unit_i_apu);
    phase.data.orders.push(unit_a_ven.hold());
    phase.data.orders.push(unit_i_rom.support_move(unit_i_apu, p("ven")));
    phase.data.orders.push(unit_i_apu.move_to(p("ven")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_a_ven));
    assert!(phase.data.units.contains(&unit_i_rom));
    assert!(phase.data.units.contains(&unit_i_apu));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.A.11. TEST CASE, SIMPLE BOUNCE
/// Two armies bouncing on each other.
///
/// Austria:
///     A Vienna - Tyrolia
///
/// Italy:
///     A Venice - Tyrolia
///
/// The two units bounce.
#[test]
fn test_datc_6_a_11() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_vie = a("a", "vie");
    let unit_i_ven = a("i", "ven");
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_i_ven);
    phase.data.orders.push(unit_a_vie.move_to(p("tyr")));
    phase.data.orders.push(unit_i_ven.move_to(p("tyr")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 2);
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&unit_i_ven));
    assert!(phase.data.standoff_codes.contains(&"tyr".to_string()));
}

/// 6.A.12. TEST CASE, BOUNCE OF THREE UNITS
/// If three units move to the same area,
/// the adjudicator should not bounce the first two units and then let the third unit go to the now open area.
///
/// Austria:
///     A Vienna - Tyrolia
///
/// Germany:
///     A Munich - Tyrolia
///
/// Italy:
///     A Venice - Tyrolia
///
/// The three units bounce.
#[test]
fn test_datc_6_a_12() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_vie = a("a", "vie");
    let unit_g_mun = a("g", "mun");
    let unit_i_ven = a("i", "ven");
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_g_mun);
    phase.data.units.push(unit_i_ven);
    phase.data.orders.push(unit_a_vie.move_to(p("tyr")));
    phase.data.orders.push(unit_g_mun.move_to(p("tyr")));
    phase.data.orders.push(unit_i_ven.move_to(p("tyr")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&unit_g_mun));
    assert!(phase.data.units.contains(&unit_i_ven));
    assert!(phase.data.standoff_codes.contains(&"tyr".to_string()));
}
