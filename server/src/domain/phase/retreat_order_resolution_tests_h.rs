//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6H]
//!
//! * 6.H. TEST CASES, RETREATING
//!
//! [DATC_6H]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.H

use super::super::order::*;
use super::super::phase::retreat_order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.H.1. TEST CASE, NO SUPPORTS DURING RETREAT
/// Supports are not allowed in the retreat phase.
///
/// Austria:
///     F Trieste Hold
///     A Serbia Hold
///
/// Turkey:
///     F Greece Hold
///
/// Italy:
///     A Venice Supports A Tyrolia - Trieste
///     A Tyrolia - Trieste
///     F Ionian Sea - Greece
///     F Aegean Sea Supports F Ionian Sea - Greece
///
/// The fleet in Trieste and the fleet in Greece are dislodged. If the retreat orders are as follows:
///
/// Austria:
///     F Trieste - Albania
///     A Serbia Supports F Trieste - Albania
///
/// Turkey:
///     F Greece - Albania
///
/// The Austrian support order is illegal. Both dislodged fleets are disbanded.
#[allow(unused)]
fn test_datc_6_h_1() {
    // 撤退フェイズでは解体か撤退以外の命令は受け付けないためテスト不要
}

/// 6.H.2. TEST CASE, NO SUPPORTS FROM RETREATING UNIT
/// Even a retreating unit cannot give support.
///
/// England:
///     A Liverpool - Edinburgh
///     F Yorkshire Supports A Liverpool - Edinburgh
///     F Norway Hold
///
/// Germany:
///     A Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// Russia:
///     F Edinburgh Hold
///     A Sweden Supports A Finland - Norway
///     A Finland - Norway
///     F Holland Hold
///
/// The English fleet in Norway and the Russian fleets in Edinburgh and Holland are dislodged. If the following retreat orders are given:
///
/// England:
///     F Norway - North Sea
///
/// Russia:
///     F Edinburgh - North Sea
///     F Holland Supports F Edinburgh - North Sea
///
/// Although the fleet in Holland may receive an order, it may not support (it is disbanded).
/// The English fleet in Norway and the Russian fleet in Edinburgh bounce and are disbanded.
#[allow(unused)]
fn test_datc_6_h_2() {
    // 撤退フェイズでは解体か撤退以外の命令は受け付けないためテスト不要
}

/// 6.H.3. TEST CASE, NO CONVOY DURING RETREAT
/// Convoys during retreat are not allowed.
///
/// England:
///     F North Sea Hold
///     A Holland Hold
///
/// Germany:
///     F Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// The English army in Holland is dislodged. If England orders the following in retreat:
///
/// England:
///     A Holland - Yorkshire
///     F North Sea Convoys A Holland - Yorkshire
///
/// The convoy order is illegal. The army in Holland is disbanded.
#[allow(unused)]
fn test_datc_6_h_3() {
    // 撤退フェイズでは解体か撤退以外の命令は受け付けないためテスト不要
}

/// 6.H.4. TEST CASE, NO OTHER MOVES DURING RETREAT
/// Of course, you may not do any other move during a retreat. But look if the adjudicator checks for it.
///
/// England:
///     F North Sea Hold
///     A Holland Hold
///
/// Germany:
///     F Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// The English army in Holland is dislodged. If England orders the following in retreat:
///
/// England:
///     A Holland - Belgium
///     F North Sea - Norwegian Sea
///
/// The fleet in the North Sea is not dislodge, so the move is illegal.
#[allow(unused)]
fn test_datc_6_h_4() {
    // 撤退フェイズでは通常ユニットへの命令は受け付けないためテスト不要
}

