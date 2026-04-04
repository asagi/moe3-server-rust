//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6H]
//!
//! * 6.H. TEST CASES, RETREATING
//!
//! [DATC_6H]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.H

// use super::super::order::*;
// use super::super::phase::main_order_resolution::*;
// use super::super::phase::*;
// use super::super::power::*;
// use super::super::province::*;
// use super::super::unit::*;

// fn p(code: &str) -> Province {
//     Province::from_code(code).expect("valid province code")
// }

/// 6.H.1. TEST CASE, NO SUPPORTS DURING RETREAT
/// Supports are not allowed in the retreat phase.
///
/// Austria:
/// F Trieste Hold
/// A Serbia Hold
///
/// Turkey:
/// F Greece Hold
///
/// Italy:
/// A Venice Supports A Tyrolia - Trieste
/// A Tyrolia - Trieste
/// F Ionian Sea - Greece
/// F Aegean Sea Supports F Ionian Sea - Greece
/// The fleet in Trieste and the fleet in Greece are dislodged. If the retreat orders are as follows:
///
/// Austria:
/// F Trieste - Albania
/// A Serbia Supports F Trieste - Albania
///
/// Turkey:
/// F Greece - Albania
/// The Austrian support order is illegal. Both dislodged fleets are disbanded.
#[test]
fn test_datc_6_h_1() {}

/// 6.H.2. TEST CASE, NO SUPPORTS FROM RETREATING UNIT
/// Even a retreating unit cannot give support.
///
/// England:
/// A Liverpool - Edinburgh
/// F Yorkshire Supports A Liverpool - Edinburgh
/// F Norway Hold
///
/// Germany:
/// A Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
///
/// Russia:
/// F Edinburgh Hold
/// A Sweden Supports A Finland - Norway
/// A Finland - Norway
/// F Holland Hold
/// The English fleet in Norway and the Russian fleets in Edinburgh and Holland are dislodged. If the following retreat orders are given:
///
/// England:
/// F Norway - North Sea
///
/// Russia:
/// F Edinburgh - North Sea
/// F Holland Supports F Edinburgh - North Sea
/// Although the fleet in Holland may receive an order, it may not support (it is disbanded). The English fleet in Norway and the Russian fleet in Edinburgh bounce and are disbanded.
#[test]
fn test_datc_6_h_2() {}

/// 6.H.3. TEST CASE, NO CONVOY DURING RETREAT
/// Convoys during retreat are not allowed.
///
/// England:
/// F North Sea Hold
/// A Holland Hold
///
/// Germany:
/// F Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
/// The English army in Holland is dislodged. If England orders the following in retreat:
///
/// England:
/// A Holland - Yorkshire
/// F North Sea Convoys A Holland - Yorkshire
/// The convoy order is illegal. The army in Holland is disbanded.
#[test]
fn test_datc_6_h_3() {}

/// 6.H.4. TEST CASE, NO OTHER MOVES DURING RETREAT
/// Of course, you may not do any other move during a retreat. But look if the adjudicator checks for it.
///
/// England:
/// F North Sea Hold
/// A Holland Hold
///
/// Germany:
/// F Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
/// The English army in Holland is dislodged. If England orders the following in retreat:
///
/// England:
/// A Holland - Belgium
/// F North Sea - Norwegian Sea
/// The fleet in the North Sea is not dislodge, so the move is illegal.
#[test]
fn test_datc_6_h_4() {}

/// 6.H.5. TEST CASE, A UNIT MAY NOT RETREAT TO THE AREA FROM WHICH IT IS ATTACKED
/// Well, that would be of course stupid. Still, the adjudicator must be tested on this.
///
/// Russia:
/// F Constantinople Supports F Black Sea - Ankara
/// F Black Sea - Ankara
///
/// Turkey:
/// F Ankara Hold
/// Fleet in Ankara is dislodged and may not retreat to Black Sea.
#[test]
fn test_datc_6_h_5() {}

/// 6.H.6. TEST CASE, UNIT MAY NOT RETREAT TO A CONTESTED AREA
/// Standoff prevents retreat to the area.
///
/// Austria:
/// A Budapest Supports A Trieste - Vienna
/// A Trieste - Vienna
///
/// Germany:
/// A Munich - Bohemia
/// A Silesia - Bohemia
///
/// Italy:
/// A Vienna Hold
/// The Italian army in Vienna is dislodged. It may not retreat to Bohemia.
#[test]
fn test_datc_6_h_6() {}

