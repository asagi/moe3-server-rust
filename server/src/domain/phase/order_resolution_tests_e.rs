//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6E]
//!
//! * 6.A. 6.E. TEST CASES, HEAD-TO-HEAD BATTLES AND BELEAGUERED GARRISON
//!
//! [DATC_6E]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.E

use super::super::order::*;
use super::super::phase::order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.E.1. TEST CASE, DISLODGED UNIT HAS NO EFFECT ON ATTACKER'S AREA
/// An army can follow.
///
/// Germany:
/// A Berlin - Prussia
/// F Kiel - Berlin
/// A Silesia Supports A Berlin - Prussia
///
/// Russia:
/// A Prussia - Berlin
/// The army in Kiel will move to Berlin.
#[test]
fn test_datc_6_e_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Germany, p("ber"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("kie"));
    let unit_g_3 = Unit::new_army(Power::Germany, p("sil"));
    let unit_r_1 = Unit::new_army(Power::Russia, p("pru"));
    phase.data.orders.push(unit_g_1.move_to(p("pru")));
    phase.data.orders.push(unit_g_2.move_to(p("ber")));
    phase.data.orders.push(unit_g_3.support_move(unit_g_1, p("pru")));
    phase.data.orders.push(unit_r_1.move_to(p("ber")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
}

/// 6.E.2. TEST CASE, NO SELF DISLODGEMENT IN HEAD-TO-HEAD BATTLE
/// Self dislodgement is not allowed. This also counts for head-to-head battles.
///
/// Germany:
/// A Berlin - Kiel
/// F Kiel - Berlin
/// A Munich Supports A Berlin - Kiel
/// No unit will move.
#[test]
fn test_datc_6_e_2() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Germany, p("ber"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("kie"));
    let unit_g_3 = Unit::new_army(Power::Germany, p("mun"));
    phase.data.orders.push(unit_g_1.move_to(p("kie")));
    phase.data.orders.push(unit_g_2.move_to(p("ber")));
    phase.data.orders.push(unit_g_3.support_move(unit_g_1, p("kie")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
}

/// 6.E.3. TEST CASE, NO HELP IN DISLODGING OWN UNIT
/// It is not possible to help a foreign power dislodge own unit in a head-to-head battle.
///
/// Germany:
/// A Berlin - Kiel
/// A Munich Supports F Kiel - Berlin
///
/// England:
/// F Kiel - Berlin
/// No unit will move.
#[test]
fn test_datc_6_e_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_army(Power::Germany, p("ber"));
    let unit_g_2 = Unit::new_army(Power::Germany, p("mun"));
    let unit_e_1 = Unit::new_fleet(Power::England, p("kie"));
    phase.data.orders.push(unit_g_1.move_to(p("kie")));
    phase.data.orders.push(unit_g_2.support_move(unit_e_1, p("ber")));
    phase.data.orders.push(unit_e_1.move_to(p("ber")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.E.4. TEST CASE, NON-DISLODGED LOSER STILL HAS EFFECT
/// If in an unbalanced head-to-head battle the loser is not dislodged,
/// it still has an effect on the area of the attacker.
///
/// Germany:
/// F Holland - North Sea
/// F Helgoland Bight Supports F Holland - North Sea
/// F Skagerrak Supports F Holland - North Sea
///
/// France:
/// F North Sea - Holland
/// F Belgium Supports F North Sea - Holland
///
/// England:
/// F Edinburgh Supports F Norwegian Sea - North Sea
/// F Yorkshire Supports F Norwegian Sea - North Sea
/// F Norwegian Sea - North Sea
///
/// Austria:
/// A Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
/// The French fleet in the North Sea is not dislodged due to the beleaguered garrison.
/// Therefore, the Austrian army in Ruhr will not move to Holland.
#[test]
fn test_datc_6_e_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_3 = Unit::new_fleet(Power::Germany, p("ska"));
    let unit_f_1 = Unit::new_fleet(Power::France, p("nth"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("bel"));
    let unit_e_1 = Unit::new_fleet(Power::England, p("edi"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("yor"));
    let unit_e_3 = Unit::new_fleet(Power::England, p("nwg"));
    let unit_a_1 = Unit::new_army(Power::Austria, p("kie"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("ruh"));
    phase.data.orders.push(unit_g_1.move_to(p("nth")));
    phase.data.orders.push(unit_g_2.support_move(unit_g_1, p("nth")));
    phase.data.orders.push(unit_g_3.support_move(unit_g_1, p("nth")));
    phase.data.orders.push(unit_f_1.move_to(p("hol")));
    phase.data.orders.push(unit_f_2.support_move(unit_f_1, p("hol")));
    phase.data.orders.push(unit_e_1.support_move(unit_e_3, p("nth")));
    phase.data.orders.push(unit_e_2.support_move(unit_e_3, p("nth")));
    phase.data.orders.push(unit_e_3.move_to(p("nth")));
    phase.data.orders.push(unit_a_1.support_move(unit_a_2, p("hol")));
    phase.data.orders.push(unit_a_2.move_to(p("hol")));
    resolve_orders_for_order_phase(&mut phase);
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
}

/// 6.E.5. TEST CASE, LOSER DISLODGED BY ANOTHER ARMY STILL HAS EFFECT
/// If in an unbalanced head-to-head battle the loser is dislodged by a unit not part of the head-to-head battle,
/// the loser still has an effect on the area of the winner of the head-to-head battle.
///
/// Germany:
/// F Holland - North Sea
/// F Helgoland Bight Supports F Holland - North Sea
/// F Skagerrak Supports F Holland - North Sea
///
/// France:
/// F North Sea - Holland
/// F Belgium Supports F North Sea - Holland
///
/// England:
/// F Edinburgh Supports F Norwegian Sea - North Sea
/// F Yorkshire Supports F Norwegian Sea - North Sea
/// F Norwegian Sea - North Sea
/// F London Supports F Norwegian Sea - North Sea
///
/// Austria:
/// A Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
/// The French fleet in the North Sea is dislodged but not by the German fleet in Holland.
/// Therefore, the French fleet can still prevent that the Austrian army in Ruhr will move to Holland.
/// So, the Austrian move in Ruhr fails and the German fleet in Holland is not dislodged.
#[test]
fn test_datc_6_e_5() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_3 = Unit::new_fleet(Power::Germany, p("ska"));
    let unit_f_1 = Unit::new_fleet(Power::France, p("nth"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("bel"));
    let unit_e_1 = Unit::new_fleet(Power::England, p("edi"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("yor"));
    let unit_e_3 = Unit::new_fleet(Power::England, p("nwg"));
    let unit_e_4 = Unit::new_fleet(Power::England, p("lon"));
    let unit_a_1 = Unit::new_army(Power::Austria, p("kie"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("ruh"));
    phase.data.orders.push(unit_g_1.move_to(p("nth")));
    phase.data.orders.push(unit_g_2.support_move(unit_g_1, p("nth")));
    phase.data.orders.push(unit_g_3.support_move(unit_g_1, p("nth")));
    phase.data.orders.push(unit_f_1.move_to(p("hol")));
    phase.data.orders.push(unit_f_2.support_move(unit_f_1, p("hol")));
    phase.data.orders.push(unit_e_1.support_move(unit_e_3, p("nth")));
    phase.data.orders.push(unit_e_2.support_move(unit_e_3, p("nth")));
    phase.data.orders.push(unit_e_3.move_to(p("nth")));
    phase.data.orders.push(unit_e_4.support_move(unit_e_3, p("nth")));
    phase.data.orders.push(unit_a_1.support_move(unit_a_2, p("hol")));
    phase.data.orders.push(unit_a_2.move_to(p("hol")));
    resolve_orders_for_order_phase(&mut phase);
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
}

/// 6.E.6. TEST CASE, NOT DISLODGE BECAUSE OF OWN SUPPORT STILL HAS EFFECT
/// If in an unbalanced head-to-head battle the loser is not dislodged
/// because the winner had help of a unit of the loser,
/// the loser still has an effect on the area of the winner.
///
/// Germany:
/// F Holland - North Sea
/// F Helgoland Bight Supports F Holland - North Sea
///
/// France:
/// F North Sea - Holland
/// F Belgium Supports F North Sea - Holland
/// F English Channel Supports F Holland - North Sea
///
/// Austria:
/// A Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
/// Although the German force from Holland to North Sea is one larger
/// than the French force from North Sea to Holland,
/// the French fleet in the North Sea is not dislodged,
/// because one of the supports on the German movement is French.
/// Therefore, the Austrian army in Ruhr will not move to Holland.
#[test]
fn test_datc_6_e_6() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_f_1 = Unit::new_fleet(Power::France, p("nth"));
    let unit_f_2 = Unit::new_fleet(Power::France, p("bel"));
    let unit_f_3 = Unit::new_fleet(Power::France, p("eng"));
    let unit_a_1 = Unit::new_army(Power::Austria, p("kie"));
    let unit_a_2 = Unit::new_army(Power::Austria, p("ruh"));
    phase.data.orders.push(unit_g_1.move_to(p("nth")));
    phase.data.orders.push(unit_g_2.support_move(unit_g_1, p("nth")));
    phase.data.orders.push(unit_f_1.move_to(p("hol")));
    phase.data.orders.push(unit_f_2.support_move(unit_f_1, p("hol")));
    phase.data.orders.push(unit_f_3.support_move(unit_g_1, p("nth")));
    phase.data.orders.push(unit_a_1.support_move(unit_a_2, p("hol")));
    phase.data.orders.push(unit_a_2.move_to(p("hol")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
}

/// 6.E.7. TEST CASE, NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON
/// An attempt at self dislodgement can be combined with a beleaguered garrison.
/// Such self dislodgment is still not possible.
///
/// England:
/// F North Sea Hold
/// F Yorkshire Supports F Norway - North Sea
///
/// Germany:
/// F Holland Supports F Helgoland Bight - North Sea
/// F Helgoland Bight - North Sea
///
/// Russia:
/// F Skagerrak Supports F Norway - North Sea
/// F Norway - North Sea
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
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("yor"));
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("ska"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("nwy"));
    phase.data.orders.push(unit_e_1.hold());
    phase.data.orders.push(unit_e_2.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_g_1.support_move(unit_g_2, p("nth")));
    phase.data.orders.push(unit_g_2.move_to(p("nth")));
    phase.data.orders.push(unit_r_1.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_r_2.move_to(p("nth")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
}

/// 6.E.8. TEST CASE, NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON AND HEAD-TO-HEAD BATTLE
/// Similar to the previous test case, but now the beleaguered fleet is also engaged in a head-to-head battle.
///
/// England:
/// F North Sea - Norway
/// F Yorkshire Supports F Norway - North Sea
///
/// Germany:
/// F Holland Supports F Helgoland Bight - North Sea
/// F Helgoland Bight - North Sea
///
/// Russia:
/// F Skagerrak Supports F Norway - North Sea
/// F Norway - North Sea
/// Again, none of the fleets move.
#[test]
fn test_datc_6_e_8() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("yor"));
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("ska"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("nwy"));
    phase.data.orders.push(unit_e_1.move_to(p("nwy")));
    phase.data.orders.push(unit_e_2.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_g_1.support_move(unit_g_2, p("nth")));
    phase.data.orders.push(unit_g_2.move_to(p("nth")));
    phase.data.orders.push(unit_r_1.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_r_2.move_to(p("nth")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
}

/// 6.E.9. TEST CASE, ALMOST SELF DISLODGEMENT WITH BELEAGUERED GARRISON
/// Similar to the previous test case, but now the beleaguered fleet is moving away.
///
/// England:
/// F North Sea - Norwegian Sea
/// F Yorkshire Supports F Norway - North Sea
///
/// Germany:
/// F Holland Supports F Helgoland Bight - North Sea
/// F Helgoland Bight - North Sea
///
/// Russia:
/// F Skagerrak Supports F Norway - North Sea
/// F Norway - North Sea
/// Both the fleet in the North Sea and the fleet in Norway move.
#[test]
fn test_datc_6_e_9() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("yor"));
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("ska"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("nwy"));
    phase.data.orders.push(unit_e_1.move_to(p("nwg")));
    phase.data.orders.push(unit_e_2.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_g_1.support_move(unit_g_2, p("nth")));
    phase.data.orders.push(unit_g_2.move_to(p("nth")));
    phase.data.orders.push(unit_r_1.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_r_2.move_to(p("nth")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
}

/// 6.E.10. TEST CASE, ALMOST CIRCULAR MOVEMENT WITH NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON
/// Similar to the previous test case, but now the beleaguered fleet is in circular movement with the weaker attacker.
/// So, the circular movement fails.
///
/// England:
/// F North Sea - Denmark
/// F Yorkshire Supports F Norway - North Sea
///
/// Germany:
/// F Holland Supports F Helgoland Bight - North Sea
/// F Helgoland Bight - North Sea
/// F Denmark - Helgoland Bight
///
/// Russia:
/// F Skagerrak Supports F Norway - North Sea
/// F Norway - North Sea
/// There is no movement of fleets.
#[test]
fn test_datc_6_e_10() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1 = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_2 = Unit::new_fleet(Power::England, p("yor"));
    let unit_g_1 = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_2 = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_3 = Unit::new_fleet(Power::Germany, p("den"));
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("ska"));
    let unit_r_2 = Unit::new_fleet(Power::Russia, p("nwy"));
    phase.data.orders.push(unit_e_1.move_to(p("den")));
    phase.data.orders.push(unit_e_2.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_g_1.support_move(unit_g_2, p("nth")));
    phase.data.orders.push(unit_g_2.move_to(p("nth")));
    phase.data.orders.push(unit_g_3.move_to(p("hel")));
    phase.data.orders.push(unit_r_1.support_move(unit_r_2, p("nth")));
    phase.data.orders.push(unit_r_2.move_to(p("nth")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
}

/// 6.E.11. TEST CASE, NO SELF DISLODGEMENT WITH BELEAGUERED GARRISON, UNIT SWAP WITH ADJACENT CONVOYING AND TWO COASTS
/// Similar to the previous test case, but now the beleaguered fleet is in a unit swap with the stronger attacker. So, the unit swap succeeds. To make the situation more complex, the swap is on an area with two coasts.
///
/// France:
/// A Spain - Portugal via convoy
/// F Mid-Atlantic Ocean Convoys A Spain - Portugal
/// F Gulf of Lyon Supports F Portugal - Spain(nc)
///
/// Germany:
/// A Marseilles Supports A Gascony - Spain
/// A Gascony - Spain
///
/// Italy:
/// F Portugal - Spain(nc)
/// F Western Mediterranean Supports F Portugal - Spain(nc)
/// The unit swap succeeds. Note that due to the success of the swap, there is no beleaguered garrison anymore.
#[test]
fn test_datc_6_e_11() {}

/// 6.E.12. TEST CASE, SUPPORT ON ATTACK ON OWN UNIT CAN BE USED FOR OTHER MEANS
/// A support on an attack on your own unit still has an effect. It can prevent that another army will dislodge the unit.
///
/// Austria:
/// A Budapest - Rumania
/// A Serbia Supports A Vienna - Budapest
///
/// Italy:
/// A Vienna - Budapest
///
/// Russia:
/// A Galicia - Budapest
/// A Rumania Supports A Galicia - Budapest
/// The support of Serbia on the Italian army prevents that the Russian army in Galicia will advance. No army will move.
#[test]
fn test_datc_6_e_12() {}

/// 6.E.13. TEST CASE, THREE WAY BELEAGUERED GARRISON
/// In a beleaguered garrison from three sides, the adjudicator may not let two attacks fail and then let the third succeed.
///
/// England:
/// F Edinburgh Supports F Yorkshire - North Sea
/// F Yorkshire - North Sea
///
/// France:
/// F Belgium - North Sea
/// F English Channel Supports F Belgium - North Sea
///
/// Germany:
/// F North Sea Hold
///
/// Russia:
/// F Norwegian Sea - North Sea
/// F Norway Supports F Norwegian Sea - North Sea
/// None of the fleets move. The German fleet in the North Sea is not dislodged.
#[test]
fn test_datc_6_e_13() {}

/// 6.E.14. TEST CASE, ILLEGAL HEAD-TO-HEAD BATTLE CAN STILL DEFEND
/// If in a head-to-head battle, one of the units makes an illegal move, then that unit still has the possibility to defend against attacks with strength of one.
///
/// England:
/// A Liverpool - Edinburgh
///
/// Russia:
/// F Edinburgh - Liverpool
/// The move of the Russian fleet is illegal, but can still prevent the English army from entering Edinburgh. So, none of the units move.
#[test]
fn test_datc_6_e_14() {}

/// 6.E.15. TEST CASE, THE FRIENDLY HEAD-TO-HEAD BATTLE
/// In this case each unit in the head-to-head battle prevents that the other unit from being dislodged.
///
/// England:
/// F Holland Supports A Ruhr - Kiel
/// A Ruhr - Kiel
///
/// France:
/// A Kiel - Berlin
/// A Munich Supports A Kiel - Berlin
/// A Silesia Supports A Kiel - Berlin
///
/// Germany:
/// A Berlin - Kiel
/// F Denmark Supports A Berlin - Kiel
/// F Helgoland Bight Supports A Berlin - Kiel
///
/// Russia:
/// F Baltic Sea Supports A Prussia - Berlin
/// A Prussia - Berlin
/// None of the moves succeeds. This case is especially difficult for sequence based adjudicators. They will start adjudicating the head-to-head battle and continue to adjudicate the attack on one of the units which is part of the head-to-head battle. In this process, one of the sides of the head-to-head battle might be cancelled out.
#[test]
fn test_datc_6_e_15() {}