/// 6.H.5. TEST CASE, A UNIT MAY NOT RETREAT TO THE AREA FROM WHICH IT IS ATTACKED
/// Well, that would be of course stupid. Still, the adjudicator must be tested on this.
///
/// Russia:
///     F Constantinople Supports F Black Sea - Ankara
///     F Black Sea - Ankara
///
/// Turkey:
///     F Ankara Hold
///
/// Fleet in Ankara is dislodged and may not retreat to Black Sea.
#[test]
fn test_datc_6_h_5() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_r_con = Unit::new_fleet(Power::Russia, p("con"));
    let unit_r_ank = Unit::new_fleet(Power::Russia, p("ank"));
    let unit_t_ank = Unit::new_fleet(Power::Turkey, p("ank")).dislodged_from(p("bla"));
    context.last_resolved_units.push(unit_r_con);
    context.last_resolved_units.push(unit_r_ank);
    context.last_resolved_units.push(unit_t_ank);
    phase.data.orders.push(unit_t_ank.retreat_to(p("bla")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.H.6. TEST CASE, UNIT MAY NOT RETREAT TO A CONTESTED AREA
/// Standoff prevents retreat to the area.
///
/// Austria:
///     A Budapest Supports A Trieste - Vienna
///     A Trieste - Vienna
///
/// Germany:
///     A Munich - Bohemia
///     A Silesia - Bohemia
///
/// Italy:
///     A Vienna Hold
///
/// The Italian army in Vienna is dislodged. It may not retreat to Bohemia.
#[test]
fn test_datc_6_h_6() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_a_bud = Unit::new_army(Power::Austria, p("bud"));
    let unit_a_vie = Unit::new_army(Power::Austria, p("vie"));
    let unit_g_mun = Unit::new_army(Power::Germany, p("mun"));
    let unit_g_sil = Unit::new_army(Power::Germany, p("sil"));
    let unit_i_vie = Unit::new_army(Power::Italy, p("vie")).dislodged_from(p("tri"));
    context.last_resolved_units.push(unit_a_bud);
    context.last_resolved_units.push(unit_a_vie);
    context.last_resolved_units.push(unit_i_vie);
    context.last_resolved_units.push(unit_g_mun);
    context.last_resolved_units.push(unit_g_sil);
    context.standoff_codes.push(&p("boh").code()[..3]);
    phase.data.orders.push(unit_i_vie.retreat_to(p("boh")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
}

/// 6.H.7. TEST CASE, MULTIPLE RETREAT TO SAME AREA WILL DISBAND UNITS
/// There can only be one unit in an area.
///
/// Austria:
///     A Budapest Supports A Trieste - Vienna
///     A Trieste - Vienna
///
/// Germany:
///     A Munich Supports A Silesia - Bohemia
///     A Silesia - Bohemia
///
/// Italy:
///     A Vienna Hold
///     A Bohemia Hold
///
/// If Italy orders the following for retreat:
///
/// Italy:
///     A Bohemia - Tyrolia
///     A Vienna - Tyrolia
///
/// Both armies will be disbanded.
#[test]
fn test_datc_6_h_7() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_a_vie = Unit::new_army(Power::Austria, p("vie"));
    let unit_a_tri = Unit::new_army(Power::Austria, p("tri"));
    let unit_g_mun = Unit::new_army(Power::Germany, p("mun"));
    let unit_g_boh = Unit::new_army(Power::Germany, p("boh"));
    let unit_i_vie = Unit::new_army(Power::Italy, p("vie")).dislodged_from(p("tri"));
    let unit_i_boh = Unit::new_army(Power::Italy, p("boh")).dislodged_from(p("sil"));
    context.last_resolved_units.push(unit_a_tri);
    context.last_resolved_units.push(unit_a_vie);
    context.last_resolved_units.push(unit_g_mun);
    context.last_resolved_units.push(unit_g_boh);
    context.last_resolved_units.push(unit_i_vie);
    context.last_resolved_units.push(unit_i_boh);
    phase.data.orders.push(unit_i_vie.retreat_to(p("tyr")));
    phase.data.orders.push(unit_i_boh.retreat_to(p("tyr")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
}

/// 6.H.8. TEST CASE, TRIPLE RETREAT TO SAME AREA WILL DISBAND UNITS
/// When three units retreat to the same area, then all three units are disbanded.
///
/// England:
///     A Liverpool - Edinburgh
///     F Yorkshire Supports A Liverpool - Edinburgh
///     F Norway Hold
///
/// Germany:
///     A Kiel Supports A Ruhr - Holland
///     A Ruhr - Holland
///
/// Russia:
///     F Edinburgh Hold
///     A Sweden Supports A Finland - Norway
///     A Finland - Norway
///     F Holland Hold
///
/// The fleets in Norway, Edinburgh and Holland are dislodged. If the following retreat orders are given:
///
/// England:
///     F Norway - North Sea
///
/// Russia:
///     F Edinburgh - North Sea
///     F Holland - North Sea
///
/// All three units are disbanded.
#[test]
fn test_datc_6_h_8() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_e_edi = Unit::new_army(Power::England, p("edi"));
    let unit_e_yor = Unit::new_fleet(Power::England, p("yor"));
    let unit_e_nwy = Unit::new_fleet(Power::England, p("nwy")).dislodged_from(p("fin"));
    let unit_g_kie = Unit::new_fleet(Power::Germany, p("kie"));
    let unit_g_hol = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_r_edi = Unit::new_fleet(Power::Russia, p("edi")).dislodged_from(p("lvp"));
    let unit_r_swe = Unit::new_army(Power::Russia, p("swe"));
    let unit_r_nwy = Unit::new_army(Power::Russia, p("nwy"));
    let unit_r_hol = Unit::new_fleet(Power::Russia, p("hol")).dislodged_from(p("ruh"));
    context.last_resolved_units.push(unit_e_edi);
    context.last_resolved_units.push(unit_e_yor);
    context.last_resolved_units.push(unit_e_nwy);
    context.last_resolved_units.push(unit_g_kie);
    context.last_resolved_units.push(unit_g_hol);
    context.last_resolved_units.push(unit_r_edi);
    context.last_resolved_units.push(unit_r_swe);
    context.last_resolved_units.push(unit_r_nwy);
    context.last_resolved_units.push(unit_r_hol);
    phase.data.orders.push(unit_e_nwy.retreat_to(p("nth")));
    phase.data.orders.push(unit_r_edi.retreat_to(p("nth")));
    phase.data.orders.push(unit_r_hol.retreat_to(p("nth")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.H.9. TEST CASE, DISLODGED UNIT WILL NOT MAKE ATTACKERS AREA CONTESTED
/// An army can follow.
///
/// England:
///     F Helgoland Bight - Kiel
///     F Denmark Supports F Helgoland Bight - Kiel
///
/// Germany:
///     A Berlin - Prussia
///     F Kiel Hold
///     A Silesia Supports A Berlin - Prussia
///
/// Russia:
///     A Prussia - Berlin
///
/// The fleet in Kiel can retreat to Berlin.
#[test]
fn test_datc_6_h_9() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_e_kie = Unit::new_fleet(Power::England, p("kie"));
    let unit_e_den = Unit::new_fleet(Power::England, p("den"));
    let unit_g_pru = Unit::new_army(Power::Germany, p("pru"));
    let unit_g_kie = Unit::new_fleet(Power::Germany, p("kie")).dislodged_from(p("hel"));
    let unit_g_sil = Unit::new_army(Power::Germany, p("sil"));
    let unit_r_pru = Unit::new_army(Power::Russia, p("pru")).dislodged_from(p("ber"));
    context.last_resolved_units.push(unit_e_kie);
    context.last_resolved_units.push(unit_e_den);
    context.last_resolved_units.push(unit_g_pru);
    context.last_resolved_units.push(unit_g_kie);
    context.last_resolved_units.push(unit_g_sil);
    context.last_resolved_units.push(unit_r_pru);
    phase.data.orders.push(unit_g_kie.retreat_to(p("ber")));
    phase.data.orders.push(unit_r_pru.retreat_to(p("ber")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
}

/// 6.H.10. TEST CASE, NOT RETREATING TO ATTACKER DOES NOT MEAN CONTESTED
/// An army cannot retreat to the area of the attacker.
/// The easiest way to program that, is to mark that area as "contested".
/// However, this is not correct. Another army may retreat to that area.
///
/// England:
///     A Kiel Hold
///
/// Germany:
///     A Berlin - Kiel
///     A Munich Supports A Berlin - Kiel
///     A Prussia Hold
///
/// Russia:
///     A Warsaw - Prussia
///     A Silesia Supports A Warsaw - Prussia
///
/// The armies in Kiel and Prussia are dislodged.
/// The English army in Kiel cannot retreat to Berlin, but the army in Prussia can retreat to Berlin.
/// Suppose the following retreat orders are given:
///
/// England:
///     A Kiel - Berlin
///
/// Germany:
///     A Prussia - Berlin
///
/// The English retreat to Berlin is illegal and fails (the unit is disbanded).
/// The German retreat to Berlin is successful and does not bounce on the English unit.
#[test]
fn test_datc_6_h_10() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_e_kie = Unit::new_army(Power::England, p("kie")).dislodged_from(p("ber"));
    let unit_g_kie = Unit::new_army(Power::Germany, p("kie"));
    let unit_g_mun = Unit::new_army(Power::Germany, p("mun"));
    let unit_g_pru = Unit::new_army(Power::Germany, p("pru")).dislodged_from(p("war"));
    let unit_r_pru = Unit::new_army(Power::Russia, p("pru"));
    let unit_r_sil = Unit::new_army(Power::Russia, p("sil"));
    context.last_resolved_units.push(unit_e_kie);
    context.last_resolved_units.push(unit_g_kie);
    context.last_resolved_units.push(unit_g_mun);
    context.last_resolved_units.push(unit_g_pru);
    context.last_resolved_units.push(unit_r_pru);
    context.last_resolved_units.push(unit_r_sil);
    phase.data.orders.push(unit_e_kie.retreat_to(p("ber")));
    phase.data.orders.push(unit_g_pru.retreat_to(p("ber")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
}

/// 6.H.11. TEST CASE, RETREAT WHEN DISLODGED BY ADJACENT CONVOY
/// If a unit is dislodged by an army via convoy,
/// the question arises whether the dislodged army can retreat to the original province of the convoyed army.
/// This is only relevant in case the convoy was to an adjacent province.
///
/// France:
///     A Gascony - Marseilles via convoy
///     A Burgundy Supports A Gascony - Marseilles
///     F Mid-Atlantic Ocean Convoys A Gascony - Marseilles
///     F Western Mediterranean Convoys A Gascony - Marseilles
///     F Gulf of Lyon Convoys A Gascony - Marseilles
///
/// Italy:
///     A Marseilles Hold
///
/// The army in Gascony takes a convoy and does not pass the border of Gascony with Marseilles
/// (it went a completely different direction).
/// Now, the result depends on which rule is used for retreating (see issue 4.A.5).
///
/// The 2023 rules explicitly allow this. So, I prefer that Marseilles may retreat to Gascony.
#[test]
fn test_datc_6_h_11() {
    let mut phase = Phase::new_spring_retreat(1901, 2);
    let context = &mut PhaseContext::new();
    let unit_f_mar = Unit::new_army(Power::France, p("mar"));
    let unit_f_bur = Unit::new_army(Power::France, p("bur"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    let unit_f_wes = Unit::new_fleet(Power::France, p("wes"));
    let unit_f_lyo = Unit::new_fleet(Power::France, p("lyo"));
    let unit_i_mar = Unit::new_army(Power::Italy, p("mar")).dislodged_via_convoy();
    context.last_resolved_units.push(unit_f_mar);
    context.last_resolved_units.push(unit_f_mar);
    context.last_resolved_units.push(unit_f_bur);
    context.last_resolved_units.push(unit_f_mao);
    context.last_resolved_units.push(unit_f_wes);
    context.last_resolved_units.push(unit_f_lyo);
    context.last_resolved_units.push(unit_i_mar);
    phase.data.orders.push(unit_i_mar.retreat_to(p("gas")));
    resolve_orders_for_retreat_phase(&mut phase, context);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
}

/// 6.H.12. TEST CASE, RETREAT WHEN DISLODGED BY ADJACENT CONVOY WHILE TRYING TO DO THE SAME
/// The previous test case can be made more extra ordinary, when both armies tried to move by convoy.
///
/// England:
/// A Liverpool - Edinburgh via convoy
/// F Irish Sea Convoys A Liverpool - Edinburgh
/// F English Channel Convoys A Liverpool - Edinburgh
/// F North Sea Convoys A Liverpool - Edinburgh
///
/// France:
/// F Brest - English Channel
/// F Mid-Atlantic Ocean Supports F Brest - English Channel
///
/// Russia:
/// A Edinburgh - Liverpool via convoy
/// F Norwegian Sea Convoys A Edinburgh - Liverpool
/// F North Atlantic Ocean Convoys A Edinburgh - Liverpool
/// A Clyde Supports A Edinburgh - Liverpool
/// Both the army in Liverpool as in Edinburgh will try to move by convoy. The army in Edinburgh will succeed. The army in Liverpool will fail, because of the disrupted convoy. It is dislodged by the army of Edinburgh. Now, the question is whether the army in Liverpool may retreat to Edinburgh. The result depends on which rule is used for retreating (see issue 4.A.5).
///
/// The 2023 rules, which I prefer, explicitly allow that the army in Liverpool may retreat to Edinburgh.
#[test]
fn test_datc_6_h_12() {}

/// 6.H.13. TEST CASE, NO RETREAT WITH CONVOY IN MOVEMENT PHASE
/// The areas where a unit may retreat to, must be determined during the movement phase. Care should be taken that a convoy ordered in the movement phase cannot be used in the retreat phase.
///
/// England:
/// A Picardy Hold
/// F English Channel Convoys A Picardy - London
///
/// France:
/// A Paris - Picardy
/// A Brest Supports A Paris - Picardy
/// The dislodged army in Picardy cannot retreat to London.
#[test]
fn test_datc_6_h_13() {}

/// 6.H.14. TEST CASE, NO RETREAT WITH SUPPORT IN MOVEMENT PHASE
/// Comparable to the previous test case, a support given in the movement phase cannot be used in the retreat phase.
///
/// England:
/// A Picardy Hold
/// F English Channel Supports A Picardy - Belgium
///
/// France:
/// A Paris - Picardy
/// A Brest Supports A Paris - Picardy
/// A Burgundy Hold
///
/// Germany:
/// A Munich Supports A Marseilles - Burgundy
/// A Marseilles - Burgundy
/// After the movement phase the following retreat orders are given:
///
/// England:
/// A Picardy - Belgium
///
/// France:
/// A Burgundy - Belgium
/// Both the army in Picardy and Burgundy are disbanded.
#[test]
fn test_datc_6_h_14() {}

/// 6.H.15. TEST CASE, NO COASTAL CRAWL IN RETREAT
/// You cannot go to the other coast from where the attacker came from.
///
/// England:
/// F Portugal Hold
///
/// France:
/// F Spain(sc) - Portugal
/// F Mid-Atlantic Ocean Supports F Spain(sc) - Portugal
/// The English fleet in Portugal is destroyed and cannot retreat to Spain(nc).
#[test]
fn test_datc_6_h_15() {}

/// 6.H.16. TEST CASE, CONTESTED FOR BOTH COASTS
/// If a coast is contested, the other is not available for retreat.
///
/// France:
/// F Mid-Atlantic Ocean - Spain(nc)
/// F Gascony - Spain(nc)
/// F Western Mediterranean Hold
///
/// Italy:
/// F Tunis Supports F Tyrrhenian Sea - Western Mediterranean
/// F Tyrrhenian Sea - Western Mediterranean
/// The French fleet in the Western Mediterranean cannot retreat to Spain(sc).
#[test]
fn test_datc_6_h_16() {}
