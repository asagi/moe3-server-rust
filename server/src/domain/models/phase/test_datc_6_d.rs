//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6D]
//!
//! * 6.D. TEST CASES, SUPPORTS AND DISLODGES
//!
//! [DATC_6D]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.D

use crate::domain::models::order::*;
use crate::domain::models::phase::methods::*;
use crate::domain::tests::a;
use crate::domain::tests::f;
use crate::domain::tests::p;

/// 6.D.1. TEST CASE, SUPPORTED HOLD CAN PREVENT DISLODGEMENT
/// The simplest support to hold order.
///
/// Austria:
///     F Adriatic Sea Supports A Trieste - Venice
///     A Trieste - Venice
///
/// Italy:
///     A Venice Hold
///     A Tyrolia Supports A Venice
///
/// The support of Tyrolia prevents the army in Venice from being dislodged.
/// The army in Trieste will not move.
#[test]
fn test_datc_6_d_1() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_adr = f("a", "adr");
    let unit_a_tri = a("a", "tri");
    let unit_i_ven = a("i", "ven");
    let unit_i_tyr = a("i", "tyr");
    phase.data.units.push(unit_a_adr);
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_tyr);
    phase.data.orders.push(unit_a_adr.support_move(unit_a_tri, p("ven")));
    phase.data.orders.push(unit_a_tri.move_to(p("ven")));
    phase.data.orders.push(unit_i_ven.hold());
    phase.data.orders.push(unit_i_tyr.support_hold(unit_i_ven));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_a_adr));
    assert!(phase.data.units.contains(&unit_a_tri));
    assert!(phase.data.units.contains(&unit_i_ven));
    assert!(phase.data.units.contains(&unit_i_tyr));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.2. TEST CASE, A MOVE CUTS SUPPORT ON HOLD
/// The simplest support on hold cut.
///
/// Austria:
///     F Adriatic Sea Supports A Trieste - Venice
///     A Trieste - Venice
///     A Vienna - Tyrolia
///
/// Italy:
///     A Venice Hold
///     A Tyrolia Supports A Venice
///
/// The support of Tyrolia is cut by the army in Vienna.
/// That means that the army in Venice is dislodged by the army from Trieste.
#[test]
fn test_datc_6_d_2() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_adr = f("a", "adr");
    let unit_a_tri = a("a", "tri");
    let unit_a_vie = a("a", "vie");
    let mut unit_i_ven = a("i", "ven");
    let unit_i_tyr = a("i", "tyr");
    phase.data.units.push(unit_a_adr);
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_tyr);
    phase.data.orders.push(unit_a_adr.support_move(unit_a_tri, p("ven")));
    phase.data.orders.push(unit_a_tri.move_to(p("ven")));
    phase.data.orders.push(unit_a_vie.move_to(p("tyr")));
    phase.data.orders.push(unit_i_ven.hold());
    phase.data.orders.push(unit_i_tyr.support_hold(unit_i_ven));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Cut);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_a_adr));
    assert!(phase.data.units.contains(&a("a", "ven")));
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&unit_i_ven.set_dislodged_from(Some(p("tri")))));
    assert!(phase.data.units.contains(&unit_i_tyr));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.3. TEST CASE, A MOVE CUTS SUPPORT ON MOVE
/// The simplest support on move cut.
///
/// Austria:
///     F Adriatic Sea Supports A Trieste - Venice
///     A Trieste - Venice
///
/// Italy:
///     A Venice Hold
///     F Ionian Sea - Adriatic Sea
///
/// The support of the fleet in the Adriatic Sea is cut.
/// That means that the army in Venice will not be dislodged and the army in Trieste stays in Trieste.
#[test]
fn test_datc_6_d_3() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_adr = f("a", "adr");
    let unit_a_tri = a("a", "tri");
    let unit_i_ven = a("i", "ven");
    let unit_i_ion = f("i", "ion");
    phase.data.units.push(unit_a_adr);
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_ion);
    phase.data.orders.push(unit_a_adr.support_move(unit_a_tri, p("ven")));
    phase.data.orders.push(unit_a_tri.move_to(p("ven")));
    phase.data.orders.push(unit_i_ven.hold());
    phase.data.orders.push(unit_i_ion.move_to(p("adr")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_a_adr));
    assert!(phase.data.units.contains(&unit_a_tri));
    assert!(phase.data.units.contains(&a("i", "ven")));
    assert!(phase.data.units.contains(&unit_i_ion));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.4. TEST CASE, SUPPORT TO HOLD ON UNIT SUPPORTING A HOLD ALLOWED
/// A unit that is supporting a hold, can receive a hold support.
///
/// Germany:
///     A Berlin Supports F Kiel
///     F Kiel Supports A Berlin
///
/// Russia:
///     F Baltic Sea Supports A Prussia - Berlin
///     A Prussia - Berlin
///
/// The Russian move from Prussia to Berlin fails.
#[test]
fn test_datc_6_d_4() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_r_bal = f("r", "bal");
    let unit_r_pru = a("r", "pru");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_r_bal);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_g_ber.support_hold(unit_g_kie));
    phase.data.orders.push(unit_g_kie.support_hold(unit_g_ber));
    phase.data.orders.push(unit_r_bal.support_move(unit_r_pru, p("ber")));
    phase.data.orders.push(unit_r_pru.move_to(p("ber")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&unit_r_bal));
    assert!(phase.data.units.contains(&unit_r_pru));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.5. TEST CASE, SUPPORT TO HOLD ON UNIT SUPPORTING A MOVE ALLOWED
