//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6E]
//!
//! * 6.E. TEST CASES, HEAD-TO-HEAD BATTLES AND BELEAGUERED GARRISON
//!
//! [DATC_6E]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.E

use crate::domain::models::order::*;
use crate::domain::models::phase::*;
use crate::domain::tests::a;
use crate::domain::tests::f;
use crate::domain::tests::p;

/// 6.E.1. TEST CASE, DISLODGED UNIT HAS NO EFFECT ON ATTACKER'S AREA
/// An army can follow.
///
/// Germany:
///     A Berlin - Prussia
///     F Kiel - Berlin
///     A Silesia Supports A Berlin - Prussia
///
/// Russia:
///     A Prussia - Berlin
///
/// The army in Kiel will move to Berlin.
#[test]
fn test_datc_6_e_1() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_g_sil = a("g", "sil");
    let mut unit_r_pru = a("r", "pru");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_g_sil);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_g_ber.move_to(p("pru")));
    phase.data.orders.push(unit_g_kie.move_to(p("ber")));
    phase.data.orders.push(unit_g_sil.support_move(unit_g_ber, p("pru")));
    phase.data.orders.push(unit_r_pru.move_to(p("ber")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&a("g", "pru")));
    assert!(phase.data.units.contains(&f("g", "ber")));
    assert!(phase.data.units.contains(&unit_g_sil));
    assert!(phase.data.units.contains(&unit_r_pru.set_dislodged_from(Some(p("ber")))));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.E.2. TEST CASE, NO SELF DISLODGEMENT IN HEAD-TO-HEAD BATTLE
/// Self dislodgement is not allowed. This also counts for head-to-head battles.
///
/// Germany:
///     A Berlin - Kiel
///     F Kiel - Berlin
///     A Munich Supports A Berlin - Kiel
///
/// No unit will move.
#[test]
fn test_datc_6_e_2() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_kie = f("g", "kie");
    let unit_g_mun = a("g", "mun");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_kie);
    phase.data.units.push(unit_g_mun);
    phase.data.orders.push(unit_g_ber.move_to(p("kie")));
    phase.data.orders.push(unit_g_kie.move_to(p("ber")));
    phase.data.orders.push(unit_g_mun.support_move(unit_g_ber, p("kie")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_kie));
    assert!(phase.data.units.contains(&unit_g_mun));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.E.3. TEST CASE, NO HELP IN DISLODGING OWN UNIT
/// It is not possible to help a foreign power dislodge own unit in a head-to-head battle.
///
/// Germany:
///     A Berlin - Kiel
///     A Munich Supports F Kiel - Berlin
///
/// England:
///     F Kiel - Berlin
///
/// No unit will move.
#[test]
fn test_datc_6_e_3() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_ber = a("g", "ber");
    let unit_g_mun = a("g", "mun");
    let unit_e_kie = f("e", "kie");
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_mun);
    phase.data.units.push(unit_e_kie);
    phase.data.orders.push(unit_g_ber.move_to(p("kie")));
    phase.data.orders.push(unit_g_mun.support_move(unit_e_kie, p("ber")));
    phase.data.orders.push(unit_e_kie.move_to(p("ber")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_mun));
    assert!(phase.data.units.contains(&unit_e_kie));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.E.4. TEST CASE, NON-DISLODGED LOSER STILL HAS EFFECT
