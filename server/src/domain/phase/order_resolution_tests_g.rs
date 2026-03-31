//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6G]
//!
//! * 6.G. TEST CASES, CONVOYING TO ADJACENT PROVINCES
//!
//! [DATC_6G]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.G

use super::super::order::*;
use super::super::phase::order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.G.1. TEST CASE, TWO UNITS CAN SWAP PROVINCES BY CONVOY
/// The only way to swap two units, is by convoy.
///
/// England:
///     A Norway - Sweden
///     F Skagerrak Convoys A Norway - Sweden
///
/// Russia:
///     A Sweden - Norway
///
/// If explicit adjacent convoying is used (DPTG, see issue 4.A.3),
/// then it is just a head-to-head battle.
/// However, all rulebooks (which I prefer) allow that convoy intent is given
/// by a convoying fleet of same country.
/// So, swap should happen.
#[test]
fn test_datc_6_g_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_1_nwy = Unit::new_army(Power::England, p("nwy"));
    let unit_e_2_ska = Unit::new_fleet(Power::England, p("ska"));
    let unit_r_1_swe = Unit::new_army(Power::Russia, p("swe"));
    phase.data.orders.push(unit_e_1_nwy.move_to(p("swe")));
    phase.data.orders.push(unit_e_2_ska.convoy(unit_e_1_nwy, p("swe")));
    phase.data.orders.push(unit_r_1_swe.move_to(p("nwy")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
}

/// 6.G.2. TEST CASE, KIDNAPPING AN ARMY
/// Germany promised England to support to dislodge the Russian fleet in Sweden
/// and it promised Russia to support to dislodge the English army in Norway.
/// Instead, the joking German orders a convoy.
///
/// England:
///     A Norway - Sweden
///
/// Russia:
///     F Sweden - Norway
///
/// Germany:
///     F Skagerrak Convoys A Norway - Sweden
///
/// See issue 4.A.3. If the 1971 rulebook is used,
/// then the army in Norway is kidnapped and swaps with the army in Sweden.
/// In all other rulebooks (which I prever),
/// kidnapping is prevented and the armies fail to move.
#[test]
fn test_datc_6_g_2() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nwy = Unit::new_army(Power::England, p("nwy"));
    let unit_r_swe = Unit::new_fleet(Power::Russia, p("swe"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    phase.data.orders.push(unit_e_nwy.move_to(p("swe")));
    phase.data.orders.push(unit_r_swe.move_to(p("nwy")));
    phase.data.orders.push(unit_g_ska.convoy(unit_e_nwy, p("swe")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
}

/// 6.G.3. TEST CASE, AN UNWANTED DISRUPTED CONVOY TO ADJACENT PROVINCE
/// One can try to convoy an army unwanted with a fleet that is almost certainly dislodged.
/// However, this trick should not work.
///
/// France:
///     F Brest - English Channel
///     A Picardy - Belgium
///     A Burgundy Supports A Picardy - Belgium
///     F Mid-Atlantic Ocean Supports F Brest - English Channel
///
/// England:
///     F English Channel Convoys A Picardy - Belgium
///
/// See issue 4.A.3.
/// The 1982/2000/2023 rulebooks (which I prefer) will only use the convoy route if intent is clear.
/// The army in Picardy will successfully move by land route to Belgium.
/// In case of the 1971 rulebook it is less clear.
/// However, since no unit in Belgium moves in opposite direction the convoy should be ignored,
/// resulting in the same adjudication.
#[test]
fn test_datc_6_g_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_bre = Unit::new_fleet(Power::France, p("bre"));
    let unit_f_pic = Unit::new_army(Power::France, p("pic"));
    let unit_f_bur = Unit::new_army(Power::France, p("bur"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    phase.data.orders.push(unit_f_bre.move_to(p("eng")));
    phase.data.orders.push(unit_f_pic.move_to(p("bel")));
    phase.data.orders.push(unit_f_bur.support_move(unit_f_pic, p("bel")));
    phase.data.orders.push(unit_f_mao.support_move(unit_f_bre, p("eng")));
    phase.data.orders.push(unit_e_eng.convoy(unit_f_pic, p("bel")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Dislodged);
}

/// 6.G.4. TEST CASE, AN UNWANTED DISRUPTED CONVOY TO ADJACENT PROVINCE AND OPPOSITE MOVE
/// In the situation of the previous test case,
/// it was rather clear that the army didn't want to take the convoy.
/// But what if there is an army moving in opposite direction?
///
/// France:
///     F Brest - English Channel
///     A Picardy - Belgium
///     A Burgundy Supports A Picardy - Belgium
///     F Mid-Atlantic Ocean Supports F Brest - English Channel
///
/// England:
///     F English Channel Convoys A Picardy - Belgium
///     A Belgium - Picardy
///
/// See issue 4.A.3. In case of the 1971 rules,
/// it is not directly clear whether the French army in Picardy will take the land route.
/// However, if unwanted convoys are prevented as much as possible,
/// it will not take the convoy if it is disrupted.
/// So, the move of the army in Picardy will succeed.
/// With the 1982/2000/2023 rulebooks (which I prefer) and with explicit adjacent convoying,
/// kidnapping is prevented and the French army will successfully move.
#[test]
fn test_datc_6_g_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_bre = Unit::new_fleet(Power::France, p("bre"));
    let unit_f_pic = Unit::new_army(Power::France, p("pic"));
    let unit_f_bur = Unit::new_army(Power::France, p("bur"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    let unit_e_bel = Unit::new_army(Power::England, p("bel"));
    phase.data.orders.push(unit_f_bre.move_to(p("eng")));
    phase.data.orders.push(unit_f_pic.move_to(p("bel")));
    phase.data.orders.push(unit_f_bur.support_move(unit_f_pic, p("bel")));
    phase.data.orders.push(unit_f_mao.support_move(unit_f_bre, p("eng")));
    phase.data.orders.push(unit_e_eng.convoy(unit_f_pic, p("bel")));
    phase.data.orders.push(unit_e_bel.move_to(p("pic")));
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Dislodged);
}

/// 6.G.5. TEST CASE, SWAPPING WITH MULTIPLE FLEETS WITH ONE OWN FLEET
/// One fleet is sufficient to show the intent to convoy.
///
/// Italy:
/// A Rome - Apulia
/// F Tyrrhenian Sea Convoys A Apulia - Rome
///
/// Turkey:
/// A Apulia - Rome
/// F Ionian Sea Convoys A Apulia - Rome
/// If explicit adjacent convoying is used (DPTG, see issue 4.A.3), then it is just a head-to-head battle. However, all rulebooks (which I prefer) allow that convoy intent is given by a convoying fleet of same country. So, the swap should happen.
#[test]
fn test_datc_6_g_5() {}

/// 6.G.6. TEST CASE, SWAPPING WITH UNINTENDED INTENT
/// The intent is questionable.
///
/// England:
/// A Liverpool - Edinburgh
/// F English Channel Convoys A Liverpool - Edinburgh
///
/// Germany:
/// A Edinburgh - Liverpool
///
/// France:
/// F Irish Sea Hold
/// F North Sea Hold
///
/// Russia:
/// F Norwegian Sea Convoys A Liverpool - Edinburgh
/// F North Atlantic Ocean Convoys A Liverpool - Edinburgh
/// Here England intended to convoy via the French fleets in the Irish Sea and the North Sea. However, the French did not order the convoy. The alternative route with the Russian fleets was unintended. The English fleet in the English Channel (with the convoy order) is not part of this alternative route with the Russian fleets.
///
/// See issue 4.A.3.
///
/// If the 1971 rules are used, the intent is not important and the units are swapped.
///
/// In case of the 1982/2000/2023 rulebooks (which I prefer) England still intents to convoy and the armies should swap.
///
/// When explicit adjacent convoying is used (DPTG), then the English army did not receive an order to move by convoy. So, it is just a head-to-head battle and both the army in Edinburgh and Liverpool will not move.
#[test]
fn test_datc_6_g_6() {}

/// 6.G.7. TEST CASE, SWAPPING WITH ILLEGAL INTENT
/// Can the intent be made clear with an impossible order?
///
/// England:
/// F Skagerrak Convoys A Sweden - Norway
/// F Norway - Sweden
///
/// Russia:
/// A Sweden - Norway
/// F Gulf of Bothnia Convoys A Sweden - Norway
/// See issue 4.A.3 and 4.E.1.
///
/// In case the 1971 rules are used, the intent is not important and the units in Norway and Sweden swap.
///
/// With the 2023 rules (which I prefer) impossible orders are ignored. Also, with modern webbased adjudicators, impossible orders cannot be given at all. With this, there is no intent to convoy and the units in Norway and Sweden fail to move.
///
/// If explicit adjacent convoying is used (DPTG) there is also no convoy and none of the units move.
#[test]
fn test_datc_6_g_7() {}

/// 6.G.8. TEST CASE, EXPLICIT CONVOY THAT ISN'T THERE
/// What to do when a unit is explicitly ordered to move via convoy and the convoy is not there?
///
/// France:
/// A Belgium - Holland via convoy
///
/// England:
/// F North Sea - Helgoland Bight
/// A Holland - Kiel
/// The French army in Belgium intended to move convoyed with the English fleet in the North Sea. But England changed its plans.
///
/// See issue 4.A.3.
///
/// In case of 1971 or 1982 rulebook, this test case not applicable, because they don't have the notion of 'via convoy'.
///
/// For the 2000/2023 rulebook and the DPTG, the question is whether the land route should be used as "fallback".
///
/// As discussed in the issue, I don't prefer fallback anymore.
#[test]
fn test_datc_6_g_8() {}

/// 6.G.9. TEST CASE, SWAPPED OR DISLODGED?
/// In the following situation the English army in Norway will end in all cases in Sweden. But whether it is convoyed or not has effect on the Russian army. In case of convoy the Russian army ends in Norway and in case of a land route the Russian army is dislodged (see issue 4.A.3).
///
/// England:
/// A Norway - Sweden
/// F Skagerrak Convoys A Norway - Sweden
/// F Finland Supports A Norway - Sweden
///
/// Russia:
/// A Sweden - Norway
/// If played according to the DPTG, then an army is only convoyed to an adjacent province if it is tagged with "via convoy". This means that the Russian army in Sweden is dislodged by the army from Norway.
///
/// If played according to any of the rulebooks (which I prefer) then the move of Norway is via convoy and the armies swap.
#[test]
fn test_datc_6_g_9() {}

/// 6.G.10. TEST CASE, SWAPPED OR AN HEAD-TO-HEAD BATTLE?
/// Can a dislodged unit have effect on the attacker's area, when the attacker moved by convoy?
///
/// England:
/// A Norway - Sweden via convoy
/// F Denmark Supports A Norway - Sweden
/// F Finland Supports A Norway - Sweden
///
/// Germany:
/// F Skagerrak Convoys A Norway - Sweden
///
/// Russia:
/// A Sweden - Norway
/// F Barents Sea Supports A Sweden - Norway
///
/// France:
/// F Norwegian Sea - Norway
/// F North Sea Supports F Norwegian Sea - Norway
/// Since England ordered the army in Norway to move explicitly via convoy and the army in Sweden is moving in opposite direction, there is no head-to-head battle. It is clear that the army in Norway will dislodge the Russian army in Sweden. Since the strength of three is in all cases the strongest force.
///
/// The army in Sweden will not advance to Norway, because it cannot beat the force in the Norwegian Sea. It will be dislodged by the army from Norway.
///
/// The more interesting question is whether the French fleet in the Norwegian Sea is bounced by the Russian army from Sweden. This depends on the interpretation of issue 4.A.7. If the rulebook is taken literally (choice a), then a dislodged unit cannot bounce a unit in the area where the attacker came from. This would mean that the move of the fleet in the Norwegian Sea succeeds. However, if choice b is taken (which I prefer), then a bounce is still possible, when there is no head-to-head battle. So, the fleet in the Norwegian Sea will fail to move.
#[test]
fn test_datc_6_g_10() {}

/// 6.G.11. TEST CASE, A CONVOY TO AN ADJACENT PROVINCE WITH A PARADOX
/// In this case the convoy route is available when the land route is chosen and the convoy route is not available when the convoy route is chosen.
///
/// England:
/// F Norway Supports F North Sea - Skagerrak
/// F North Sea - Skagerrak
///
/// Russia:
/// A Sweden - Norway
/// F Skagerrak Convoys A Sweden - Norway
/// F Barents Sea Supports A Sweden - Norway
/// See issue 4.A.2 and 4.A.3.
///
/// In case of the 1971 rulebook the move from Sweden to Norway is not a convoy (because Norway is not moving in opposite direction) and the English fleet in Norway is dislodged and the fleet in Skagerrak will not be dislodged.
///
/// In case of the 1982/2000/2023 rulebook, the question arises whether the land route is the fallback of the convoy route. If not, then this is just the most simple convoy paradox. The fleet in Skagerrak is dislodged and the army in Sweden will not advance.
///
/// In case fallback is possible, then the convoy is available when the land route is taken, but not otherwise.
///
/// I prefer no fallback. That means that according to these preferences the fleet in the North Sea will dislodge the Russian fleet in Skagerrak and the army in Sweden will not advance.
#[test]
fn test_datc_6_g_11() {}

/// 6.G.12. TEST CASE, SWAPPING TWO UNITS WITH TWO CONVOYS
/// Of course, two armies can also swap by when they are both convoyed.
///
/// England:
/// A Liverpool - Edinburgh via convoy
/// F North Atlantic Ocean Convoys A Liverpool - Edinburgh
/// F Norwegian Sea Convoys A Liverpool - Edinburgh
///
/// Germany:
/// A Edinburgh - Liverpool via convoy
/// F North Sea Convoys A Edinburgh - Liverpool
/// F English Channel Convoys A Edinburgh - Liverpool
/// F Irish Sea Convoys A Edinburgh - Liverpool
/// The armies in Liverpool and Edinburgh are swapped.
#[test]
fn test_datc_6_g_12() {}

/// 6.G.13. TEST CASE, SUPPORT CUT ON ATTACK ON ITSELF VIA CONVOY
/// If a unit is attacked by a supported unit, it is not possible to prevent dislodgement by trying to cut the support. But what, if a move is attempted via a convoy?
///
/// Austria:
/// F Adriatic Sea Convoys A Trieste - Venice
/// A Trieste - Venice via convoy
///
/// Italy:
/// A Venice Supports F Albania - Trieste
/// F Albania - Trieste
/// First it should be mentioned that if for issue 4.A.3 the 1971 rulebook is chosen, the move from Trieste to Venice is just a move over land (because Venice does not move in opposite direction). In that case, the support of Venice will not be cut as normal.
///
/// For the 1982/2000/2023 rulebooks the attack is via convoy and it should be decided whether the Austrian attack is considered to be coming from Trieste or from the Adriatic Sea. If it comes from Trieste, the support in Venice is not cut and the army in Trieste is dislodged by the fleet in Albania. If the Austrian attack is considered to be coming from the Adriatic Sea, then the support is cut and the army in Trieste will not be dislodged. See also issue 4.A.4.
///
/// First of all, I prefer the 2023 rules for adjacent convoying, meaning that the move from Trieste uses the convoy. Furthermore, I think that the two Italian units are still stronger than the army in Trieste. Therefore, I prefer that the support in Venice is not cut and that the army in Trieste is dislodged by the fleet in Albania.
#[test]
fn test_datc_6_g_13() {}

/// 6.G.14. TEST CASE, BOUNCE BY CONVOY TO ADJACENT PROVINCE
/// Similar to test case 6.G.10, but now the other unit is taking the convoy.
///
/// England:
/// A Norway - Sweden
/// F Denmark Supports A Norway - Sweden
/// F Finland Supports A Norway - Sweden
///
/// France:
/// F Norwegian Sea - Norway
/// F North Sea Supports F Norwegian Sea - Norway
///
/// Germany:
/// F Skagerrak Convoys A Sweden - Norway
///
/// Russia:
/// A Sweden - Norway via convoy
/// F Barents Sea Supports A Sweden - Norway
/// Again, the army in Sweden is bounced by the fleet in the Norwegian Sea. The army in Norway will move to Sweden and dislodge the Russian army.
///
/// The final destination of the fleet in the Norwegian Sea depends on how issue 4.A.7 is resolved. If choice a is taken, then the fleet advances to Norway, but if choice b is taken (which I prefer) the fleet bounces and stays in the Norwegian Sea.
#[test]
fn test_datc_6_g_14() {}

/// 6.G.15. TEST CASE, BOUNCE AND DISLODGE WITH DOUBLE CONVOY
/// Similar to test case 6.G.10, but now both units use a convoy and without some support.
///
/// England:
/// F North Sea Convoys A London - Belgium
/// A Holland Supports A London - Belgium
/// A Yorkshire - London
/// A London - Belgium via convoy
///
/// France:
/// F English Channel Convoys A Belgium - London
/// A Belgium - London via convoy
/// The French army in Belgium is bounced by the army from Yorkshire. The army in London move to Belgium, dislodging the unit there.
///
/// The final destination of the army in the Yorkshire depends on how issue 4.A.7 is resolved. If choice a is taken, then the army advances to London, but if choice b is taken (which I prefer) the army bounces and stays in Yorkshire.
#[test]
fn test_datc_6_g_15() {}

/// 6.G.16. TEST CASE, THE TWO UNIT IN ONE AREA BUG, MOVING BY CONVOY
/// If the adjudicator is not correctly implemented, this may lead to a resolution where two units end up in the same area.
///
/// England:
/// A Norway - Sweden
/// A Denmark Supports A Norway - Sweden
/// F Baltic Sea Supports A Norway - Sweden
/// F North Sea - Norway
///
/// Russia:
/// A Sweden - Norway via convoy
/// F Skagerrak Convoys A Sweden - Norway
/// F Norwegian Sea Supports A Sweden - Norway
/// See decision details 5.B.6. If the 'PREVENT STRENGTH' is incorrectly implemented, due to the fact that it does not take into account that the 'PREVENT STRENGTH' is only zero when the unit is engaged in a head-to-head battle, then this goes wrong in this test case. The 'PREVENT STRENGTH' of Sweden would be zero, because the opposing unit in Norway successfully moves. Since, this strength would be zero, the fleet in the North Sea would move to Norway. However, although the 'PREVENT STRENGTH' is zero, the army in Sweden would also move to Norway. So, the final result would contain two units that successfully moved to Norway.
///
/// Of course, this is incorrect. Norway will indeed successfully move to Sweden while the army in Sweden ends in Norway, because it is stronger than the fleet in the North Sea. This fleet will stay in the North Sea.
#[test]
fn test_datc_6_g_16() {}

/// 6.G.17. TEST CASE, THE TWO UNIT IN ONE AREA BUG, MOVING OVER LAND
/// Similar to the previous test case, but now the other unit moves by convoy.
///
/// England:
/// A Norway - Sweden via convoy
/// A Denmark Supports A Norway - Sweden
/// F Baltic Sea Supports A Norway - Sweden
/// F Skagerrak Convoys A Norway - Sweden
/// F North Sea - Norway
///
/// Russia:
/// A Sweden - Norway
/// F Norwegian Sea Supports A Sweden - Norway
/// Sweden and Norway are swapped, while the fleet in the North Sea will bounce.
#[test]
fn test_datc_6_g_17() {}

/// 6.G.18. TEST CASE, THE TWO UNIT IN ONE AREA BUG, WITH DOUBLE CONVOY
/// Similar to the previous test case, but now both units move by convoy.
///
/// England:
/// F North Sea Convoys A London - Belgium
/// A Holland Supports A London - Belgium
/// A Yorkshire - London
/// A London - Belgium
/// A Ruhr Supports A London - Belgium
///
/// France:
/// F English Channel Convoys A Belgium - London
/// A Belgium - London
/// A Wales Supports A Belgium - London
/// Belgium and London are swapped, while the army in Yorkshire fails to move to London.
#[test]
fn test_datc_6_g_18() {}

/// 6.G.19. TEST CASE, SWAPPING WITH INTENT OF UNNECESSARY CONVOY
/// Can the intent made clear by the order of a fleet that is not necessary?
///
/// France:
/// A Marseilles - Spain
/// F Western Mediterranean Convoys A Marseilles - Spain
///
/// Italy:
/// F Gulf of Lyon Convoys A Marseilles - Spain
/// A Spain - Marseilles
/// See issue 4.A.3 and 4.E.1.
///
/// In case the 1971 rules are used, the intent is not important and the units in Marseilles and Spain swap.
///
/// The point of interest is that there is a convoy route from Marseilles, Gulf of Lyon, Western Mediterranean to Spain. However, the fleet in Western Mediterranean is not necessary for this convoy and not necessary for any other convoy route. Therefore, this order should be considered illegal. Webbased adjudicators should not give this order as an option.
///
/// With the 2023 rules (which I prefer) illegal orders are ignored. The fleet in Gulf of Lyon is foreign and foreign units cannot express intent. With this, there is no intent to convoy and the units in Marseilles and Spain fail to move.
///
/// If explicit adjacent convoying is used (DPTG) there is also no convoy and none of the units move.
#[test]
fn test_datc_6_g_19() {}

/// 6.G.20. TEST CASE, EXPLICIT CONVOY TO ADJACENT PROVINCE DISRUPTED
/// If a move to adjacent province was explicit via convoy, and the convoy is disrupted, should it fall back to the land route?
///
/// France:
/// F Brest - English Channel
/// A Picardy - Belgium via Convoy
/// A Burgundy Supports A Picardy - Belgium
/// F Mid-Atlantic Ocean Supports F Brest - English Channel
///
/// England:
/// F English Channel Convoys A Picardy - Belgium
/// This situation is not applicable for the 1971 and 1982 rulebooks, because they don't have the notion of 'via convoy'.
///
/// For the 2000/2023 rulebook the question arises whether the army in Picardy will fall back to the land route, since the convoy route is disrupted. See issue 4.A.3.
///
/// I don't prefer the fallback anymore. So, the move of Picardy fails.
#[test]
fn test_datc_6_g_20() {}