/// 6.H.7. TEST CASE, MULTIPLE RETREAT TO SAME AREA WILL DISBAND UNITS
/// There can only be one unit in an area.
///
/// Austria:
/// A Budapest Supports A Trieste - Vienna
/// A Trieste - Vienna
///
/// Germany:
/// A Munich Supports A Silesia - Bohemia
/// A Silesia - Bohemia
///
/// Italy:
/// A Vienna Hold
/// A Bohemia Hold
/// If Italy orders the following for retreat:
///
/// Italy:
/// A Bohemia - Tyrolia
/// A Vienna - Tyrolia
/// Both armies will be disbanded.
#[test]
fn test_datc_6_h_7() {}

/// 6.H.8. TEST CASE, TRIPLE RETREAT TO SAME AREA WILL DISBAND UNITS
/// When three units retreat to the same area, then all three units are disbanded.
///
/// England:
/// A Liverpool - Edinburgh
/// F Yorkshire Supports A Liverpool - Edinburgh
/// F Norway Hold
///
/// Germany:
/// A Kiel Supports A Ruhr - Holland
/// A Ruhr - Holland
///
/// Russia:
/// F Edinburgh Hold
/// A Sweden Supports A Finland - Norway
/// A Finland - Norway
/// F Holland Hold
/// The fleets in Norway, Edinburgh and Holland are dislodged. If the following retreat orders are given:
///
/// England:
/// F Norway - North Sea
///
/// Russia:
/// F Edinburgh - North Sea
/// F Holland - North Sea
/// All three units are disbanded.
#[test]
fn test_datc_6_h_8() {}

/// 6.H.9. TEST CASE, DISLODGED UNIT WILL NOT MAKE ATTACKERS AREA CONTESTED
/// An army can follow.
///
/// England:
/// F Helgoland Bight - Kiel
/// F Denmark Supports F Helgoland Bight - Kiel
///
/// Germany:
/// A Berlin - Prussia
/// F Kiel Hold
/// A Silesia Supports A Berlin - Prussia
///
/// Russia:
/// A Prussia - Berlin
/// The fleet in Kiel can retreat to Berlin.
#[test]
fn test_datc_6_h_9() {}

/// 6.H.10. TEST CASE, NOT RETREATING TO ATTACKER DOES NOT MEAN CONTESTED
/// An army cannot retreat to the area of the attacker. The easiest way to program that, is to mark that area as "contested". However, this is not correct. Another army may retreat to that area.
///
/// England:
/// A Kiel Hold
///
/// Germany:
/// A Berlin - Kiel
/// A Munich Supports A Berlin - Kiel
/// A Prussia Hold
///
/// Russia:
/// A Warsaw - Prussia
/// A Silesia Supports A Warsaw - Prussia
/// The armies in Kiel and Prussia are dislodged. The English army in Kiel cannot retreat to Berlin, but the army in Prussia can retreat to Berlin. Suppose the following retreat orders are given:
///
/// England:
/// A Kiel - Berlin
///
/// Germany:
/// A Prussia - Berlin
/// The English retreat to Berlin is illegal and fails (the unit is disbanded). The German retreat to Berlin is successful and does not bounce on the English unit.
#[test]
fn test_datc_6_h_10() {}

/// 6.H.11. TEST CASE, RETREAT WHEN DISLODGED BY ADJACENT CONVOY
/// If a unit is dislodged by an army via convoy, the question arises whether the dislodged army can retreat to the original province of the convoyed army. This is only relevant in case the convoy was to an adjacent province.
///
/// France:
/// A Gascony - Marseilles via convoy
/// A Burgundy Supports A Gascony - Marseilles
/// F Mid-Atlantic Ocean Convoys A Gascony - Marseilles
/// F Western Mediterranean Convoys A Gascony - Marseilles
/// F Gulf of Lyon Convoys A Gascony - Marseilles
///
/// Italy:
/// A Marseilles Hold
/// The army in Gascony takes a convoy and does not pass the border of Gascony with Marseilles (it went a completely different direction). Now, the result depends on which rule is used for retreating (see issue 4.A.5).
///
/// The 2023 rules explicitly allow this. So, I prefer that Marseilles may retreat to Gascony.
#[test]
fn test_datc_6_h_11() {}

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