/// A unit that is supporting a move, can receive a hold support.
///
/// Germany:
///     A Berlin Supports A Munich - Silesia
///     F Kiel Supports A Berlin
///     A Munich - Silesia
///
/// Russia:
///     F Baltic Sea Supports A Prussia - Berlin
///     A Prussia - Berlin
///
/// The Russian move from Prussia to Berlin fails.
#[test]
fn test_datc_6_d_5() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_g_mun = a("g", "mun");
    let unit_r_bal = f("r", "bal");
    let unit_r_pru = a("r", "pru");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_g_mun);
    phase.data.units.push(unit_r_bal);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_g_ber.support_move(unit_g_mun, p("sil")));
    phase.data.orders.push(unit_g_kie.support_hold(unit_g_ber));
    phase.data.orders.push(unit_g_mun.move_to(p("sil")));
    phase.data.orders.push(unit_r_bal.support_move(unit_r_pru, p("ber")));
    phase.data.orders.push(unit_r_pru.move_to(p("ber")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&a("g", "sil")));
    assert!(phase.data.units.contains(&unit_r_bal));
    assert!(phase.data.units.contains(&unit_r_pru));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.6. TEST CASE, SUPPORT TO HOLD ON CONVOYING UNIT ALLOWED
/// A unit that is convoying, can receive a hold support.
///
/// Germany:
///     A Berlin - Sweden
///     F Baltic Sea Convoys A Berlin - Sweden
///     F Prussia Supports F Baltic Sea
///
/// Russia:
///     F Livonia - Baltic Sea
///     F Gulf of Bothnia Supports F Livonia - Baltic Sea
///
/// The Russian move from Livonia to the Baltic Sea fails. The convoy from Berlin to Sweden succeeds.
#[test]
fn test_datc_6_d_6() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_bal = f("g", "bal");
    let unit_g_pru = f("g", "pru");
    let unit_r_lvn = f("r", "lvn");
    let unit_r_bot = f("r", "bot");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_bal);
    phase.data.units.push(unit_g_pru);
    phase.data.units.push(unit_r_lvn);
    phase.data.units.push(unit_r_bot);
    phase.data.orders.push(unit_g_ber.move_to(p("swe")));
    phase.data.orders.push(unit_g_bal.convoy(unit_g_ber, p("swe")));
    phase.data.orders.push(unit_g_pru.support_hold(unit_g_bal));
    phase.data.orders.push(unit_r_lvn.move_to(p("bal")));
    phase.data.orders.push(unit_r_bot.support_move(unit_r_lvn, p("bal")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&a("g", "swe")));
    assert!(phase.data.units.contains(&unit_g_bal));
    assert!(phase.data.units.contains(&unit_g_pru));
    assert!(phase.data.units.contains(&unit_r_lvn));
    assert!(phase.data.units.contains(&unit_r_bot));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.7. TEST CASE, SUPPORT TO HOLD ON MOVING UNIT NOT ALLOWED
/// A unit that is moving, cannot receive a hold support for the situation that the move fails.
///
/// Germany:
///     F Baltic Sea - Sweden
///     F Prussia Supports F Baltic Sea
///
/// Russia:
///     F Livonia - Baltic Sea
///     F Gulf of Bothnia Supports F Livonia - Baltic Sea
///     A Finland - Sweden
///
/// The support of the fleet in Prussia fails.
/// The fleet in Baltic Sea will bounce on the Russian army in Finland
/// and will be dislodged by the Russian fleet from Livonia when it returns to the Baltic Sea.
#[test]
fn test_datc_6_d_7() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let mut unit_g_bal = f("g", "bal");
    let unit_g_pru = f("g", "pru");
    let unit_r_lvn = f("r", "lvn");
    let unit_r_bot = f("r", "bot");
    let unit_r_fin = a("r", "fin");
    phase.data.units.push(unit_g_bal);
    phase.data.units.push(unit_g_pru);
    phase.data.units.push(unit_r_lvn);
    phase.data.units.push(unit_r_bot);
    phase.data.units.push(unit_r_fin);
    phase.data.orders.push(unit_g_bal.move_to(p("swe")));
    phase.data.orders.push(unit_g_pru.support_hold(unit_g_bal));
    phase.data.orders.push(unit_r_lvn.move_to(p("bal")));
    phase.data.orders.push(unit_r_bot.support_move(unit_r_lvn, p("bal")));
    phase.data.orders.push(unit_r_fin.move_to(p("swe")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_g_bal.set_dislodged_from(Some(p("lvn")))));
    assert!(phase.data.units.contains(&unit_g_pru));
    assert!(phase.data.units.contains(&f("r", "bal")));
    assert!(phase.data.units.contains(&unit_r_bot));
    assert!(phase.data.units.contains(&unit_r_fin));
    assert!(phase.data.standoff_codes.contains(&"swe".to_string()));
}

/// 6.D.8. TEST CASE, FAILED CONVOY CANNOT RECEIVE HOLD SUPPORT
/// If a convoy fails because of disruption of the convoy or when the right convoy orders are not given,
/// then the army to be convoyed cannot receive support in hold, since it still tried to move.
///
/// Austria:
///     F Ionian Sea Hold
///     A Serbia Supports A Albania - Greece
///     A Albania - Greece
///
/// Turkey:
///     A Greece - Naples
///     A Bulgaria Supports A Greece
///
/// There was a possible convoy from Greece to Naples, before the orders were made public (via the Ionian Sea).
/// This means that the order of Greece to Naples should never be treated as illegal order
/// and be changed in a hold order able to receive hold support (see also issue 4.E.1).
/// Therefore, the support in Bulgaria fails and the army in Greece is dislodged by the army in Albania.
#[test]
fn test_datc_6_d_8() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_ion = f("a", "ion");
    let unit_a_ser = a("a", "ser");
    let unit_a_alb = a("a", "alb");
    let mut unit_t_gre = a("t", "gre");
    let unit_t_bul = a("t", "bul");
    phase.data.units.push(unit_a_ion);
    phase.data.units.push(unit_a_ser);
    phase.data.units.push(unit_a_alb);
    phase.data.units.push(unit_t_gre);
    phase.data.units.push(unit_t_bul);
    phase.data.orders.push(unit_a_ion.hold());
    phase.data.orders.push(unit_a_ser.support_move(unit_a_alb, p("gre")));
    phase.data.orders.push(unit_a_alb.move_to(p("gre")));
    phase.data.orders.push(unit_t_gre.move_to(p("nap")));
    phase.data.orders.push(unit_t_bul.support_hold(unit_t_gre));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_a_ion));
    assert!(phase.data.units.contains(&unit_a_ser));
    assert!(phase.data.units.contains(&a("a", "gre")));
    assert!(phase.data.units.contains(&unit_t_gre.set_dislodged_from(Some(p("alb")))));
    assert!(phase.data.units.contains(&unit_t_bul));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.9. TEST CASE, SUPPORT TO MOVE ON HOLDING UNIT NOT ALLOWED