/// If in an unbalanced head-to-head battle the loser is not dislodged,
/// it still has an effect on the area of the attacker.
///
/// Germany:
///     F Holland - North Sea
///     F Helgoland Bight Supports F Holland - North Sea
///     F Skagerrak Supports F Holland - North Sea
///
/// France:
///     F North Sea - Holland
///     F Belgium Supports F North Sea - Holland
///
/// England:
///     F Edinburgh Supports F Norwegian Sea - North Sea
///     F Yorkshire Supports F Norwegian Sea - North Sea
///     F Norwegian Sea - North Sea
///
/// Austria:
///     A Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// The French fleet in the North Sea is not dislodged due to the beleaguered garrison.
/// Therefore, the Austrian army in Ruhr will not move to Holland.
#[test]
fn test_datc_6_e_4() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_g_ska = f("g", "ska");
    let unit_f_nth = f("f", "nth");
    let unit_f_bel = f("f", "bel");
    let unit_e_edi = f("e", "edi");
    let unit_e_yor = f("e", "yor");
    let unit_e_nrg = f("e", "nrg");
    let unit_a_kie = a("a", "kie");
    let unit_a_ruh = a("a", "ruh");
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_g_ska);
    phase.data.units.push(unit_f_nth);
    phase.data.units.push(unit_f_bel);
    phase.data.units.push(unit_e_edi);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_e_nrg);
    phase.data.units.push(unit_a_kie);
    phase.data.units.push(unit_a_ruh);
    phase.data.orders.push(unit_g_hol.move_to(p("nth")));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_hol, p("nth")));
    phase.data.orders.push(unit_g_ska.support_move(unit_g_hol, p("nth")));
    phase.data.orders.push(unit_f_nth.move_to(p("hol")));
    phase.data.orders.push(unit_f_bel.support_move(unit_f_nth, p("hol")));
    phase.data.orders.push(unit_e_edi.support_move(unit_e_nrg, p("nth")));
    phase.data.orders.push(unit_e_yor.support_move(unit_e_nrg, p("nth")));
    phase.data.orders.push(unit_e_nrg.move_to(p("nth")));
    phase.data.orders.push(unit_a_kie.support_move(unit_a_ruh, p("hol")));
    phase.data.orders.push(unit_a_ruh.move_to(p("hol")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[8].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[9].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 10);
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_g_ska));
    assert!(phase.data.units.contains(&unit_f_nth));
    assert!(phase.data.units.contains(&unit_f_bel));
    assert!(phase.data.units.contains(&unit_e_edi));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_e_nrg));
    assert!(phase.data.units.contains(&unit_a_kie));
    assert!(phase.data.units.contains(&unit_a_ruh));
    assert!(phase.data.standoff_codes.contains(&"nth".to_string()));
}

/// 6.E.5. TEST CASE, LOSER DISLODGED BY ANOTHER ARMY STILL HAS EFFECT
/// If in an unbalanced head-to-head battle the loser is dislodged by a unit not part of the head-to-head battle,
/// the loser still has an effect on the area of the winner of the head-to-head battle.
///
/// Germany:
///     F Holland - North Sea
///     F Helgoland Bight Supports F Holland - North Sea
///     F Skagerrak Supports F Holland - North Sea
///
/// France:
///     F North Sea - Holland
///     F Belgium Supports F North Sea - Holland
///
/// England:
///     F Edinburgh Supports F Norwegian Sea - North Sea
///     F Yorkshire Supports F Norwegian Sea - North Sea
///     F Norwegian Sea - North Sea
///     F London Supports F Norwegian Sea - North Sea
///
/// Austria:
///     A Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// The French fleet in the North Sea is dislodged but not by the German fleet in Holland.
/// Therefore, the French fleet can still prevent that the Austrian army in Ruhr will move to Holland.
/// So, the Austrian move in Ruhr fails and the German fleet in Holland is not dislodged.
#[test]
fn test_datc_6_e_5() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_g_ska = f("g", "ska");
    let mut unit_f_nth = f("f", "nth");
    let unit_f_bel = f("f", "bel");
    let unit_e_edi = f("e", "edi");
    let unit_e_yor = f("e", "yor");
    let unit_e_nrg = f("e", "nrg");
    let unit_e_lon = f("e", "lon");
    let unit_a_kie = a("a", "kie");
    let unit_a_ruh = a("a", "ruh");
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_g_ska);
    phase.data.units.push(unit_f_nth);
    phase.data.units.push(unit_f_bel);
    phase.data.units.push(unit_e_edi);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_e_nrg);
    phase.data.units.push(unit_e_lon);
    phase.data.units.push(unit_a_kie);
    phase.data.units.push(unit_a_ruh);
    phase.data.orders.push(unit_g_hol.move_to(p("nth")));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_hol, p("nth")));
    phase.data.orders.push(unit_g_ska.support_move(unit_g_hol, p("nth")));
    phase.data.orders.push(unit_f_nth.move_to(p("hol")));
    phase.data.orders.push(unit_f_bel.support_move(unit_f_nth, p("hol")));
    phase.data.orders.push(unit_e_edi.support_move(unit_e_nrg, p("nth")));
    phase.data.orders.push(unit_e_yor.support_move(unit_e_nrg, p("nth")));
    phase.data.orders.push(unit_e_nrg.move_to(p("nth")));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_nrg, p("nth")));
    phase.data.orders.push(unit_a_kie.support_move(unit_a_ruh, p("hol")));
    phase.data.orders.push(unit_a_ruh.move_to(p("hol")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[8].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[9].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[10].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 11);
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_g_ska));
    assert!(phase.data.units.contains(&unit_f_nth.set_dislodged_from(Some(p("nrg")))));
    assert!(phase.data.units.contains(&unit_f_bel));
    assert!(phase.data.units.contains(&unit_e_edi));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&f("e", "nth")));
    assert!(phase.data.units.contains(&unit_e_lon));
    assert!(phase.data.units.contains(&unit_a_kie));
    assert!(phase.data.units.contains(&unit_a_ruh));
    assert!(phase.data.standoff_codes.contains(&"hol".to_string()));
}

