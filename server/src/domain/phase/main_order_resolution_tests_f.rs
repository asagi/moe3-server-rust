//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6F]
//!
//! * 6.F. TEST CASES, CONVOYS
//!
//! [DATC_6F]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.F

use super::super::order::*;
use super::super::phase::main_order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

/// 6.F.1. TEST CASE, NO CONVOY IN COASTAL AREAS
/// A fleet in a coastal area may not convoy.
///
/// Turkey:
///     A Greece - Sevastopol
///     F Aegean Sea Convoys A Greece - Sevastopol
///     F Constantinople Convoys A Greece - Sevastopol
///     F Black Sea Convoys A Greece - Sevastopol
///
/// The convoy in Constantinople is not possible.
/// So, the army in Greece will not move to Sevastopol.
#[test]
fn test_datc_6_f_1() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_t_gre = Unit::new_army(Power::Turkey, p("gre"));
    let unit_t_aeg = Unit::new_fleet(Power::Turkey, p("aeg"));
    let unit_t_con = Unit::new_fleet(Power::Turkey, p("con"));
    let unit_t_bla = Unit::new_fleet(Power::Turkey, p("bla"));
    phase.data.orders.push(unit_t_gre.move_to(p("sev")));
    phase.data.orders.push(unit_t_aeg.convoy(unit_t_gre, p("sev")));
    phase.data.orders.push(unit_t_con.convoy(unit_t_gre, p("sev")));
    phase.data.orders.push(unit_t_bla.convoy(unit_t_gre, p("sev")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Invalid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Invalid);
}