/// A unit that is holding cannot receive a support in moving.
///
/// Italy:
///     A Venice - Trieste
///     A Tyrolia Supports A Venice - Trieste
///
/// Austria:
///     A Albania Supports A Trieste - Serbia
///     A Trieste Hold
///
/// The support of the army in Albania fails and the army in Trieste is dislodged by the army from Venice.
#[test]
fn test_datc_6_d_9() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_i_ven = a("i", "ven");
    let unit_i_tyr = a("i", "tyr");
    let unit_a_alb = a("a", "alb");
    let mut unit_a_tri = a("a", "tri");
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_tyr);
    phase.data.units.push(unit_a_alb);
    phase.data.units.push(unit_a_tri);
    phase.data.orders.push(unit_i_ven.move_to(p("tri")));
    phase.data.orders.push(unit_i_tyr.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_a_alb.support_move(unit_a_tri, p("ser")));
    phase.data.orders.push(unit_a_tri.hold());
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&a("i", "tri")));
    assert!(phase.data.units.contains(&unit_i_tyr));
    assert!(phase.data.units.contains(&unit_a_alb));
    assert!(phase.data.units.contains(&unit_a_tri.set_dislodged_from(Some(p("ven")))));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.10. TEST CASE, SELF DISLODGMENT PROHIBITED
/// A unit may not dislodge a unit of the same great power.
///
/// Germany:
///     A Berlin Hold
///     F Kiel - Berlin
///     A Munich Supports F Kiel - Berlin
///
/// Move to Berlin fails.
#[test]
fn test_datc_6_d_10() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = a("g", "kie");
    let unit_g_mun = a("g", "mun");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_g_mun);
    phase.data.orders.push(unit_g_ber.hold());
    phase.data.orders.push(unit_g_kie.move_to(p("ber")));
    phase.data.orders.push(unit_g_mun.support_move(unit_g_kie, p("ber")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&unit_g_mun));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.11. TEST CASE, NO SELF DISLODGMENT OF RETURNING UNIT
/// Idem.
///
/// Germany:
///     A Berlin - Prussia
///     F Kiel - Berlin
///     A Munich Supports F Kiel - Berlin
///
/// Russia:
///     A Warsaw - Prussia
///
/// Army in Berlin bounces, but is not dislodged by own unit.
#[test]
fn test_datc_6_d_11() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_g_mun = a("g", "mun");
    let unit_r_war = a("r", "war");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_g_mun);
    phase.data.units.push(unit_r_war);
    phase.data.orders.push(unit_g_ber.move_to(p("pru")));
    phase.data.orders.push(unit_g_kie.move_to(p("ber")));
    phase.data.orders.push(unit_g_mun.support_move(unit_g_kie, p("ber")));
    phase.data.orders.push(unit_r_war.move_to(p("pru")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&unit_g_mun));
    assert!(phase.data.units.contains(&unit_r_war));
    assert!(phase.data.standoff_codes.contains(&"pru".to_string()));
}

/// 6.D.12. TEST CASE, SUPPORTING A FOREIGN UNIT TO DISLODGE OWN UNIT PROHIBITED
/// You may not help another power in dislodging your own unit.
///
/// Austria:
///     F Trieste Hold
///     A Vienna Supports A Venice - Trieste
///
/// Italy:
///     A Venice - Trieste
///
/// No dislodgment of fleet in Trieste.
#[test]
fn test_datc_6_d_12() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_tri = f("a", "tri");
    let unit_a_vie = a("a", "vie");
    let unit_i_ven = a("i", "ven");
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_i_ven);
    phase.data.orders.push(unit_a_tri.hold());
    phase.data.orders.push(unit_a_vie.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_i_ven.move_to(p("tri")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_a_tri));
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&unit_i_ven));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.13. TEST CASE, SUPPORTING A FOREIGN UNIT TO DISLODGE A RETURNING OWN UNIT PROHIBITED
/// Idem.
///
/// Austria:
///     F Trieste - Adriatic Sea
///     A Vienna Supports A Venice - Trieste
///
/// Italy:
///     A Venice - Trieste
///     F Apulia - Adriatic Sea
///
/// No dislodgment of fleet in Trieste.
#[test]
fn test_datc_6_d_13() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_tri = f("a", "tri");
    let unit_a_vie = a("a", "vie");
    let unit_i_ven = a("i", "ven");
    let unit_i_apu = f("i", "apu");
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_apu);
    phase.data.orders.push(unit_a_tri.move_to(p("adr")));
    phase.data.orders.push(unit_a_vie.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_i_ven.move_to(p("tri")));
    phase.data.orders.push(unit_i_apu.move_to(p("adr")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_a_tri));
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&unit_i_ven));
    assert!(phase.data.units.contains(&unit_i_apu));
    assert!(phase.data.standoff_codes.contains(&"adr".to_string()));
}