/// 6.E.6. TEST CASE, NOT DISLODGE BECAUSE OF OWN SUPPORT STILL HAS EFFECT
/// If in an unbalanced head-to-head battle the loser is not dislodged
/// because the winner had help of a unit of the loser,
/// the loser still has an effect on the area of the winner.
///
/// Germany:
///     F Holland - North Sea
///     F Helgoland Bight Supports F Holland - North Sea
///
/// France:
///     F North Sea - Holland
///     F Belgium Supports F North Sea - Holland
///     F English Channel Supports F Holland - North Sea
///
/// Austria:
///     A Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// Although the German force from Holland to North Sea is one larger
/// than the French force from North Sea to Holland,
/// the French fleet in the North Sea is not dislodged,
/// because one of the supports on the German movement is French.
/// Therefore, the Austrian army in Ruhr will not move to Holland.
#[test]
fn test_datc_6_e_6() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_f_nth = f("f", "nth");
    let unit_f_bel = f("f", "bel");
    let unit_f_eng = f("f", "eng");
    let unit_a_kie = a("a", "kie");
    let unit_a_ruh = a("a", "ruh");
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_f_nth);
    phase.data.units.push(unit_f_bel);
    phase.data.units.push(unit_f_eng);
    phase.data.units.push(unit_a_kie);
    phase.data.units.push(unit_a_ruh);
    phase.data.orders.push(unit_g_hol.move_to(p("nth")));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_hol, p("nth")));
    phase.data.orders.push(unit_f_nth.move_to(p("hol")));
    phase.data.orders.push(unit_f_bel.support_move(unit_f_nth, p("hol")));
    phase.data.orders.push(unit_f_eng.support_move(unit_g_hol, p("nth")));
    phase.data.orders.push(unit_a_kie.support_move(unit_a_ruh, p("hol")));
    phase.data.orders.push(unit_a_ruh.move_to(p("hol")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 7);
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_f_nth));
    assert!(phase.data.units.contains(&unit_f_bel));
    assert!(phase.data.units.contains(&unit_f_eng));
    assert!(phase.data.units.contains(&unit_a_kie));
    assert!(phase.data.units.contains(&unit_a_ruh));
    assert!(phase.data.standoff_codes.contains(&"hol".to_string()));
}

/// 6.E.7. TEST CASE, NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON
/// An attempt at self dislodgement can be combined with a beleaguered garrison.
/// Such self dislodgment is still not possible.
///
/// England:
///     F North Sea Hold
///     F Yorkshire Supports F Norway - North Sea
///
/// Germany:
///     F Holland Supports F Helgoland Bight - North Sea
///     F Helgoland Bight - North Sea
///
/// Russia:
///     F Skagerrak Supports F Norway - North Sea
///     F Norway - North Sea
///
/// Although the Russians beat the German attack (with the support of Yorkshire)
/// and the two Russian fleets are enough to dislodge the fleet in the North Sea,
/// the fleet in the North Sea is not dislodged,
/// since it would not be dislodged if the English fleet in Yorkshire would not give support.
/// This is a typical bug that can happen if a grand winner is calculated of a contested area
/// (instead of calculating every move separately).
/// Of the contested area the North Sea, the Russians are the grand winner with a strength of three,
/// but this doesn't mean that they can advance.
#[test]
fn test_datc_6_e_7() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let unit_e_yor = f("e", "yor");
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_r_ska = f("r", "ska");
    let unit_r_nwy = f("r", "nwy");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_r_ska);
    phase.data.units.push(unit_r_nwy);
    phase.data.orders.push(unit_e_nth.hold());
    phase.data.orders.push(unit_e_yor.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_g_hol.support_move(unit_g_hel, p("nth")));
    phase.data.orders.push(unit_g_hel.move_to(p("nth")));
    phase.data.orders.push(unit_r_ska.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_r_nwy.move_to(p("nth")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 6);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_r_ska));
    assert!(phase.data.units.contains(&unit_r_nwy));
    assert!(phase.data.standoff_codes.contains(&"nth".to_string()));
}