/// 6.F.2. TEST CASE, AN ARMY BEING CONVOYED CAN BOUNCE AS NORMAL
/// Armies being convoyed bounce on other units just as armies that are not being convoyed.
///
/// England:
///     F English Channel Convoys A London - Brest
///     A London - Brest
///
/// France:
///     A Paris - Brest
///
/// The English army in London bounces on the French army in Paris.
/// Both units do not move.
#[test]
fn test_datc_6_f_2() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_f_par = Unit::new_army(Power::France, p("par"));
    phase.data.orders.push(unit_e_eng.convoy(unit_e_lon, p("bre")));
    phase.data.orders.push(unit_e_lon.move_to(p("bre")));
    phase.data.orders.push(unit_f_par.move_to(p("bre")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.F.3. TEST CASE, AN ARMY BEING CONVOYED CAN RECEIVE SUPPORT
/// Armies being convoyed can receive support as in any other move.
///
/// England:
///     F English Channel Convoys A London - Brest
///     A London - Brest
///     F Mid-Atlantic Ocean Supports A London - Brest
///
/// France:
///     A Paris - Brest
///
/// The army in London receives support and beats the army in Paris.
/// This means that the army London will end in Brest
/// and the French army in Paris stays in Paris.
#[test]
fn test_datc_6_f_3() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_e_mao = Unit::new_fleet(Power::England, p("mao"));
    let unit_f_par = Unit::new_army(Power::France, p("par"));
    phase.data.orders.push(unit_e_eng.convoy(unit_e_lon, p("bre")));
    phase.data.orders.push(unit_e_lon.move_to(p("bre")));
    phase.data.orders.push(unit_e_mao.support_move(unit_e_lon, p("bre")));
    phase.data.orders.push(unit_f_par.move_to(p("bre")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
}

/// 6.F.4. TEST CASE, AN ATTACKED CONVOY IS NOT DISRUPTED
/// A convoy can only be disrupted by dislodging the fleets.
/// Attacking is not sufficient.
///
/// England:
///     F North Sea Convoys A London - Holland
///     A London - Holland
///
/// Germany:
///     F Skagerrak - North Sea
///
/// The army in London will successfully convoy and end in Holland.
#[test]
fn test_datc_6_f_4() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("hol")));
    phase.data.orders.push(unit_e_lon.move_to(p("hol")));
    phase.data.orders.push(unit_g_ska.move_to(p("nth")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
}

/// 6.F.5. TEST CASE, A BELEAGUERED CONVOY IS NOT DISRUPTED
/// Even when a convoy is in a beleaguered garrison it is not disrupted.
///
/// England:
///     F North Sea Convoys A London - Holland
///     A London - Holland
///
/// France:
///     F English Channel - North Sea
///     F Belgium Supports F English Channel - North Sea
///
/// Germany:
///     F Skagerrak - North Sea
///     F Denmark Supports F Skagerrak - North Sea
///
/// The army in London will successfully convoy and end in Holland.
#[test]
fn test_datc_6_f_5() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_bel = Unit::new_fleet(Power::France, p("bel"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    let unit_g_den = Unit::new_fleet(Power::Germany, p("den"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("hol")));
    phase.data.orders.push(unit_e_lon.move_to(p("hol")));
    phase.data.orders.push(unit_f_eng.move_to(p("nth")));
    phase.data.orders.push(unit_f_bel.support_move(unit_f_eng, p("nth")));
    phase.data.orders.push(unit_g_ska.move_to(p("nth")));
    phase.data.orders.push(unit_g_den.support_move(unit_g_ska, p("nth")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
}

/// 6.F.6. TEST CASE, DISLODGED CONVOY DOES NOT CUT SUPPORT
/// When a fleet of a convoy is dislodged, the convoy is completely cancelled.
/// So, no support is cut.
///
/// England:
///     F North Sea Convoys A London - Holland
///     A London - Holland
///
/// Germany:
///     A Holland Supports A Belgium
///     A Belgium Supports A Holland
///     F Helgoland Bight Supports F Skagerrak - North Sea
///     F Skagerrak - North Sea
///
/// France:
///     A Picardy - Belgium
///     A Burgundy Supports A Picardy - Belgium
///
/// The hold order of Holland on Belgium will sustain
/// and Belgium will not be dislodged
/// by the French in Picardy.
#[test]
fn test_datc_6_f_6() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_g_hol = Unit::new_army(Power::Germany, p("hol"));
    let unit_g_bel = Unit::new_army(Power::Germany, p("bel"));
    let unit_g_hel = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    let unit_f_pic = Unit::new_army(Power::France, p("pic"));
    let unit_f_bur = Unit::new_army(Power::France, p("bur"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("hol")));
    phase.data.orders.push(unit_e_lon.move_to(p("hol")));
    phase.data.orders.push(unit_g_hol.support_hold(unit_g_bel));
    phase.data.orders.push(unit_g_bel.support_hold(unit_g_hol));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_ska, p("nth")));
    phase.data.orders.push(unit_g_ska.move_to(p("nth")));
    phase.data.orders.push(unit_f_pic.move_to(p("bel")));
    phase.data.orders.push(unit_f_bur.support_move(unit_f_pic, p("bel")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Valid);
}

/// 6.F.7. TEST CASE, DISLODGED CONVOY DOES NOT CAUSE CONTESTED AREA
/// When a fleet of a convoy is dislodged, the landing area is not contested,
/// so other units can retreat to that area.
///
/// England:
///     F North Sea Convoys A London - Holland
///     A London - Holland
///
/// Germany:
///     F Helgoland Bight Supports F Skagerrak - North Sea
///     F Skagerrak - North Sea
///
/// The dislodged English fleet can retreat to Holland.
#[test]
fn test_datc_6_f_7() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_g_hel = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("hol")));
    phase.data.orders.push(unit_e_lon.move_to(p("hol")));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_ska, p("nth")));
    phase.data.orders.push(unit_g_ska.move_to(p("nth")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_ne!(phase.data.orders[0].dislodged_from.map(|p| p.code()), Some("hol"));
    assert!(!phase.data.standoff_province_codes.contains(&"hol".to_string()));
}

/// 6.F.8. TEST CASE, DISLODGED CONVOY DOES NOT CAUSE A BOUNCE
/// When a fleet of a convoy is dislodged, then there will be no bounce in the landing area.
///
/// England:
///     F North Sea Convoys A London - Holland
///     A London - Holland
///
/// Germany:
///     F Helgoland Bight Supports F Skagerrak - North Sea
///     F Skagerrak - North Sea
///     A Belgium - Holland
///
/// The army in Belgium will not bounce and move to Holland.
#[test]
fn test_datc_6_f_8() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_g_hel = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    let unit_g_bel = Unit::new_army(Power::Germany, p("bel"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("hol")));
    phase.data.orders.push(unit_e_lon.move_to(p("hol")));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_ska, p("nth")));
    phase.data.orders.push(unit_g_ska.move_to(p("nth")));
    phase.data.orders.push(unit_g_bel.move_to(p("hol")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
}

/// 6.F.9. TEST CASE, DISLODGE OF MULTI-ROUTE CONVOY
/// When a fleet of a convoy with multiple routes is dislodged,
/// the result depends on the rulebook that is used.
///
/// England:
///     F English Channel Convoys A London - Belgium
///     F North Sea Convoys A London - Belgium
///     A London - Belgium
///
/// France:
///     F Brest Supports F Mid-Atlantic Ocean - English Channel
///     F Mid-Atlantic Ocean - English Channel
///
/// The French fleet in Mid Atlantic Ocean will dislodge the convoying fleet in the English Channel.
///
/// If the 1971 rules are used (see issue 4.A.1), this will disrupt the convoy and the army will stay in London.
/// When later rulebooks are used (which I prefer) the army can still go via the North Sea and the convoy succeeds
/// and the London army will end in Belgium.
#[test]
fn test_datc_6_f_9() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_f_bre = Unit::new_fleet(Power::France, p("bre"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    phase.data.orders.push(unit_e_eng.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_f_bre.support_move(unit_f_mao, p("eng")));
    phase.data.orders.push(unit_f_mao.move_to(p("eng")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
}

/// 6.F.10. TEST CASE, DISLODGE OF MULTI-ROUTE CONVOY WITH FOREIGN FLEET
/// When the 1971 rulebook is used "unwanted" multi-route convoys are possible.
///
/// England:
///     F North Sea Convoys A London - Belgium
///     A London - Belgium
///
/// Germany:
///     F English Channel Convoys A London - Belgium
///
/// France:
///     F Brest Supports F Mid-Atlantic Ocean - English Channel
///     F Mid-Atlantic Ocean - English Channel
///
/// The same as in the previous test case, the French fleet in Mid Atlantic Ocean
/// will dislodge the convoying fleet in the English Channel.
/// If the 1971 rules are used (see issue 4.A.1),
/// this will disrupt the convoy and the army will stay in London.
/// Without the "help" of the Germans the convoy would have succeeded!
/// When later rulebooks are used (which I prefer) the army can still go via the North Sea
/// and the convoy succeeds and the London army will end in Belgium.
#[test]
fn test_datc_6_f_10() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_g_eng = Unit::new_fleet(Power::Germany, p("eng"));
    let unit_f_bre = Unit::new_fleet(Power::France, p("bre"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_g_eng.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_f_bre.support_move(unit_f_mao, p("eng")));
    phase.data.orders.push(unit_f_mao.move_to(p("eng")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
}

/// 6.F.11. TEST CASE, DISLODGE OF MULTI-ROUTE CONVOY WITH ONLY FOREIGN FLEETS
/// With the 1971 rulebook one could adopt a rule (DPTG)
/// that foreign fleets are not used when not necessary,
/// but this doesn't prevent an "unwanted" convoy when all convoying fleets are foreign.
///
/// England:
///     A London - Belgium
///
/// Germany:
///     F English Channel Convoys A London - Belgium
///
/// Russia:
///     F North Sea Convoys A London - Belgium
///
/// France:
///     F Brest Supports F Mid-Atlantic Ocean - English Channel
///     F Mid-Atlantic Ocean - English Channel
///
/// Again, the French fleet in Mid Atlantic Ocean will dislodge the convoying fleet
/// in the English Channel.
/// If the 1971 rules are used (see issue 4.A.1),
/// this will disrupt the convoy and the army will stay in London.
/// When later rulebooks are used (which I prefer)
/// the army can still go via the North Sea and the convoy succeeds
/// and the London army will end in Belgium.
#[test]
fn test_datc_6_f_11() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_g_eng = Unit::new_fleet(Power::Germany, p("eng"));
    let unit_r_nth = Unit::new_fleet(Power::Russia, p("nth"));
    let unit_f_bre = Unit::new_fleet(Power::France, p("bre"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_g_eng.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_r_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_f_bre.support_move(unit_f_mao, p("eng")));
    phase.data.orders.push(unit_f_mao.move_to(p("eng")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
}

/// 6.F.12. TEST CASE, DISLODGED CONVOYING FLEET NOT ON ROUTE
/// When the rule is used that convoys are disrupted
/// when one of the routes is disrupted (see issue 4.A.1),
/// the convoy is not necessarily disrupted
/// when one of the fleets ordered to convoy is dislodged.
///
/// England:
///     F English Channel Convoys A London - Belgium
///     A London - Belgium
///     F Irish Sea Convoys A London - Belgium
///
/// France:
///     F North Atlantic Ocean Supports F Mid-Atlantic Ocean - Irish Sea
///     F Mid-Atlantic Ocean - Irish Sea
///
/// Even when convoys are disrupted when one of the routes is disrupted (see issue 4.A.1),
/// the convoy from London to Belgium will still succeed,
/// since the dislodged fleet in the Irish Sea is not part of any route,
/// although it can be reached from the starting point London.
#[test]
fn test_datc_6_f_12() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_e_iri = Unit::new_fleet(Power::England, p("iri"));
    let unit_f_nao = Unit::new_fleet(Power::France, p("nao"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    phase.data.orders.push(unit_e_eng.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_e_iri.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_f_nao.support_move(unit_f_mao, p("iri")));
    phase.data.orders.push(unit_f_mao.move_to(p("iri")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
}

/// 6.F.13. TEST CASE, THE UNWANTED ALTERNATIVE
/// This situation is not difficult to adjudicate,
/// but it shows that even if someone wants to convoy,
/// the player might not want an alternative route for the convoy.
///
/// England:
///     A London - Belgium
///     F North Sea Convoys A London - Belgium
///
/// France:
///     F English Channel Convoys A London - Belgium
///
/// Germany:
///     F Holland Supports F Denmark - North Sea
///     F Denmark - North Sea
///
/// If France and German are allies, England want to keep its army in London,
/// to defend the island. An army in Belgium could easily be destroyed
/// by an alliance of France and Germany.
/// England tries to be friends with Germany,
/// however France and Germany trick England.
/// The convoy of the army in London succeeds
/// and the fleet in Denmark dislodges the fleet in the North Sea.
#[test]
fn test_datc_6_f_13() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_g_hol = Unit::new_fleet(Power::Germany, p("hol"));
    let unit_g_den = Unit::new_fleet(Power::Germany, p("den"));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_f_eng.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_g_hol.support_move(unit_g_den, p("nth")));
    phase.data.orders.push(unit_g_den.move_to(p("nth")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
}

/// 6.F.14. TEST CASE, SIMPLE CONVOY PARADOX
/// The most common paradox is when the attacked unit supports an attack
/// on one of the convoying fleets.
///
/// England:
///     F London Supports F Wales - English Channel
///     F Wales - English Channel
///
/// France:
///     A Brest - London
///     F English Channel Convoys A Brest - London
///
/// See issue 4.A.2
/// According to all rulebooks (including the Szykman rule which I prefer),
/// the support of London is not cut.
/// That means that the fleet in the English Channel is dislodged.
#[test]
fn test_datc_6_f_14() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_lon = Unit::new_fleet(Power::England, p("lon"));
    let unit_e_wal = Unit::new_fleet(Power::England, p("wal"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_wal, p("eng")));
    phase.data.orders.push(unit_e_wal.move_to(p("eng")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
}

/// 6.F.15. TEST CASE, SIMPLE CONVOY PARADOX WITH ADDITIONAL CONVOY
/// Paradox rules only apply on the paradox core.
///
/// England:
///     F London Supports F Wales - English Channel
///     F Wales - English Channel
///
/// France:
///     A Brest - London
///     F English Channel Convoys A Brest - London
///
/// Italy:
///     F Irish Sea Convoys A North Africa - Wales
///     F Mid-Atlantic Ocean Convoys A North Africa - Wales
///     A North Africa - Wales
///
/// The adjudication of the paradox in the English Channel should not interfere
/// with the adjudication of the Italian convoy.
/// Both the fleet in Wales as the army in North Africa succeed in moving.
#[test]
fn test_datc_6_f_15() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_lon = Unit::new_fleet(Power::England, p("lon"));
    let unit_e_wal = Unit::new_fleet(Power::England, p("wal"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_i_iri = Unit::new_fleet(Power::Italy, p("iri"));
    let unit_i_mao = Unit::new_fleet(Power::Italy, p("mao"));
    let unit_i_naf = Unit::new_army(Power::Italy, p("naf"));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_wal, p("eng")));
    phase.data.orders.push(unit_e_wal.move_to(p("eng")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_i_iri.convoy(unit_i_naf, p("wal")));
    phase.data.orders.push(unit_i_mao.convoy(unit_i_naf, p("wal")));
    phase.data.orders.push(unit_i_naf.move_to(p("wal")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Success);
}

/// 6.F.16. TEST CASE, PANDIN'S PARADOX
/// In Pandin's paradox, the attacked unit protects the convoying fleet
/// by a beleaguered garrison.
///
/// England:
///     F London Supports F Wales - English Channel
///     F Wales - English Channel
///
/// France:
///     A Brest - London
///     F English Channel Convoys A Brest - London
///
/// Germany:
///     F North Sea Supports F Belgium - English Channel
///     F Belgium - English Channel
///
/// See issue 4.A.2
/// According to all rulebooks (including the Szykman rule which I prefer),
/// the support of London is not cut.
/// That means that the fleet in the English Channel is not dislodged
/// and none of the units succeed to move.
#[test]
fn test_datc_6_f_16() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_lon = Unit::new_fleet(Power::England, p("lon"));
    let unit_e_wal = Unit::new_fleet(Power::England, p("wal"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_g_nth = Unit::new_fleet(Power::Germany, p("nth"));
    let unit_g_bel = Unit::new_fleet(Power::Germany, p("bel"));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_wal, p("eng")));
    phase.data.orders.push(unit_e_wal.move_to(p("eng")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_g_nth.support_move(unit_g_bel, p("eng")));
    phase.data.orders.push(unit_g_bel.move_to(p("eng")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
}

/// 6.F.17. TEST CASE, PANDIN'S EXTENDED PARADOX
/// In Pandin's extended paradox,
/// the attacked unit protects the convoying fleet
/// by a beleaguered garrison and the attacked unit can dislodge
/// the unit that gives the protection.
///
/// England:
///     F London Supports F Wales - English Channel
///     F Wales - English Channel
///
/// France:
///     A Brest - London
///     F English Channel Convoys A Brest - London
///     F Yorkshire Supports A Brest - London
///
/// Germany:
///     F North Sea Supports F Belgium - English Channel
///     F Belgium - English Channel
///
/// When the 1971/1982/2000/2023 rules are used (see issue 4.A.2),
/// the support of London is not cut. That means that the fleet
/// in the English Channel is not dislodged.
/// The convoy will succeed and dislodge the fleet in London.
/// One can argue that this violates the dislodge rule,
/// but one may assume that the paradox convoy rule take precedence over the dislodge rule.
/// If the Simon Szykman alternative is used (which I prefer),
/// the convoy fails and the fleet in London and the English Channel are not dislodged
/// (which I think is a more appealing adjudication).
#[test]
fn test_datc_6_f_17() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_lon = Unit::new_fleet(Power::England, p("lon"));
    let unit_e_wal = Unit::new_fleet(Power::England, p("wal"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_yor = Unit::new_fleet(Power::France, p("yor"));
    let unit_g_nth = Unit::new_fleet(Power::Germany, p("nth"));
    let unit_g_bel = Unit::new_fleet(Power::Germany, p("bel"));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_wal, p("eng")));
    phase.data.orders.push(unit_e_wal.move_to(p("eng")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_f_yor.support_move(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_g_nth.support_move(unit_g_bel, p("eng")));
    phase.data.orders.push(unit_g_bel.move_to(p("eng")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
}

/// 6.F.18. TEST CASE, BETRAYAL PARADOX
/// The betrayal paradox is comparable to Pandin's paradox,
/// but now the attacked unit directly supports the convoying fleet.
/// Of course, this will only happen when the player of the attacked unit is betrayed.
///
/// England:
///     F North Sea Convoys A London - Belgium
///     A London - Belgium
///     F English Channel Supports A London - Belgium
///
/// France:
///     F Belgium Supports F North Sea
///
/// Germany:
///     F Helgoland Bight Supports F Skagerrak - North Sea
///     F Skagerrak - North Sea
///
/// If the English convoy from London to Belgium is successful,
/// then it cuts the France support necessary to hold the fleet
/// in the North Sea (see issue 4.A.2).
/// The 1971, 2000 and 2023 rules do not give an answer on this.
/// According to the 1982 rules the French support
/// on the North Sea will not be cut.
/// So, the fleet in the North Sea will not be dislodged by the Germans
/// and the army in London will dislodge the French army in Belgium.
/// If the Szykman rule is followed (which I prefer),
/// the convoy in the English Channel fails.
/// Without the convoy, the move of the army in London will fail
/// and the support in Belgium will not be cut.
/// That means that the fleet in the North Sea will not be dislodged.
#[test]
fn test_datc_6_f_18() {
    // Szykman ルールを採用する
    // - F bel からの支援はカットされず F nth は撃退されない
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_e_eng = Unit::new_fleet(Power::England, p("eng"));
    let unit_f_bel = Unit::new_fleet(Power::France, p("bel"));
    let unit_g_hel = Unit::new_fleet(Power::Germany, p("hel"));
    let unit_g_ska = Unit::new_fleet(Power::Germany, p("ska"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_e_eng.support_move(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_f_bel.support_hold(unit_e_nth));
    phase.data.orders.push(unit_g_hel.support_move(unit_g_ska, p("nth")));
    phase.data.orders.push(unit_g_ska.move_to(p("nth")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Failure);
}

/// 6.F.19. TEST CASE, MULTI-ROUTE CONVOY DISRUPTION PARADOX
/// The situation becomes more complex when the convoy has alternative routes.
///
/// France:
///     A Tunis - Naples
///     F Tyrrhenian Sea Convoys A Tunis - Naples
///     F Ionian Sea Convoys A Tunis - Naples
///
/// Italy:
///     F Naples Supports F Rome - Tyrrhenian Sea
///     F Rome - Tyrrhenian Sea
///
/// Now, two issues play a role. The rule about disruption
/// of multi-route convoys (issue 4.A.1)
/// and the determination of how paradoxes are resolved (issue 4.A.2).
/// If the 1971 rulebook is used then a multi-route convoy is disrupted
/// when one of the routes is disrupted.
/// That makes this situation paradoxical and the 1971 paradox rule kicks in.
/// The support of the fleet in Naples is not cut and the fleet
/// in Rome dislodges the fleet in the Tyrrhenian Sea.
/// With the 1982 rulebook, the support of Naples is not cut,
/// because it is supporting an action in a body of water that contains a convoying fleet.
/// This means that the fleet in Rome dislodges the fleet in the Tyrrhenian Sea.
/// According to the 2000/2023 rules
/// the fleet in the Tyrrhenian Sea is not "necessary" for the convoy
/// and the support of Naples is cut and the fleet in the Tyrrhenian Sea is not dislodged.
/// If the Szykman rule is used (which I prefer),
/// then there is no paradoxical situation. The support of Naples is cut
/// (the same as in the 2000/2023 ruling)
/// and the fleet in the Tyrrhenian Sea is not dislodged.
#[test]
fn test_datc_6_f_19() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_tun = Unit::new_army(Power::France, p("tun"));
    let unit_f_tyr = Unit::new_fleet(Power::France, p("tys"));
    let unit_f_ion = Unit::new_fleet(Power::France, p("ion"));
    let unit_i_nap = Unit::new_fleet(Power::Italy, p("nap"));
    let unit_i_rom = Unit::new_fleet(Power::Italy, p("rom"));
    phase.data.orders.push(unit_f_tun.move_to(p("nap")));
    phase.data.orders.push(unit_f_tyr.convoy(unit_f_tun, p("nap")));
    phase.data.orders.push(unit_f_ion.convoy(unit_f_tun, p("nap")));
    phase.data.orders.push(unit_i_nap.support_move(unit_i_rom, p("tys")));
    phase.data.orders.push(unit_i_rom.move_to(p("tys")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
}

/// 6.F.20. TEST CASE, UNWANTED MULTI-ROUTE CONVOY PARADOX
/// The 1982 paradox rule allows some creative defense.
///
/// France:
///     A Tunis - Naples
///     F Tyrrhenian Sea Convoys A Tunis - Naples
///
/// Italy:
///     F Naples Supports F Ionian Sea
///     F Ionian Sea Convoys A Tunis - Naples
///
/// Turkey:
///     F Aegean Sea Supports F Eastern Mediterranean - Ionian Sea
///     F Eastern Mediterranean - Ionian Sea
///
/// Again, two issues play a role.
/// The rule about disruption of multi-route convoys (issue 4.A.1)
/// and the determination of how paradoxes are resolved (issue 4.A.2).
/// If the 1971 rulebook is used,
/// then a multi-route convoy is disrupted
/// when one of the routes is disrupted.
/// This makes the situation paradoxical.
/// However, since the fleet in Naples is not supporting an attack
/// on a convoying fleet,
/// the paradox rule does not apply
/// and the 1971 rules do not give answer to this situation.
/// With the 1982 rules the support in Naples is not cut,
/// because it is supporting an action in a body of water that contains a convoying fleet.
/// That means that the fleet in the Ionian Sea is not dislodged.
/// The paradox rule of the 2000/2023 rules,
/// does not kick in, because the support is not a support that attacks the convoying fleet.
/// However, with these rules a multi-route convoy is only disrupted
/// when all routes are disrupted,
/// which prevents that this situation is a paradox.
/// So, the support of Naples is cut and the fleet in the Ionian Sea is dislodged
/// by the Turkish fleet in the Eastern Mediterranean.
/// If the Szykman rule is used, then there is no paradoxical situation.
/// The support of Naples is cut and the fleet in the Ionian Sea is dislodged
/// by the Turkish fleet in the Eastern Mediterranean.
/// As you can see, the 1982 rules allow the Italian player to save its fleet
/// in the Ionian Sea with a trick.
/// I do not consider this trick as normal tactical play.
/// I prefer the Szykman rule as one of the rules that does not allow this trick.
/// According to this rule the fleet in the Ionian Sea is dislodged.
#[test]
fn test_datc_6_f_20() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_f_tun = Unit::new_army(Power::France, p("tun"));
    let unit_f_tyr = Unit::new_fleet(Power::France, p("tys"));
    let unit_i_nap = Unit::new_fleet(Power::Italy, p("nap"));
    let unit_i_ion = Unit::new_fleet(Power::Italy, p("ion"));
    let unit_t_aeg = Unit::new_fleet(Power::Turkey, p("aeg"));
    let unit_t_eas = Unit::new_fleet(Power::Turkey, p("eas"));
    phase.data.orders.push(unit_f_tun.move_to(p("nap")));
    phase.data.orders.push(unit_f_tyr.convoy(unit_f_tun, p("nap")));
    phase.data.orders.push(unit_i_nap.support_hold(unit_i_ion));
    phase.data.orders.push(unit_i_ion.convoy(unit_f_tun, p("nap")));
    phase.data.orders.push(unit_t_aeg.support_move(unit_t_eas, p("ion")));
    phase.data.orders.push(unit_t_eas.move_to(p("ion")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Cut);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
}

/// 6.F.21. TEST CASE, DAD'S ARMY CONVOY
/// The 1982 paradox rule has as side effect that convoying armies do not cut support
/// in some situations that are not paradoxical.
///
/// Russia:
///     A Edinburgh Supports A Norway - Clyde
///     F Norwegian Sea Convoys A Norway - Clyde
///     A Norway - Clyde
///
/// France:
///     F Irish Sea Supports F Mid-Atlantic Ocean - North Atlantic Ocean
///     F Mid-Atlantic Ocean - North Atlantic Ocean
///
/// England:
///     A Liverpool - Clyde via convoy
///     F North Atlantic Ocean Convoys A Liverpool - Clyde
///     F Clyde Supports F North Atlantic Ocean
///
/// In all rules, except the 1982 paradox rule,
/// the support of the fleet in Clyde on the North Atlantic Ocean is cut
/// and the French fleet in the Mid-Atlantic Ocean will dislodge the fleet
/// in the North Atlantic Ocean.
/// This is the preferred way.
/// However, in the 1982 paradox rule (see issue 4.A.2),
/// the support of the fleet in Clyde is not cut.
/// That means that the English fleet in the North Atlantic Ocean is not dislodged.
/// As you can see, the 1982 rule allows England to save its fleet
/// in the North Atlantic Ocean in a very strange way.
/// Just the support of Clyde is insufficient
/// (if there is no convoy, the support is cut).
/// Only the convoy to the area occupied by own unit,
/// can do the trick in this situation.
/// The embarking of troops in the fleet deceives the enemy so much that it works
/// as a magic cloak.
/// The enemy is not able to dislodge the fleet in the North Atlantic Ocean any more.
/// Of course, this will only work in comedies.
/// I prefer the Szykman rule as one of the rules that does not allow this trick.
/// According to this rule (and all other paradox rules),
/// the fleet in the North Atlantic is just dislodged.
#[test]
fn test_datc_6_f_21() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_r_edi = Unit::new_army(Power::Russia, p("edi"));
    let unit_r_nwg = Unit::new_fleet(Power::Russia, p("nwg"));
    let unit_r_nwy = Unit::new_army(Power::Russia, p("nwy"));
    let unit_f_iri = Unit::new_fleet(Power::France, p("iri"));
    let unit_f_mao = Unit::new_fleet(Power::France, p("mao"));
    let unit_e_lvp = Unit::new_army(Power::England, p("lvp"));
    let unit_e_nao = Unit::new_fleet(Power::England, p("nao"));
    let unit_e_cly = Unit::new_fleet(Power::England, p("cly"));
    phase.data.orders.push(unit_r_edi.support_move(unit_r_nwy, p("cly")));
    phase.data.orders.push(unit_r_nwg.convoy(unit_r_nwy, p("cly")));
    phase.data.orders.push(unit_r_nwy.move_to(p("cly")));
    phase.data.orders.push(unit_f_iri.support_move(unit_f_mao, p("nao")));
    phase.data.orders.push(unit_f_mao.move_to(p("nao")));
    phase.data.orders.push(unit_e_lvp.move_to(p("cly")));
    phase.data.orders.push(unit_e_nao.convoy(unit_e_lvp, p("cly")));
    phase.data.orders.push(unit_e_cly.support_hold(unit_e_nao));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Dislodged);
}

/// 6.F.22. TEST CASE, SECOND ORDER PARADOX WITH TWO RESOLUTIONS
/// Two convoys are involved in a second order paradox.
///
/// England:
/// F Edinburgh - North Sea
/// F London Supports F Edinburgh - North Sea
///
/// France:
/// A Brest - London
/// F English Channel Convoys A Brest - London
///
/// Germany:
/// F Belgium Supports F Picardy - English Channel
/// F Picardy - English Channel
///
/// Russia:
/// A Norway - Belgium
/// F North Sea Convoys A Norway - Belgium
/// Without any paradox rule, there are two consistent resolutions.
/// The supports of the English fleet in London and the German fleet
/// in Picardy are not cut.
/// That means that the French fleet in the English Channel
/// and the Russian fleet in the North Sea are dislodged,
/// which makes it impossible to cut the support.
/// The other resolution is that the supports of the English fleet
/// in London the German fleet in Picardy are cut.
/// In that case the French fleet in the English Channel
/// and the Russian fleet in the North Sea will survive and will not be dislodged.
/// This gives the possibility to cut the support.
/// The 1971, 2000 and 2023 rules (see issue 4.A.2) do not have an answer on this.
/// According to the 1982 rule,
/// the supports are not cut which means that the French fleet in the English Channel
/// and the Russian fleet in the North Sea are dislodged.
/// The Szykman (which I prefer),
/// has the same result as the 1982 rule. The supports are not cut,
/// the convoying armies fail to move, t
/// he fleet in Picardy dislodges the fleet in English Channel
/// and the fleet in Edinburgh dislodges the fleet in the North Sea.
#[test]
fn test_datc_6_f_22() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_edi = Unit::new_fleet(Power::England, p("edi"));
    let unit_e_lon = Unit::new_fleet(Power::England, p("lon"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_g_bel = Unit::new_fleet(Power::Germany, p("bel"));
    let unit_g_pic = Unit::new_fleet(Power::Germany, p("pic"));
    let unit_r_nwy = Unit::new_army(Power::Russia, p("nwy"));
    let unit_r_nth = Unit::new_fleet(Power::Russia, p("nth"));
    phase.data.orders.push(unit_e_edi.move_to(p("nth")));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_edi, p("nth")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_g_bel.support_move(unit_g_pic, p("eng")));
    phase.data.orders.push(unit_g_pic.move_to(p("eng")));
    phase.data.orders.push(unit_r_nwy.move_to(p("bel")));
    phase.data.orders.push(unit_r_nth.convoy(unit_r_nwy, p("bel")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Dislodged);
}

/// 6.F.23. TEST CASE, SECOND ORDER PARADOX WITH TWO EXCLUSIVE CONVOYS
/// In this paradox there are two consistent resolutions,
/// but where the two convoys do not fail or succeed at the same time.
///
/// England:
///     F Edinburgh - North Sea
///     F Yorkshire Supports F Edinburgh - North Sea
///
/// France:
///     A Brest - London
///     F English Channel Convoys A Brest - London
///
/// Germany:
///     F Belgium Supports F English Channel
///     F London Supports F North Sea
///
/// Italy:
///     F Mid-Atlantic Ocean - English Channel
///     F Irish Sea Supports F Mid-Atlantic Ocean - English Channel
///
/// Russia:
///     A Norway - Belgium
///     F North Sea Convoys A Norway - Belgium
///
/// Without any paradox rule, there are two consistent resolutions.
/// In one resolution,
/// the convoy in the English Channel is dislodged
/// by the fleet in the Mid-Atlantic Ocean,
/// while the convoy in the North Sea succeeds. In the other resolution,
/// it is the other way around.
/// The convoy in the North Sea is dislodged by the fleet in Edinburgh,
/// while the convoy in the English Channel succeeds.
/// The 1971, 2000 and 2023 rules (see issue 4.A.2) do not have an answer on this.
/// According to the 1982 rule,
/// the supports are not cut which means that the none of the units move.
/// The Szykman rule (which I prefer), has the same result as the 1982 rule.
/// The convoying armies fail to move and the supports are not cut.
/// Because of the failure to cut the support, no fleet succeeds to move.
#[test]
fn test_datc_6_f_23() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_edi = Unit::new_fleet(Power::England, p("edi"));
    let unit_e_yor = Unit::new_fleet(Power::England, p("yor"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_g_bel = Unit::new_fleet(Power::Germany, p("bel"));
    let unit_g_lon = Unit::new_fleet(Power::Germany, p("lon"));
    let unit_i_mao = Unit::new_fleet(Power::Italy, p("mao"));
    let unit_i_iri = Unit::new_fleet(Power::Italy, p("iri"));
    let unit_r_mwy = Unit::new_army(Power::Russia, p("nwy"));
    let unit_r_nth = Unit::new_fleet(Power::Russia, p("nth"));
    phase.data.orders.push(unit_e_edi.move_to(p("nth")));
    phase.data.orders.push(unit_e_yor.support_move(unit_e_edi, p("nth")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_g_bel.support_hold(unit_f_eng));
    phase.data.orders.push(unit_g_lon.support_hold(unit_r_nth));
    phase.data.orders.push(unit_i_mao.move_to(p("eng")));
    phase.data.orders.push(unit_i_iri.support_move(unit_i_mao, p("eng")));
    phase.data.orders.push(unit_r_mwy.move_to(p("bel")));
    phase.data.orders.push(unit_r_nth.convoy(unit_r_mwy, p("bel")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[8].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[9].status, OrderStatus::Valid);
}

/// 6.F.24. TEST CASE, SECOND ORDER PARADOX WITH NO RESOLUTION
/// As first order paradoxes, second order paradoxes come in two flavors, w
/// ith two resolutions or no resolution.
///
/// England:
///     F Edinburgh - North Sea
///     F London Supports F Edinburgh - North Sea
///     F Irish Sea - English Channel
///     F Mid-Atlantic Ocean Supports F Irish Sea - English Channel
///
/// France:
///     A Brest - London
///     F English Channel Convoys A Brest - London
///     F Belgium Supports F English Channel
///
/// Russia:
///     A Norway - Belgium
///     F North Sea Convoys A Norway - Belgium
///
/// When no paradox rule is used, there is no consistent resolution.
/// If the French support in Belgium is cut, the French fleet
/// in the English Channel will be dislodged.
/// That means that the support of London will not be cut
/// and the fleet in Edinburgh will dislodge the Russian fleet in the North Sea.
/// In this way the support in Belgium is not cut!
/// But if the support in Belgium is not cut,
/// the Russian fleet in the North Sea will not be dislodged
/// and the army in Norway can cut the support in Belgium.
/// The 1971, 2000 and 2023 rules (see issue 4.A.2) do not have an answer on this.
/// According to the 1982 rule,
/// the supports are not cut which means that the French fleet in the English Channel
/// will survive and but the Russian fleet in the North Sea is dislodged.
/// If the Szykman alternative is used (which I prefer),
/// the supports are not cut and the convoying armies fail to move,
/// which gives the same result as the 1982 rule.
#[test]
fn test_datc_6_f_24() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_e_edi = Unit::new_fleet(Power::England, p("edi"));
    let unit_e_lon = Unit::new_fleet(Power::England, p("lon"));
    let unit_e_iri = Unit::new_fleet(Power::England, p("iri"));
    let unit_e_mao = Unit::new_fleet(Power::England, p("mao"));
    let unit_f_bre = Unit::new_army(Power::France, p("bre"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_bel = Unit::new_fleet(Power::France, p("bel"));
    let unit_r_nwy = Unit::new_army(Power::Russia, p("nwy"));
    let unit_r_nth = Unit::new_fleet(Power::Russia, p("nth"));
    phase.data.orders.push(unit_e_edi.move_to(p("nth")));
    phase.data.orders.push(unit_e_lon.support_move(unit_e_edi, p("nth")));
    phase.data.orders.push(unit_e_iri.move_to(p("eng")));
    phase.data.orders.push(unit_e_mao.support_move(unit_e_iri, p("eng")));
    phase.data.orders.push(unit_f_bre.move_to(p("lon")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bre, p("lon")));
    phase.data.orders.push(unit_f_bel.support_hold(unit_f_eng));
    phase.data.orders.push(unit_r_nwy.move_to(p("bel")));
    phase.data.orders.push(unit_r_nth.convoy(unit_r_nwy, p("bel")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[8].status, OrderStatus::Dislodged);
}

/// 6.F.25. TEST CASE, CUT SUPPORT LAST
/// For manual play the rule of thumb is, cut support first. However,
/// in below example the support of Holland is some of the last orders
/// to adjudicated.
///
/// Germany:
///     A Rhur - Belgium
///     A Holland Supports Rhur - Belgium
///     A Denmark - Norway
///     F Skagerrak Convoys Denmark - Norway
///     A Finland Supports Denmark - Norway
///
/// England:
///     A Yorkshire - Holland
///     F North Sea Convoys Yorkshire - Holland
///     F Helgoland Bight Supports Yorkshire - Holland
///     A Belgium Hold
///
/// Russia:
///     F Norwegian Sea - North Sea
///     F Norway Supports Norwegian Sea - North Sea
///     F Sweden - Skagerrak
///
/// The fleet in Sweden fails to disrupt the convoy in Skagerrak.
/// The move from Denmark to Norway succeeds and cuts the support of Norway.
/// The fleet in the Norwegian Sea fails to disrupt the convoy in North Sea.
/// The move from Yorkshire to Holland succeeds and cuts the support of Holland.
/// The move from Rhur fails to dislodge the army in Belgium.
#[test]
fn test_datc_6_f_25() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let unit_g_1_ruh = Unit::new_army(Power::Germany, p("ruh"));
    let unit_g_2_hol = Unit::new_army(Power::Germany, p("hol"));
    let unit_g_3_den = Unit::new_army(Power::Germany, p("den"));
    let unit_g_4_ska = Unit::new_fleet(Power::Germany, p("ska"));
    let unit_g_5_fin = Unit::new_army(Power::Germany, p("fin"));
    let unit_e_1_yor = Unit::new_army(Power::England, p("yor"));
    let unit_e_2_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_3_hel = Unit::new_fleet(Power::England, p("hel"));
    let unit_e_4_bel = Unit::new_army(Power::England, p("bel"));
    let unit_r_1_nwg = Unit::new_fleet(Power::Russia, p("nwg"));
    let unit_r_2_nwy = Unit::new_fleet(Power::Russia, p("nwy"));
    let unit_r_3_swe = Unit::new_fleet(Power::Russia, p("swe"));
    phase.data.orders.push(unit_g_1_ruh.move_to(p("bel")));
    phase.data.orders.push(unit_g_2_hol.support_move(unit_g_1_ruh, p("bel")));
    phase.data.orders.push(unit_g_3_den.move_to(p("nwy")));
    phase.data.orders.push(unit_g_4_ska.convoy(unit_g_3_den, p("nwy")));
    phase.data.orders.push(unit_g_5_fin.support_move(unit_g_3_den, p("nwy")));
    phase.data.orders.push(unit_e_1_yor.move_to(p("hol")));
    phase.data.orders.push(unit_e_2_nth.convoy(unit_e_1_yor, p("hol")));
    phase.data.orders.push(unit_e_3_hel.support_move(unit_e_1_yor, p("hol")));
    phase.data.orders.push(unit_e_4_bel.hold());
    phase.data.orders.push(unit_r_1_nwg.move_to(p("nth")));
    phase.data.orders.push(unit_r_2_nwy.support_move(unit_r_1_nwg, p("nth")));
    phase.data.orders.push(unit_r_3_swe.move_to(p("ska")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[8].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[9].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[10].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[11].status, OrderStatus::Failure);
}