/// 6.D.14. TEST CASE, SUPPORTING A FOREIGN UNIT IS NOT ENOUGH TO PREVENT DISLODGEMENT
/// If a foreign unit has enough support to dislodge your unit,
/// you may not prevent that dislodgement by supporting the attack.
///
/// Austria:
///     F Trieste Hold
///     A Vienna Supports A Venice - Trieste
///
/// Italy:
///     A Venice - Trieste
///     A Tyrolia Supports A Venice - Trieste
///     F Adriatic Sea Supports A Venice - Trieste
///
/// The fleet in Trieste is dislodged.
#[test]
fn test_datc_6_d_14() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let mut unit_a_tri = f("a", "tri");
    let unit_a_vie = a("a", "vie");
    let unit_i_ven = a("i", "ven");
    let unit_i_tyr = a("i", "tyr");
    let unit_i_adr = f("i", "adr");
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_tyr);
    phase.data.units.push(unit_i_adr);
    phase.data.orders.push(unit_a_tri.hold());
    phase.data.orders.push(unit_a_vie.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_i_ven.move_to(p("tri")));
    phase.data.orders.push(unit_i_tyr.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_i_adr.support_move(unit_i_ven, p("tri")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_a_tri.set_dislodged_from(Some(p("ven")))));
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&a("i", "tri")));
    assert!(phase.data.units.contains(&unit_i_tyr));
    assert!(phase.data.units.contains(&unit_i_adr));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.15. TEST CASE, DEFENDER CANNOT CUT SUPPORT FOR ATTACK ON ITSELF
/// A unit that is attacked by a supported unit cannot prevent dislodgement
/// by guessing which of the units will do the support.
///
/// Russia:
///     F Constantinople Supports F Black Sea - Ankara
///     F Black Sea - Ankara
///
/// Turkey:
///     F Ankara - Constantinople
///
/// The support of Constantinople is not cut
/// and the fleet in Ankara is dislodged by the fleet in the Black Sea.
#[test]
fn test_datc_6_d_15() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_r_con = f("r", "con");
    let unit_r_bla = f("r", "bla");
    let mut unit_t_ank = f("t", "ank");
    phase.data.units.push(unit_r_con);
    phase.data.units.push(unit_r_bla);
    phase.data.units.push(unit_t_ank);
    phase.data.orders.push(unit_r_con.support_move(unit_r_bla, p("ank")));
    phase.data.orders.push(unit_r_bla.move_to(p("ank")));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_r_con));
    assert!(phase.data.units.contains(&f("r", "ank")));
    assert!(phase.data.units.contains(&unit_t_ank.set_dislodged_from(Some(p("bla")))));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.16. TEST CASE, CONVOYING A UNIT DISLODGING A UNIT OF SAME POWER IS ALLOWED
/// It is allowed to convoy a foreign unit that dislodges your own unit is allowed.
///
/// England:
///     A London Hold
///     F North Sea Convoys A Belgium - London
///
/// France:
///     F English Channel Supports A Belgium - London
///     A Belgium - London
///
/// The English army in London is dislodged by the French army coming from Belgium.
#[test]
fn test_datc_6_d_16() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let mut unit_e_lon = a("e", "lon");
    let unit_e_nth = f("e", "nth");
    let unit_f_eng = f("f", "eng");
    let unit_f_bel = a("f", "bel");
    phase.data.units.push(unit_e_lon);
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_f_eng);
    phase.data.units.push(unit_f_bel);
    phase.data.orders.push(unit_e_lon.hold());
    phase.data.orders.push(unit_e_nth.convoy(unit_f_bel, p("lon")));
    phase.data.orders.push(unit_f_eng.support_move(unit_f_bel, p("lon")));
    phase.data.orders.push(unit_f_bel.move_to(p("lon")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_e_lon.set_dislodged_from(Some(p("bel")))));
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&unit_f_eng));
    assert!(phase.data.units.contains(&a("f", "lon")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.17. TEST CASE, DISLODGEMENT CUTS SUPPORTS
/// The famous dislodge rule.
///
/// Russia:
///     F Constantinople Supports F Black Sea - Ankara
///     F Black Sea - Ankara
///
/// Turkey:
///     F Ankara - Constantinople
///     A Smyrna Supports F Ankara - Constantinople
///     A Armenia - Ankara
///
/// The Russian fleet in Constantinople is dislodged.
/// This cuts the support to from Black Sea to Ankara.
/// Black Sea will bounce with the army from Armenia.
#[test]
fn test_datc_6_d_17() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let mut unit_r_con = f("r", "con");
    let unit_r_bla = f("r", "bla");
    let unit_t_ank = f("t", "ank");
    let unit_t_smy = a("t", "smy");
    let unit_t_arm = a("t", "arm");
    phase.data.units.push(unit_r_con);
    phase.data.units.push(unit_r_bla);
    phase.data.units.push(unit_t_ank);
    phase.data.units.push(unit_t_smy);
    phase.data.units.push(unit_t_arm);
    phase.data.orders.push(unit_r_con.support_move(unit_r_bla, p("ank")));
    phase.data.orders.push(unit_r_bla.move_to(p("ank")));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_smy.support_move(unit_t_ank, p("con")));
    phase.data.orders.push(unit_t_arm.move_to(p("ank")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_r_con.set_dislodged_from(Some(p("ank")))));
    assert!(phase.data.units.contains(&unit_r_bla));
    assert!(phase.data.units.contains(&f("t", "con")));
    assert!(phase.data.units.contains(&unit_t_smy));
    assert!(phase.data.units.contains(&unit_t_arm));
    assert!(phase.data.standoff_codes.contains(&"ank".to_string()));
}