/// 6.E.8. TEST CASE, NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON AND HEAD-TO-HEAD BATTLE
/// Similar to the previous test case, but now the beleaguered fleet is also engaged in a head-to-head battle.
///
/// England:
///     F North Sea - Norway
///     F Yorkshire Supports F Norway - North Sea
///
/// Germany:
///     F Holland Supports F Helgoland Bight - North Sea
///     F Helgoland Bight - North Sea
///
/// Russia:
///     F Skagerrak Supports F Norway - North Sea
///     F Norway - North Sea
///
/// Again, none of the fleets move.
#[test]
fn test_datc_6_e_8() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let unit_e_yor = f("e", "yor");
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_r_ska = f("r", "ska");
    let unit_r_nwy = f("r", "nwy");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_r_ska);
    phase.data.units.push(unit_r_nwy);
    phase.data.orders.push(unit_e_nth.move_to(p("nwy")));
    phase.data.orders.push(unit_e_yor.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_g_hol.support_move(unit_g_hel, p("nth")));
    phase.data.orders.push(unit_g_hel.move_to(p("nth")));
    phase.data.orders.push(unit_r_ska.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_r_nwy.move_to(p("nth")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 6);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_r_ska));
    assert!(phase.data.units.contains(&unit_r_nwy));
    assert!(phase.data.standoff_codes.contains(&"nth".to_string()));
}

/// 6.E.9. TEST CASE, ALMOST SELF DISLODGEMENT WITH BELEAGUERED GARRISON
/// Similar to the previous test case, but now the beleaguered fleet is moving away.
///
/// England:
///     F North Sea - Norwegian Sea
///     F Yorkshire Supports F Norway - North Sea
///
/// Germany:
///     F Holland Supports F Helgoland Bight - North Sea
///     F Helgoland Bight - North Sea
///
/// Russia:
///     F Skagerrak Supports F Norway - North Sea
///     F Norway - North Sea
///
/// Both the fleet in the North Sea and the fleet in Norway move.
#[test]
fn test_datc_6_e_9() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let unit_e_yor = f("e", "yor");
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_r_ska = f("r", "ska");
    let unit_r_nwy = f("r", "nwy");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_r_ska);
    phase.data.units.push(unit_r_nwy);
    phase.data.orders.push(unit_e_nth.move_to(p("nrg")));
    phase.data.orders.push(unit_e_yor.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_g_hol.support_move(unit_g_hel, p("nth")));
    phase.data.orders.push(unit_g_hel.move_to(p("nth")));
    phase.data.orders.push(unit_r_ska.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_r_nwy.move_to(p("nth")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 6);
    assert!(phase.data.units.contains(&f("e", "nrg")));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_r_ska));
    assert!(phase.data.units.contains(&f("r", "nth")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.E.10. TEST CASE, ALMOST CIRCULAR MOVEMENT WITH NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON
/// Similar to the previous test case, but now the beleaguered fleet is in circular movement with the weaker attacker.
/// So, the circular movement fails.
///
/// England:
///     F North Sea - Denmark
///     F Yorkshire Supports F Norway - North Sea
///
/// Germany:
///     F Holland Supports F Helgoland Bight - North Sea
///     F Helgoland Bight - North Sea
///     F Denmark - Helgoland Bight
///
/// Russia:
///     F Skagerrak Supports F Norway - North Sea
///     F Norway - North Sea
///
/// There is no movement of fleets.
#[test]
fn test_datc_6_e_10() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let unit_e_yor = f("e", "yor");
    let unit_g_hol = f("g", "hol");
    let unit_g_hel = f("g", "hel");
    let unit_g_den = f("g", "den");
    let unit_r_ska = f("r", "ska");
    let unit_r_nwy = f("r", "nwy");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_g_hol);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_g_den);
    phase.data.units.push(unit_r_ska);
    phase.data.units.push(unit_r_nwy);
    phase.data.orders.push(unit_e_nth.move_to(p("den")));
    phase.data.orders.push(unit_e_yor.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_g_hol.support_move(unit_g_hel, p("nth")));
    phase.data.orders.push(unit_g_hel.move_to(p("nth")));
    phase.data.orders.push(unit_g_den.move_to(p("hel")));
    phase.data.orders.push(unit_r_ska.support_move(unit_r_nwy, p("nth")));
    phase.data.orders.push(unit_r_nwy.move_to(p("nth")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 7);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_g_hol));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_g_den));
    assert!(phase.data.units.contains(&unit_r_ska));
    assert!(phase.data.units.contains(&unit_r_nwy));
    assert!(phase.data.standoff_codes.contains(&"nth".to_string()));
}

