//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6H]
//!
//! * 6.H. TEST CASES, RETREATING
//!
//! [DATC_6H]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.H

use crate::domain::models::order::*;
use crate::domain::models::phase::*;
use crate::domain::tests::a;
use crate::domain::tests::f;
use crate::domain::tests::p;

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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_r_con = f("r", "con");
    let unit_r_bla = f("r", "bla");
    let mut unit_t_ank = f("t", "ank");
    main_phase.data.units.push(unit_r_con);
    main_phase.data.units.push(unit_r_bla);
    main_phase.data.units.push(unit_t_ank);
    main_phase.data.orders.push(unit_r_con.support_move(unit_r_bla, p("ank")));
    main_phase.data.orders.push(unit_r_bla.move_to(p("ank")));
    main_phase.data.orders.push(unit_t_ank.hold());

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_t_ank.dislodged_from(p("bla")).retreat_to(p("bla")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(retreat_phase.data.units.len(), 2);
    assert!(retreat_phase.data.units.contains(&unit_r_con));
    assert!(retreat_phase.data.units.contains(&f("r", "ank")));
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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_a_bud = a("a", "bud");
    let unit_a_tri = a("a", "tri");
    let unit_g_mun = a("g", "mun");
    let unit_g_sil = a("g", "sil");
    let mut unit_i_vie = a("i", "vie");
    main_phase.data.units.push(unit_a_bud);
    main_phase.data.units.push(unit_a_tri);
    main_phase.data.units.push(unit_g_mun);
    main_phase.data.units.push(unit_g_sil);
    main_phase.data.units.push(unit_i_vie);
    main_phase.data.orders.push(unit_a_bud.support_move(unit_a_tri, p("vie")));
    main_phase.data.orders.push(unit_a_tri.move_to(p("vie")));
    main_phase.data.orders.push(unit_g_mun.move_to(p("boh")));
    main_phase.data.orders.push(unit_g_sil.move_to(p("boh")));
    main_phase.data.orders.push(unit_i_vie.hold());

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_i_vie.dislodged_from(p("tri")).retreat_to(p("boh")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(retreat_phase.data.units.len(), 4);
    assert!(retreat_phase.data.units.contains(&unit_a_bud));
    assert!(retreat_phase.data.units.contains(&a("a", "vie")));
    assert!(retreat_phase.data.units.contains(&unit_g_mun));
    assert!(retreat_phase.data.units.contains(&unit_g_sil));
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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_a_bud = a("a", "bud");
    let unit_a_tri = a("a", "tri");
    let unit_g_mun = a("g", "mun");
    let unit_g_sil = a("g", "sil");
    let mut unit_i_vie = a("i", "vie");
    let mut unit_i_boh = a("i", "boh");
    main_phase.data.units.push(unit_a_bud);
    main_phase.data.units.push(unit_a_tri);
    main_phase.data.units.push(unit_g_mun);
    main_phase.data.units.push(unit_g_sil);
    main_phase.data.units.push(unit_i_vie);
    main_phase.data.units.push(unit_i_boh);
    main_phase.data.orders.push(unit_a_bud.support_move(unit_a_tri, p("vie")));
    main_phase.data.orders.push(unit_a_tri.move_to(p("vie")));
    main_phase.data.orders.push(unit_g_mun.support_move(unit_g_sil, p("boh")));
    main_phase.data.orders.push(unit_g_sil.move_to(p("boh")));
    main_phase.data.orders.push(unit_i_vie.hold());
    main_phase.data.orders.push(unit_i_boh.hold());

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_i_vie.dislodged_from(p("tri")).retreat_to(p("tyr")));
    retreat_phase
        .data
        .orders
        .push(unit_i_boh.dislodged_from(p("sil")).retreat_to(p("tyr")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(retreat_phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(retreat_phase.data.units.len(), 4);
    assert!(retreat_phase.data.units.contains(&unit_a_bud));
    assert!(retreat_phase.data.units.contains(&a("a", "vie")));
    assert!(retreat_phase.data.units.contains(&unit_g_mun));
    assert!(retreat_phase.data.units.contains(&a("g", "boh")));
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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_e_lvp = a("e", "lvp");
    let unit_e_yor = f("e", "yor");
    let mut unit_e_nwy = f("e", "nwy");
    let unit_g_kie = a("g", "kie");
    let unit_g_ruh = a("g", "ruh");
    let mut unit_r_edi = f("r", "edi");
    let unit_r_swe = a("r", "swe");
    let unit_r_fin = a("r", "fin");
    let mut unit_r_hol = f("r", "hol");
    main_phase.data.units.push(unit_e_lvp);
    main_phase.data.units.push(unit_e_yor);
    main_phase.data.units.push(unit_e_nwy);
    main_phase.data.units.push(unit_g_kie);
    main_phase.data.units.push(unit_g_ruh);
    main_phase.data.units.push(unit_r_edi);
    main_phase.data.units.push(unit_r_swe);
    main_phase.data.units.push(unit_r_fin);
    main_phase.data.units.push(unit_r_hol);
    main_phase.data.orders.push(unit_e_lvp.move_to(p("edi")));
    main_phase.data.orders.push(unit_e_yor.support_move(unit_e_lvp, p("edi")));
    main_phase.data.orders.push(unit_e_nwy.hold());
    main_phase.data.orders.push(unit_g_kie.support_move(unit_g_ruh, p("hol")));
    main_phase.data.orders.push(unit_g_ruh.move_to(p("hol")));
    main_phase.data.orders.push(unit_r_edi.hold());
    main_phase.data.orders.push(unit_r_swe.support_move(unit_r_fin, p("nwy")));
    main_phase.data.orders.push(unit_r_fin.move_to(p("nwy")));
    main_phase.data.orders.push(unit_r_hol.hold());

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_e_nwy.dislodged_from(p("fin")).retreat_to(p("nth")));
    retreat_phase
        .data
        .orders
        .push(unit_r_edi.dislodged_from(p("lvp")).retreat_to(p("nth")));
    retreat_phase
        .data
        .orders
        .push(unit_r_hol.dislodged_from(p("ruh")).retreat_to(p("nth")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(retreat_phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(retreat_phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(retreat_phase.data.units.len(), 6);
    assert!(retreat_phase.data.units.contains(&a("e", "edi")));
    assert!(retreat_phase.data.units.contains(&unit_e_yor));
    assert!(retreat_phase.data.units.contains(&unit_g_kie));
    assert!(retreat_phase.data.units.contains(&a("g", "hol")));
    assert!(retreat_phase.data.units.contains(&unit_r_swe));
    assert!(retreat_phase.data.units.contains(&a("r", "nwy")));
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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_e_hel = f("e", "hel");
    let unit_e_den = f("e", "den");
    let unit_g_ber = a("g", "ber");
    let mut unit_g_kie = f("g", "kie");
    let unit_g_sil = a("g", "sil");
    let mut unit_r_pru = a("r", "pru");
    main_phase.data.units.push(unit_e_hel);
    main_phase.data.units.push(unit_e_den);
    main_phase.data.units.push(unit_g_ber);
    main_phase.data.units.push(unit_g_kie);
    main_phase.data.units.push(unit_g_sil);
    main_phase.data.units.push(unit_r_pru);
    main_phase.data.orders.push(unit_e_hel.move_to(p("kie")));
    main_phase.data.orders.push(unit_e_den.support_move(unit_e_hel, p("kie")));
    main_phase.data.orders.push(unit_g_ber.move_to(p("pru")));
    main_phase.data.orders.push(unit_g_kie.hold());
    main_phase.data.orders.push(unit_g_sil.support_move(unit_g_ber, p("pru")));
    main_phase.data.orders.push(unit_r_pru.move_to(p("ber")));

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_g_kie.dislodged_from(p("hel")).retreat_to(p("ber")));
    retreat_phase
        .data
        .orders
        .push(unit_r_pru.dislodged_from(p("ber")).retreat_to(p("ber")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(retreat_phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(retreat_phase.data.units.len(), 5);
    assert!(retreat_phase.data.units.contains(&f("e", "kie")));
    assert!(retreat_phase.data.units.contains(&unit_e_den));
    assert!(retreat_phase.data.units.contains(&a("g", "pru")));
    assert!(retreat_phase.data.units.contains(&unit_g_sil));
    assert!(retreat_phase.data.units.contains(&f("g", "ber")));
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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let mut unit_e_kie = a("e", "kie");
    let unit_g_ber = a("g", "ber");
    let unit_g_mun = a("g", "mun");
    let mut unit_g_pru = a("g", "pru");
    let unit_r_war = a("r", "war");
    let unit_r_sil = a("r", "sil");
    main_phase.data.units.push(unit_e_kie);
    main_phase.data.units.push(unit_g_ber);
    main_phase.data.units.push(unit_g_mun);
    main_phase.data.units.push(unit_g_pru);
    main_phase.data.units.push(unit_r_war);
    main_phase.data.units.push(unit_r_sil);
    main_phase.data.orders.push(unit_e_kie.hold());
    main_phase.data.orders.push(unit_g_ber.move_to(p("kie")));
    main_phase.data.orders.push(unit_g_mun.support_move(unit_g_ber, p("kie")));
    main_phase.data.orders.push(unit_g_pru.hold());
    main_phase.data.orders.push(unit_r_war.move_to(p("pru")));
    main_phase.data.orders.push(unit_r_sil.support_move(unit_r_war, p("pru")));

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_e_kie.dislodged_from(p("ber")).retreat_to(p("ber")));
    retreat_phase
        .data
        .orders
        .push(unit_g_pru.dislodged_from(p("war")).retreat_to(p("ber")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(retreat_phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(retreat_phase.data.units.len(), 5);
    assert!(retreat_phase.data.units.contains(&a("g", "kie")));
    assert!(retreat_phase.data.units.contains(&unit_g_mun));
    assert!(retreat_phase.data.units.contains(&a("r", "pru")));
    assert!(retreat_phase.data.units.contains(&unit_r_sil));
    assert!(retreat_phase.data.units.contains(&a("g", "ber")));
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
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_f_gas = a("f", "gas");
    let unit_f_bur = a("f", "bur");
    let unit_f_mid = f("f", "mid");
    let unit_f_wes = f("f", "wes");
    let unit_f_gol = f("f", "gol");
    let mut unit_i_mar = a("i", "mar");
    main_phase.data.units.push(unit_f_gas);
    main_phase.data.units.push(unit_f_bur);
    main_phase.data.units.push(unit_f_mid);
    main_phase.data.units.push(unit_f_wes);
    main_phase.data.units.push(unit_f_gol);
    main_phase.data.units.push(unit_i_mar);
    main_phase.data.orders.push(unit_f_gas.move_to(p("mar")).set_via_convoy());
    main_phase.data.orders.push(unit_f_bur.support_move(unit_f_gas, p("mar")));
    main_phase.data.orders.push(unit_f_mid.convoy(unit_f_gas, p("mar")));
    main_phase.data.orders.push(unit_f_wes.convoy(unit_f_gas, p("mar")));
    main_phase.data.orders.push(unit_f_gol.convoy(unit_f_gas, p("mar")));
    main_phase.data.orders.push(unit_i_mar.hold());

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_i_mar.dislodged_via_convoy().retreat_to(p("gas")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(retreat_phase.data.units.len(), 6);
    assert!(retreat_phase.data.units.contains(&a("f", "mar")));
    assert!(retreat_phase.data.units.contains(&unit_f_bur));
    assert!(retreat_phase.data.units.contains(&unit_f_mid));
    assert!(retreat_phase.data.units.contains(&unit_f_wes));
    assert!(retreat_phase.data.units.contains(&unit_f_gol));
    assert!(retreat_phase.data.units.contains(&a("i", "gas")));
}

/// 6.H.12. TEST CASE, RETREAT WHEN DISLODGED BY ADJACENT CONVOY WHILE TRYING TO DO THE SAME
/// The previous test case can be made more extra ordinary,
/// when both armies tried to move by convoy.
///
/// England:
///     A Liverpool - Edinburgh via convoy
///     F Irish Sea Convoys A Liverpool - Edinburgh
///     F English Channel Convoys A Liverpool - Edinburgh
///     F North Sea Convoys A Liverpool - Edinburgh
///
/// France:
///     F Brest - English Channel
///     F Mid-Atlantic Ocean Supports F Brest - English Channel
///
/// Russia:
///     A Edinburgh - Liverpool via convoy
///     F Norwegian Sea Convoys A Edinburgh - Liverpool
///     F North Atlantic Ocean Convoys A Edinburgh - Liverpool
///     A Clyde Supports A Edinburgh - Liverpool
///
/// Both the army in Liverpool as in Edinburgh will try to move by convoy.
/// The army in Edinburgh will succeed.
/// The army in Liverpool will fail, because of the disrupted convoy.
/// It is dislodged by the army of Edinburgh.
/// Now, the question is whether the army in Liverpool may retreat to Edinburgh.
/// The result depends on which rule is used for retreating (see issue 4.A.5).
/// The 2023 rules, which I prefer,
/// explicitly allow that the army in Liverpool may retreat to Edinburgh.
#[test]
fn test_datc_6_h_12() {
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let mut unit_e_lvp = a("e", "lvp");
    let unit_e_iri = f("e", "iri");
    let unit_e_eng = f("e", "eng");
    let unit_e_nth = f("e", "nth");
    let unit_f_bre = f("f", "bre");
    let unit_f_mid = f("f", "mid");
    let unit_r_edi = a("r", "edi");
    let unit_r_nrg = f("r", "nrg");
    let unit_r_nat = f("r", "nat");
    let unit_r_cly = a("r", "cly");
    main_phase.data.units.push(unit_e_lvp);
    main_phase.data.units.push(unit_e_iri);
    main_phase.data.units.push(unit_e_eng);
    main_phase.data.units.push(unit_e_nth);
    main_phase.data.units.push(unit_f_bre);
    main_phase.data.units.push(unit_f_mid);
    main_phase.data.units.push(unit_r_edi);
    main_phase.data.units.push(unit_r_nrg);
    main_phase.data.units.push(unit_r_nat);
    main_phase.data.units.push(unit_r_cly);
    main_phase.data.orders.push(unit_e_lvp.move_to(p("edi")).set_via_convoy());
    main_phase.data.orders.push(unit_e_iri.convoy(unit_e_lvp, p("edi")));
    main_phase.data.orders.push(unit_e_eng.convoy(unit_e_lvp, p("edi")));
    main_phase.data.orders.push(unit_e_nth.convoy(unit_e_lvp, p("edi")));
    main_phase.data.orders.push(unit_f_bre.move_to(p("eng")));
    main_phase.data.orders.push(unit_f_mid.support_move(unit_f_bre, p("eng")));
    main_phase.data.orders.push(unit_r_edi.move_to(p("lvp")).set_via_convoy());
    main_phase.data.orders.push(unit_r_nrg.convoy(unit_r_edi, p("lvp")));
    main_phase.data.orders.push(unit_r_nat.convoy(unit_r_edi, p("lvp")));
    main_phase.data.orders.push(unit_r_cly.support_move(unit_r_edi, p("lvp")));

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_e_lvp.dislodged_via_convoy().retreat_to(p("edi")));
    retreat_phase.data.orders.push(unit_e_eng.disband());
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(retreat_phase.data.units.len(), 10);
    assert!(retreat_phase.data.units.contains(&a("e", "edi")));
    assert!(retreat_phase.data.units.contains(&unit_e_iri));
    assert!(retreat_phase.data.units.contains(&unit_e_nth));
    assert!(retreat_phase.data.units.contains(&f("f", "eng")));
    assert!(retreat_phase.data.units.contains(&unit_f_mid));
    assert!(retreat_phase.data.units.contains(&a("r", "lvp")));
    assert!(retreat_phase.data.units.contains(&unit_r_nrg));
    assert!(retreat_phase.data.units.contains(&unit_r_nat));
    assert!(retreat_phase.data.units.contains(&unit_r_cly));
}

/// 6.H.13. TEST CASE, NO RETREAT WITH CONVOY IN MOVEMENT PHASE
/// The areas where a unit may retreat to, must be determined during the movement phase.
/// Care should be taken that a convoy ordered in the movement phase cannot be used in the retreat phase.
///
/// England:
///     A Picardy Hold
///     F English Channel Convoys A Picardy - London
///
/// France:
///     A Paris - Picardy
///     A Brest Supports A Paris - Picardy
///
/// The dislodged army in Picardy cannot retreat to London.
#[allow(unused)]
fn test_datc_6_h_13() {
    // 撤退フェイズでは解体か撤退以外の命令は受け付けないためテスト不要
}

/// 6.H.14. TEST CASE, NO RETREAT WITH SUPPORT IN MOVEMENT PHASE
/// Comparable to the previous test case,
/// a support given in the movement phase cannot be used in the retreat phase.
///
/// England:
///     A Picardy Hold
///     F English Channel Supports A Picardy - Belgium
///
/// France:
///     A Paris - Picardy
///     A Brest Supports A Paris - Picardy
///     A Burgundy Hold
///
/// Germany:
///     A Munich Supports A Marseilles - Burgundy
///     A Marseilles - Burgundy
///     After the movement phase the following retreat orders are given:
///
/// England:
///     A Picardy - Belgium
///
/// France:
///     A Burgundy - Belgium
///
/// Both the army in Picardy and Burgundy are disbanded.
#[allow(unused)]
fn test_datc_6_h_14() {
    // 撤退フェイズでは解体か撤退以外の命令は受け付けないためテスト不要
}

/// 6.H.15. TEST CASE, NO COASTAL CRAWL IN RETREAT
/// You cannot go to the other coast from where the attacker came from.
///
/// England:
///     F Portugal Hold
///
/// France:
///     F Spain(sc) - Portugal
///     F Mid-Atlantic Ocean Supports F Spain(sc) - Portugal
///
/// The English fleet in Portugal is destroyed and cannot retreat to Spain(nc).
#[test]
fn test_datc_6_h_15() {
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let mut unit_e_por = f("e", "por");
    let unit_f_spa_sc = f("f", "spa_sc");
    let unit_f_mid = f("f", "mid");
    main_phase.data.units.push(unit_e_por);
    main_phase.data.units.push(unit_f_spa_sc);
    main_phase.data.units.push(unit_f_mid);
    main_phase.data.orders.push(unit_e_por.hold());
    main_phase.data.orders.push(unit_f_spa_sc.move_to(p("por")));
    main_phase.data.orders.push(unit_f_mid.support_move(unit_f_spa_sc, p("por")));

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_e_por.dislodged_from(p("spa_sc")).retreat_to(p("spa_nc")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(retreat_phase.data.units.len(), 2);
    assert!(retreat_phase.data.units.contains(&f("f", "por")));
    assert!(retreat_phase.data.units.contains(&unit_f_mid));
}

/// 6.H.16. TEST CASE, CONTESTED FOR BOTH COASTS
/// If a coast is contested, the other is not available for retreat.
///
/// France:
///     F Mid-Atlantic Ocean - Spain(nc)
///     F Gascony - Spain(nc)
///     F Western Mediterranean Hold
///
/// Italy:
///     F Tunis Supports F Tyrrhenian Sea - Western Mediterranean
///     F Tyrrhenian Sea - Western Mediterranean
///
/// The French fleet in the Western Mediterranean cannot retreat to Spain(sc).
#[test]
fn test_datc_6_h_16() {
    let mut main_phase = Phase::new_spring_main(1901, 1);
    let mut context = PhaseContext::new();
    let unit_f_mid = f("f", "mid");
    let unit_f_gas = f("f", "gas");
    let mut unit_f_wes = f("f", "wes");
    let unit_i_tun = f("i", "tun");
    let unit_i_tyn = f("i", "tyn");
    main_phase.data.units.push(unit_f_mid);
    main_phase.data.units.push(unit_f_gas);
    main_phase.data.units.push(unit_f_wes);
    main_phase.data.units.push(unit_i_tun);
    main_phase.data.units.push(unit_i_tyn);
    main_phase.data.orders.push(unit_f_mid.move_to(p("spa_nc")));
    main_phase.data.orders.push(unit_f_gas.move_to(p("spa_nc")));
    main_phase.data.orders.push(unit_f_wes.hold());
    main_phase.data.orders.push(unit_i_tun.support_move(unit_i_tyn, p("wes")));
    main_phase.data.orders.push(unit_i_tyn.move_to(p("wes")));

    main_phase.close(&mut context);
    let mut retreat_phase = context.phases.pop().unwrap();
    retreat_phase.data.orders.clear();
    retreat_phase
        .data
        .orders
        .push(unit_f_wes.dislodged_from(p("tyn")).retreat_to(p("spa_sc")));
    Phase::resolve_orders_for_retreat_phase(&mut retreat_phase);
    assert_eq!(retreat_phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(retreat_phase.data.units.len(), 4);
    assert!(retreat_phase.data.units.contains(&unit_f_mid));
    assert!(retreat_phase.data.units.contains(&unit_f_gas));
    assert!(retreat_phase.data.units.contains(&unit_i_tun));
    assert!(retreat_phase.data.units.contains(&f("i", "wes")));
}