/// 6.D.18. TEST CASE, A SURVIVING UNIT WILL SUSTAIN SUPPORT
/// Idem. But now with an additional hold that prevents dislodgement.
///
/// Russia:
///     F Constantinople Supports F Black Sea - Ankara
///     F Black Sea - Ankara
///     A Bulgaria Supports F Constantinople
///
/// Turkey:
///     F Ankara - Constantinople
///     A Smyrna Supports F Ankara - Constantinople
///     A Armenia - Ankara
///
/// The Russian fleet in the Black Sea will dislodge the Turkish fleet in Ankara.
#[test]
fn test_datc_6_d_18() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_r_con = f("r", "con");
    let unit_r_bla = f("r", "bla");
    let unit_r_bul = a("r", "bul");
    let mut unit_t_ank = f("t", "ank");
    let unit_t_smy = a("t", "smy");
    let unit_t_arm = a("t", "arm");
    phase.data.units.push(unit_r_con);
    phase.data.units.push(unit_r_bla);
    phase.data.units.push(unit_r_bul);
    phase.data.units.push(unit_t_ank);
    phase.data.units.push(unit_t_smy);
    phase.data.units.push(unit_t_arm);
    phase.data.orders.push(unit_r_con.support_move(unit_r_bla, p("ank")));
    phase.data.orders.push(unit_r_bla.move_to(p("ank")));
    phase.data.orders.push(unit_r_bul.support_hold(unit_r_con));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_smy.support_move(unit_t_ank, p("con")));
    phase.data.orders.push(unit_t_arm.move_to(p("ank")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 6);
    assert!(phase.data.units.contains(&unit_r_con));
    assert!(phase.data.units.contains(&f("r", "ank")));
    assert!(phase.data.units.contains(&unit_r_bul));
    assert!(phase.data.units.contains(&unit_t_ank.set_dislodged_from(Some(p("bla")))));
    assert!(phase.data.units.contains(&unit_t_smy));
    assert!(phase.data.units.contains(&unit_t_arm));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.19. TEST CASE, EVEN WHEN SURVIVING IS IN ALTERNATIVE WAY
/// Now, the dislodgement is prevented because the support comes from a Russian army:
///
/// Russia:
///     F Constantinople Supports F Black Sea - Ankara
///     F Black Sea - Ankara
///     A Smyrna Supports F Ankara - Constantinople
///
/// Turkey:
///     F Ankara - Constantinople
///
/// The Russian fleet in Constantinople is not dislodged,
/// because one of the supports is of Russian origin.
/// The support from Black Sea to Ankara will sustain and the fleet in Ankara will be dislodged.
#[test]
fn test_datc_6_d_19() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_r_con = f("r", "con");
    let unit_r_bla = f("r", "bla");
    let unit_r_smy = a("r", "smy");
    let mut unit_t_ank = f("t", "ank");
    phase.data.units.push(unit_r_con);
    phase.data.units.push(unit_r_bla);
    phase.data.units.push(unit_r_smy);
    phase.data.units.push(unit_t_ank);
    phase.data.orders.push(unit_r_con.support_move(unit_r_bla, p("ank")));
    phase.data.orders.push(unit_r_bla.move_to(p("ank")));
    phase.data.orders.push(unit_r_smy.support_move(unit_t_ank, p("con")));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_r_con));
    assert!(phase.data.units.contains(&f("r", "ank")));
    assert!(phase.data.units.contains(&unit_r_smy));
    assert!(phase.data.units.contains(&unit_t_ank.set_dislodged_from(Some(p("bla")))));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.20. TEST CASE, UNIT CANNOT CUT SUPPORT OF ITS OWN COUNTRY
/// Although this is not mentioned in all rulebooks,
/// it is generally accepted that when a unit attacks another unit of the same Great Power,
/// it will not cut support.
///
/// England:
///     F London Supports F North Sea - English Channel
///     F North Sea - English Channel
///     A Yorkshire - London
///
/// France:
///     F English Channel Hold
///
/// The army in York does not cut support.
/// This means that the fleet in the English Channel is dislodged by the fleet in the North Sea.
#[test]
fn test_datc_6_d_20() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_lon = f("e", "lon");
    let unit_e_nth = f("e", "nth");
    let unit_e_yor = a("e", "yor");
    let mut unit_f_eng = f("f", "eng");
    phase.data.units.push(unit_e_lon);
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_f_eng);
    phase.data.orders.push(unit_e_lon.support_move(unit_e_nth, p("eng")));
    phase.data.orders.push(unit_e_nth.move_to(p("eng")));
    phase.data.orders.push(unit_e_yor.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.hold());
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_e_lon));
    assert!(phase.data.units.contains(&f("e", "eng")));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_f_eng.set_dislodged_from(Some(p("nth")))));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.21. TEST CASE, DISLODGING DOES NOT CANCEL A SUPPORT CUT