/// 6.E.11. TEST CASE, NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON,
/// UNIT SWAP WITH ADJACENT CONVOYING AND TWO COASTS
/// Similar to the previous test case, but now the beleaguered fleet is
/// in a unit swap with the stronger attacker.
/// So, the unit swap succeeds. To make the situation more complex,
/// the swap is on an area with two coasts.
///
/// France:
///     A Spain - Portugal via convoy
///     F Mid-Atlantic Ocean Convoys A Spain - Portugal
///     F Gulf of Lyon Supports F Portugal - Spain(nc)
///
/// Germany:
///     A Marseilles Supports A Gascony - Spain
///     A Gascony - Spain
///
/// Italy:
///     F Portugal - Spain(nc)
///     F Western Mediterranean Supports F Portugal - Spain(nc)
///
/// The unit swap succeeds. Note that due to the success of the swap,
/// there is no beleaguered garrison anymore.
#[test]
fn test_datc_6_e_11() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_f_spa = a("f", "spa");
    let unit_f_mid = f("f", "mid");
    let unit_f_gol = f("f", "gol");
    let unit_g_mar = a("g", "mar");
    let unit_g_gas = a("g", "gas");
    let unit_i_por = f("i", "por");
    let unit_i_wes = f("i", "wes");
    phase.data.units.push(unit_f_spa);
    phase.data.units.push(unit_f_mid);
    phase.data.units.push(unit_f_gol);
    phase.data.units.push(unit_g_mar);
    phase.data.units.push(unit_g_gas);
    phase.data.units.push(unit_i_por);
    phase.data.units.push(unit_i_wes);
    phase.data.orders.push(unit_f_spa.move_to(p("por")).set_via_convoy());
    phase.data.orders.push(unit_f_mid.convoy(unit_f_spa, p("por")));
    phase.data.orders.push(unit_f_gol.support_move(unit_i_por, p("spa_nc")));
    phase.data.orders.push(unit_g_mar.support_move(unit_g_gas, p("spa")));
    phase.data.orders.push(unit_g_gas.move_to(p("spa")));
    phase.data.orders.push(unit_i_por.move_to(p("spa_nc")));
    phase.data.orders.push(unit_i_wes.support_move(unit_i_por, p("spa_nc")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 7);
    assert!(phase.data.units.contains(&a("f", "por")));
    assert!(phase.data.units.contains(&unit_f_mid));
    assert!(phase.data.units.contains(&unit_f_gol));
    assert!(phase.data.units.contains(&unit_g_mar));
    assert!(phase.data.units.contains(&unit_g_gas));
    assert!(phase.data.units.contains(&f("i", "spa_nc")));
    assert!(phase.data.units.contains(&unit_i_wes));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.E.12. TEST CASE, SUPPORT ON ATTACK ON OWN UNIT CAN BE USED FOR OTHER MEANS
/// A support on an attack on your own unit still has an effect.
/// It can prevent that another army will dislodge the unit.
///
/// Austria:
///     A Budapest - Rumania
///     A Serbia Supports A Vienna - Budapest
///
/// Italy:
///     A Vienna - Budapest
///
/// Russia:
///     A Galicia - Budapest
///     A Rumania Supports A Galicia - Budapest
///
/// The support of Serbia on the Italian army prevents that the Russian army in Galicia will advance.
/// No army will move.
#[test]
fn test_datc_6_e_12() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_bud = a("a", "bud");
    let unit_a_ser = a("a", "ser");
    let unit_i_vie = a("i", "vie");
    let unit_r_gal = a("r", "gal");
    let unit_r_rum = a("r", "rum");
    phase.data.units.push(unit_a_bud);
    phase.data.units.push(unit_a_ser);
    phase.data.units.push(unit_i_vie);
    phase.data.units.push(unit_r_gal);
    phase.data.units.push(unit_r_rum);
    phase.data.orders.push(unit_a_bud.move_to(p("rum")));
    phase.data.orders.push(unit_a_ser.support_move(unit_i_vie, p("bud")));
    phase.data.orders.push(unit_i_vie.move_to(p("bud")));
    phase.data.orders.push(unit_r_gal.move_to(p("bud")));
    phase.data.orders.push(unit_r_rum.support_move(unit_r_gal, p("bud")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_a_bud));
    assert!(phase.data.units.contains(&unit_a_ser));
    assert!(phase.data.units.contains(&unit_i_vie));
    assert!(phase.data.units.contains(&unit_r_gal));
    assert!(phase.data.units.contains(&unit_r_rum));
    assert!(phase.data.standoff_codes.contains(&"bud".to_string()));
}