/// Sometimes there is the question whether a dislodged moving unit does not cut support
/// (similar to the dislodge rule).
/// This is not the case.
///
/// Austria:
///     F Trieste Hold
///
/// Italy:
///     A Venice - Trieste
///     A Tyrolia Supports A Venice - Trieste
///
/// Germany:
///     A Munich - Tyrolia
///
/// Russia:
///     A Silesia - Munich
///     A Berlin Supports A Silesia - Munich
///
/// Although the German army is dislodged, it still cuts the Italian support.
/// That means that the Austrian Fleet is not dislodged.
#[test]
fn test_datc_6_d_21() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_tri = f("a", "tri");
    let unit_i_ven = a("i", "ven");
    let unit_i_tyr = a("i", "tyr");
    let mut unit_g_mun = a("g", "mun");
    let unit_r_sil = a("r", "sil");
    let unit_r_ber = a("r", "ber");
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_i_ven);
    phase.data.units.push(unit_i_tyr);
    phase.data.units.push(unit_g_mun);
    phase.data.units.push(unit_r_sil);
    phase.data.units.push(unit_r_ber);
    phase.data.orders.push(unit_a_tri.hold());
    phase.data.orders.push(unit_i_ven.move_to(p("tri")));
    phase.data.orders.push(unit_i_tyr.support_move(unit_i_ven, p("tri")));
    phase.data.orders.push(unit_g_mun.move_to(p("tyr")));
    phase.data.orders.push(unit_r_sil.move_to(p("mun")));
    phase.data.orders.push(unit_r_ber.support_move(unit_r_sil, p("mun")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 6);
    assert!(phase.data.units.contains(&unit_a_tri));
    assert!(phase.data.units.contains(&unit_i_ven));
    assert!(phase.data.units.contains(&unit_i_tyr));
    assert!(phase.data.units.contains(&unit_g_mun.set_dislodged_from(Some(p("sil")))));
    assert!(phase.data.units.contains(&a("r", "mun")));
    assert!(phase.data.units.contains(&unit_r_ber));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.22. TEST CASE, IMPOSSIBLE FLEET MOVE CANNOT BE SUPPORTED
/// If a fleet tries moves to a land area it seems pointless to support the fleet,
/// since the move will fail anyway. However, in such case,
/// the support is also invalid for defense purposes.
///
/// Germany:
///     F Kiel - Munich
///     A Burgundy Supports F Kiel - Munich
///
/// Russia:
///     A Munich - Kiel
///     A Berlin Supports A Munich - Kiel
///
/// The German move from Kiel to Munich is illegal (fleets cannot go to Munich).
/// Illegal orders are fully ignored which makes the support from Burgundy also illegal.
/// The Russian army in Munich will dislodge the fleet in Kiel.
#[test]
fn test_datc_6_d_22() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let mut unit_g_kie = f("g", "kie");
    let unit_g_bur = a("g", "bur");
    let unit_r_mun = a("r", "mun");
    let unit_r_ber = a("r", "ber");
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_g_bur);
    phase.data.units.push(unit_r_mun);
    phase.data.units.push(unit_r_ber);
    phase.data.orders.push(unit_g_kie.move_to(p("mun")));
    phase.data.orders.push(unit_g_bur.support_move(unit_g_kie, p("mun")));
    phase.data.orders.push(unit_r_mun.move_to(p("kie")));
    phase.data.orders.push(unit_r_ber.support_move(unit_r_mun, p("kie")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_g_kie.set_dislodged_from(Some(p("mun")))));
    assert!(phase.data.units.contains(&unit_g_bur));
    assert!(phase.data.units.contains(&a("r", "kie")));
    assert!(phase.data.units.contains(&unit_r_ber));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.23. TEST CASE, IMPOSSIBLE COAST MOVE CANNOT BE SUPPORTED
/// Comparable with the previous test case, but now the fleet move is impossible for coastal reasons.
///
/// Italy:
///     F Gulf of Lyon - Spain(sc)
///     F Western Mediterranean Supports F Gulf of Lyon - Spain(sc)
///
/// France:
///     F Spain(nc) - Gulf of Lyon
///     F Marseilles Supports F Spain(nc) - Gulf of Lyon
///
/// The French move from Spain North Coast to Gulf of Lyon is illegal (wrong coast).
/// Therefore, the support from Marseilles fails and the fleet in Spain is dislodged.
#[test]
fn test_datc_6_d_23() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_i_gol = f("i", "gol");
    let unit_i_wes = f("i", "wes");
    let mut unit_f_spa_nc = f("f", "spa_nc");
    let unit_f_mar = f("f", "mar");
    phase.data.units.push(unit_i_gol);
    phase.data.units.push(unit_i_wes);
    phase.data.units.push(unit_f_spa_nc);
    phase.data.units.push(unit_f_mar);
    phase.data.orders.push(unit_i_gol.move_to(p("spa_sc")));
    phase.data.orders.push(unit_i_wes.support_move(unit_i_gol, p("spa_sc")));
    phase.data.orders.push(unit_f_spa_nc.move_to(p("gol")));
    phase.data.orders.push(unit_f_mar.support_move(unit_f_spa_nc, p("gol")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&f("i", "spa_sc")));
    assert!(phase.data.units.contains(&unit_i_wes));
    assert!(phase.data.units.contains(&unit_f_spa_nc.set_dislodged_from(Some(p("gol")))));
    assert!(phase.data.units.contains(&unit_f_mar));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.24. TEST CASE, IMPOSSIBLE ARMY MOVE CANNOT BE SUPPORTED
/// Comparable with the previous test case,
/// but now an army tries to move into sea and the support is used in a beleaguered garrison.
///
/// France:
///     A Marseilles - Gulf of Lyon
///     F Spain(sc) Supports A Marseilles - Gulf of Lyon
///
/// Italy:
///     F Gulf of Lyon Hold
///
/// Turkey:
///     F Tyrrhenian Sea Supports F Western Mediterranean - Gulf of Lyon
///     F Western Mediterranean - Gulf of Lyon
///
/// The French move from Marseilles to Gulf of Lyon is illegal (an army cannot go to sea).
/// Therefore, the support from Spain fails and there is no beleaguered garrison.
/// The fleet in the Gulf of Lyon is dislodged by the Turkish fleet in the Western Mediterranean.
#[test]
fn test_datc_6_d_24() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_mar = a("f", "mar");
    let unit_f_spa_sc = f("f", "spa_sc");
    let mut unit_i_gol = f("i", "gol");
    let unit_t_tyr = f("t", "tyn");
    let unit_t_wes = f("t", "wes");
    phase.data.units.push(unit_f_mar);
    phase.data.units.push(unit_f_spa_sc);
    phase.data.units.push(unit_i_gol);
    phase.data.units.push(unit_t_tyr);
    phase.data.units.push(unit_t_wes);
    phase.data.orders.push(unit_f_mar.move_to(p("gol")));
    phase.data.orders.push(unit_f_spa_sc.support_move(unit_f_mar, p("gol")));
    phase.data.orders.push(unit_i_gol.hold());
    phase.data.orders.push(unit_t_tyr.support_move(unit_t_wes, p("gol")));
    phase.data.orders.push(unit_t_wes.move_to(p("gol")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_f_mar));
    assert!(phase.data.units.contains(&unit_f_spa_sc));
    assert!(phase.data.units.contains(&unit_i_gol.set_dislodged_from(Some(p("wes")))));
    assert!(phase.data.units.contains(&unit_t_tyr));
    assert!(phase.data.units.contains(&f("t", "gol")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.25. TEST CASE, FAILING HOLD SUPPORT CAN BE SUPPORTED
/// If an adjudicator fails on one of the previous three test cases,
/// then the bug should be removed with care.
/// A failing move cannot be supported, but a failing hold support,
/// because of some preconditions (unmatching order) can still be supported.
///
/// Germany:
///     A Berlin Supports A Prussia
///     F Kiel Supports A Berlin
///
/// Russia:
///     F Baltic Sea Supports A Prussia - Berlin
///     A Prussia - Berlin
///
/// Although the support of Berlin on Prussia fails (because of unmatching orders),
/// the support of Kiel on Berlin is still valid. So, Berlin will not be dislodged.
#[test]
fn test_datc_6_d_25() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_r_bal = f("r", "bal");
    let unit_r_pru = a("r", "pru");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_r_bal);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_g_ber.support_hold(unit_r_pru));
    phase.data.orders.push(unit_g_kie.support_hold(unit_g_ber));
    phase.data.orders.push(unit_r_bal.support_move(unit_r_pru, p("ber")));
    phase.data.orders.push(unit_r_pru.move_to(p("ber")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&unit_r_bal));
    assert!(phase.data.units.contains(&unit_r_pru));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.26. TEST CASE, FAILING MOVE SUPPORT CAN BE SUPPORTED
/// Similar as the previous test case, but now with an unmatched support to move.
///
/// Germany:
///     A Berlin Supports A Prussia - Silesia
///     F Kiel Supports A Berlin
///
/// Russia:
///     F Baltic Sea Supports A Prussia - Berlin
///     A Prussia - Berlin
///
/// Again, Berlin will not be dislodged.
#[test]
fn test_datc_6_d_26() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_r_bal = f("r", "bal");
    let unit_r_pru = a("r", "pru");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_r_bal);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_g_ber.support_move(unit_r_pru, p("sil")));
    phase.data.orders.push(unit_g_kie.support_hold(unit_g_ber));
    phase.data.orders.push(unit_r_bal.support_move(unit_r_pru, p("ber")));
    phase.data.orders.push(unit_r_pru.move_to(p("ber")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&unit_r_bal));
    assert!(phase.data.units.contains(&unit_r_pru));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.27. TEST CASE, FAILING CONVOY CAN BE SUPPORTED
/// Similar as the previous test case, but now with an unmatched convoy.
///
/// England:
///     F Sweden - Baltic Sea
///     F Denmark Supports F Sweden - Baltic Sea
///
/// Germany:
///     A Berlin Hold
///
/// Russia:
///     F Baltic Sea Convoys A Berlin - Livonia
///     F Prussia Supports F Baltic Sea
///
/// The convoy order in the Baltic Sea is unmatched and fails.
/// However, the support of Prussia on the Baltic Sea is still valid
/// and the fleet in the Baltic Sea is not dislodged.
#[test]
fn test_datc_6_d_27() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_swe = f("e", "swe");
    let unit_e_den = f("e", "den");
    let unit_g_ber = a("g", "ber");
    let unit_r_bal = f("r", "bal");
    let unit_r_pru = f("r", "pru");
    phase.data.units.push(unit_e_swe);
    phase.data.units.push(unit_e_den);
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_r_bal);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_e_swe.move_to(p("bal")));
    phase.data.orders.push(unit_e_den.support_move(unit_e_swe, p("bal")));
    phase.data.orders.push(unit_g_ber.hold());
    phase.data.orders.push(unit_r_bal.convoy(unit_g_ber, p("lvn")));
    phase.data.orders.push(unit_r_pru.support_hold(unit_r_bal));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_e_swe));
    assert!(phase.data.units.contains(&unit_e_den));
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_r_bal));
    assert!(phase.data.units.contains(&unit_r_pru));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.28. TEST CASE, IMPOSSIBLE MOVE AND SUPPORT