/// 6.E.13. TEST CASE, THREE WAY BELEAGUERED GARRISON
/// In a beleaguered garrison from three sides,
/// the adjudicator may not let two attacks fail and then let the third succeed.
///
/// England:
///     F Edinburgh Supports F Yorkshire - North Sea
///     F Yorkshire - North Sea
///
/// France:
///     F Belgium - North Sea
///     F English Channel Supports F Belgium - North Sea
///
/// Germany:
///     F North Sea Hold
///
/// Russia:
///     F Norwegian Sea - North Sea
///     F Norway Supports F Norwegian Sea - North Sea
///
/// None of the fleets move. The German fleet in the North Sea is not dislodged.
#[test]
fn test_datc_6_e_13() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_edi = f("e", "edi");
    let unit_e_yor = f("e", "yor");
    let unit_f_bel = f("f", "bel");
    let unit_f_eng = f("f", "eng");
    let unit_g_nth = f("g", "nth");
    let unit_r_nrg = f("r", "nrg");
    let unit_r_nwy = f("r", "nwy");
    phase.data.units.push(unit_e_edi);
    phase.data.units.push(unit_e_yor);
    phase.data.units.push(unit_f_bel);
    phase.data.units.push(unit_f_eng);
    phase.data.units.push(unit_g_nth);
    phase.data.units.push(unit_r_nrg);
    phase.data.units.push(unit_r_nwy);
    phase.data.orders.push(unit_e_edi.support_move(unit_e_yor, p("nth")));
    phase.data.orders.push(unit_e_yor.move_to(p("nth")));
    phase.data.orders.push(unit_f_bel.move_to(p("nth")));
    phase.data.orders.push(unit_f_eng.support_move(unit_f_bel, p("nth")));
    phase.data.orders.push(unit_g_nth.hold());
    phase.data.orders.push(unit_r_nrg.move_to(p("nth")));
    phase.data.orders.push(unit_r_nwy.support_move(unit_r_nrg, p("nth")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 7);
    assert!(phase.data.units.contains(&unit_e_edi));
    assert!(phase.data.units.contains(&unit_e_yor));
    assert!(phase.data.units.contains(&unit_f_bel));
    assert!(phase.data.units.contains(&unit_f_eng));
    assert!(phase.data.units.contains(&unit_g_nth));
    assert!(phase.data.units.contains(&unit_r_nrg));
    assert!(phase.data.units.contains(&unit_r_nwy));
    assert!(phase.data.standoff_codes.contains(&"nth".to_string()));
}