/// An impossible move is "illegal" and should be ignored.
///
/// Austria:
///     A Budapest Supports F Rumania
///
/// Russia:
///     F Rumania - Holland
///
/// Turkey:
///     F Black Sea - Rumania
///     A Bulgaria Supports F Black Sea - Rumania
///
/// See issue 4.E.1. Illegal orders are ignored. Without an order,
/// Rumania holds and receives support.
/// The fleet in Rumania is not dislodged.
#[allow(unused)]
fn test_datc_6_d_28() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.D.29. TEST CASE, MOVE TO IMPOSSIBLE COAST AND SUPPORT
/// Similar to the previous test case, but now the move "illegal" due the wrong coast.
///
/// Austria:
///     A Budapest Supports F Rumania
///
/// Russia:
///     F Rumania - Bulgaria(sc)
///
/// Turkey:
///     F Black Sea - Rumania
///     A Bulgaria Supports F Black Sea - Rumania
///
/// See issue 4.E.1. Illegal orders are ignored. Without an order,
/// Rumania holds and receives support.
/// The fleet in Rumania is not dislodged.
#[allow(unused)]
fn test_datc_6_d_29() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.D.30. TEST CASE, MOVE WITHOUT COAST AND SUPPORT
/// Similar to the previous test case, but now the move is "illegal" due to missing coast.
///
/// Italy:
///     F Aegean Sea Supports F Constantinople
///
/// Russia:
///     F Constantinople - Bulgaria
///
/// Turkey:
///     F Black Sea - Constantinople
///     A Bulgaria Supports F Black Sea - Constantinople
///
/// See issue 4.E.1. Illegal orders are ignored. Without an order,
/// Constantinople holds and receives support.
/// The fleet in Constantinople is not dislodged.
#[allow(unused)]
fn test_datc_6_d_30() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.D.31. TEST CASE, A TRICKY IMPOSSIBLE SUPPORT
/// A support order can be impossible for complex reasons.
///
/// Austria:
///     A Rumania - Armenia
///
/// Turkey:
///     F Black Sea Supports A Rumania - Armenia
///
/// Although the army in Rumania can move to Armenia
/// and the fleet in the Black Sea can also go to Armenia, the support is still not possible.
/// The reason is that the only possible convoy is through the Black Sea
/// and a fleet cannot convoy and support at the same time.
/// This is relevant for computer programs that show only the possible orders.
/// In the list of possible orders,
/// the support as given to the fleet in the Black Sea, should not be listed.
/// Furthermore, the support order should be judged to be illegal,
/// meaning that it is completely ignored.
/// If there is a second order for the Black Sea, that order should be executed (see issue 4.E.1).
#[test]
fn test_datc_6_d_31() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_rum = a("a", "rum");
    let unit_t_bla = f("t", "bla");
    phase.data.units.push(unit_a_rum);
    phase.data.units.push(unit_t_bla);
    phase.data.orders.push(unit_a_rum.move_to(p("arm")));
    phase.data.orders.push(unit_t_bla.support_move(unit_a_rum, p("arm")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 2);
    assert!(phase.data.units.contains(&unit_a_rum));
    assert!(phase.data.units.contains(&unit_t_bla));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.32. TEST CASE, A MISSING FLEET
/// The previous test cases contained an order that was impossible
/// even when some other pieces on the board where changed.
/// In this test case, the order is impossible, but only for that situation.
///
/// England:
///     F Edinburgh Supports A Liverpool - Yorkshire
///     A Liverpool - Yorkshire
///
/// France:
///     F London Supports A Yorkshire
///
/// Germany:
///     A Yorkshire - Holland
///
/// The German order to Yorkshire cannot be executed,
/// because there is no fleet in the North Sea.
/// In other situations (where there is a fleet in the North Sea),
/// the exact same order would be possible.
/// This is considered "illegal" (see issue 4.E.1).
/// The order should be ignored and the support of the French fleet in London succeeds.
/// This means that the army in Yorkshire is not dislodged.
#[allow(unused)]
fn test_datc_6_d_32() {
    // 不適切命令の救済に関するテスト（対応予定なし）
}

/// 6.D.33. TEST CASE, UNWANTED SUPPORT ALLOWED
/// A self standoff can be broken by an unwanted support.
///
/// Austria:
///     A Serbia - Budapest
///     A Vienna - Budapest
///
/// Russia:
///     A Galicia Supports A Serbia - Budapest
///
/// Turkey:
///     A Bulgaria - Serbia
///
/// Due to the Russian support, the army in Serbia advances to Budapest.
/// This enables Turkey to capture Serbia with the army in Bulgaria.
#[test]
fn test_datc_6_d_33() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_ser = a("a", "ser");
    let unit_a_vie = a("a", "vie");
    let unit_r_gal = a("r", "gal");
    let unit_t_bul = a("t", "bul");
    phase.data.units.push(unit_a_ser);
    phase.data.units.push(unit_a_vie);
    phase.data.units.push(unit_r_gal);
    phase.data.units.push(unit_t_bul);
    phase.data.orders.push(unit_a_ser.move_to(p("bud")));
    phase.data.orders.push(unit_a_vie.move_to(p("bud")));
    phase.data.orders.push(unit_r_gal.support_move(unit_a_ser, p("bud")));
    phase.data.orders.push(unit_t_bul.move_to(p("ser")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&a("a", "bud")));
    assert!(phase.data.units.contains(&unit_a_vie));
    assert!(phase.data.units.contains(&unit_r_gal));
    assert!(phase.data.units.contains(&a("t", "ser")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.D.34. TEST CASE, SUPPORT TARGETING OWN AREA NOT ALLOWED
/// Support targeting the area where the supporting unit is standing, is illegal.
///
/// Germany:
///     A Berlin - Prussia
///     A Silesia Supports A Berlin - Prussia
///     F Baltic Sea Supports A Berlin - Prussia
///
/// Italy:
///     A Prussia Supports Livonia - Prussia
///
/// Russia:
///     A Warsaw Supports A Livonia - Prussia
///     A Livonia - Prussia
///
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
fn test_datc_6_d_34() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_sil = a("g", "sil");
    let unit_g_bal = f("g", "bal");
    let mut unit_i_pru = a("i", "pru");
    let unit_r_war = a("r", "war");
    let unit_r_lvn = a("r", "lvn");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_sil);
    phase.data.units.push(unit_g_bal);
    phase.data.units.push(unit_i_pru);
    phase.data.units.push(unit_r_war);
    phase.data.units.push(unit_r_lvn);
    phase.data.orders.push(unit_g_ber.move_to(p("pru")));
    phase.data.orders.push(unit_g_sil.support_move(unit_g_ber, p("pru")));
    phase.data.orders.push(unit_g_bal.support_move(unit_g_ber, p("pru")));
    phase.data.orders.push(unit_i_pru.support_move(unit_i_pru, p("pru")));
    phase.data.orders.push(unit_r_war.support_move(unit_r_lvn, p("pru")));
    phase.data.orders.push(unit_r_lvn.move_to(p("pru")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 6);
    assert!(phase.data.units.contains(&a("g", "pru")));
    assert!(phase.data.units.contains(&unit_g_sil));
    assert!(phase.data.units.contains(&unit_g_bal));
    assert!(phase.data.units.contains(&unit_i_pru.set_dislodged_from(Some(p("ber")))));
    assert!(phase.data.units.contains(&unit_r_war));
    assert!(phase.data.units.contains(&unit_r_lvn));
    assert!(phase.data.standoff_codes.is_empty());
}