/// 6.E.14. TEST CASE, ILLEGAL HEAD-TO-HEAD BATTLE CAN STILL DEFEND
/// If in a head-to-head battle, one of the units makes an illegal move,
/// then that unit still has the possibility to defend against attacks with strength of one.
///
/// England:
///     A Liverpool - Edinburgh
///
/// Russia:
///     F Edinburgh - Liverpool
///
/// The move of the Russian fleet is illegal,
/// but can still prevent the English army from entering Edinburgh.
/// So, none of the units move.
#[test]
fn test_datc_6_e_14() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_lvp = a("e", "lvp");
    let unit_r_edi = f("r", "edi");
    phase.data.units.push(unit_e_lvp);
    phase.data.units.push(unit_r_edi);
    phase.data.orders.push(unit_e_lvp.move_to(p("edi")));
    phase.data.orders.push(unit_r_edi.move_to(p("lvp")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.units.len(), 2);
    assert!(phase.data.units.contains(&unit_e_lvp));
    assert!(phase.data.units.contains(&unit_r_edi));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.E.15. TEST CASE, THE FRIENDLY HEAD-TO-HEAD BATTLE
/// In this case each unit in the head-to-head battle prevents that
/// the other unit from being dislodged.
///
/// England:
///     F Holland Supports A Ruhr - Kiel
///     A Ruhr - Kiel
///
/// France:
///     A Kiel - Berlin
///     A Munich Supports A Kiel - Berlin
///     A Silesia Supports A Kiel - Berlin
///
/// Germany:
///     A Berlin - Kiel
///     F Denmark Supports A Berlin - Kiel
///     F Helgoland Bight Supports A Berlin - Kiel
///
/// Russia:
///     F Baltic Sea Supports A Prussia - Berlin
///     A Prussia - Berlin
///
/// None of the moves succeeds.
/// This case is especially difficult for sequence based adjudicators.
/// They will start adjudicating the head-to-head battle and continue to adjudicate
/// the attack on one of the units which is part of the head-to-head battle.
/// In this process, one of the sides of the head-to-head battle might be cancelled out.
#[test]
fn test_datc_6_e_15() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_hol = f("e", "hol");
    let unit_e_ruh = a("e", "ruh");
    let unit_f_kie = a("f", "kie");
    let unit_f_mun = a("f", "mun");
    let unit_f_sil = a("f", "sil");
    let unit_g_ber = a("g", "ber");
    let unit_g_den = f("g", "den");
    let unit_g_hel = f("g", "hel");
    let unit_r_bal = f("r", "bal");
    let unit_r_pru = a("r", "pru");
    phase.data.units.push(unit_e_hol);
    phase.data.units.push(unit_e_ruh);
    phase.data.units.push(unit_f_kie);
    phase.data.units.push(unit_f_mun);
    phase.data.units.push(unit_f_sil);
    phase.data.units.push(unit_g_ber);
    phase.data.units.push(unit_g_den);
    phase.data.units.push(unit_g_hel);
    phase.data.units.push(unit_r_bal);
    phase.data.units.push(unit_r_pru);
    phase.data.orders.push(unit_e_hol.support_move(unit_e_ruh, p("kie")));
    phase.data.orders.push(unit_e_ruh.move_to(p("kie")));
    phase.data.orders.push(unit_f_kie.move_to(p("ber")));
    phase.data.orders.push(unit_f_mun.support_move(unit_f_kie, p("ber")));
    phase.data.orders.push(unit_f_sil.support_move(unit_f_kie, p("ber")));
    phase.data.orders.push(unit_g_ber.move_to(p("kie")));
    phase.data.orders.push(unit_g_den.support_move(unit_g_ber, p("kie")));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_ber, p("kie")));
    phase.data.orders.push(unit_r_bal.support_move(unit_r_pru, p("ber")));
    phase.data.orders.push(unit_r_pru.move_to(p("ber")));
    Phase::resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[8].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[9].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 10);
    assert!(phase.data.units.contains(&unit_e_hol));
    assert!(phase.data.units.contains(&unit_e_ruh));
    assert!(phase.data.units.contains(&unit_f_kie));
    assert!(phase.data.units.contains(&unit_f_mun));
    assert!(phase.data.units.contains(&unit_f_sil));
    assert!(phase.data.units.contains(&unit_g_ber));
    assert!(phase.data.units.contains(&unit_g_den));
    assert!(phase.data.units.contains(&unit_g_hel));
    assert!(phase.data.units.contains(&unit_r_bal));
    assert!(phase.data.units.contains(&unit_r_pru));
    assert!(phase.data.standoff_codes.is_empty());
}
