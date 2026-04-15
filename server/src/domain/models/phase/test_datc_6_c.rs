//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6C]
//!
//! * 6.C. TEST CASES, CIRCULAR MOVEMENT
//!
//! [DATC_6C]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.C

use crate::domain::models::order::*;
use crate::domain::models::phase::methods::*;
use crate::domain::tests::a;
use crate::domain::tests::f;
use crate::domain::tests::p;

/// 6.C.1. TEST CASE, THREE ARMY CIRCULAR MOVEMENT
/// Three units can change place, even in spring 1901.
///
/// Turkey:
///     F Ankara - Constantinople
///     A Constantinople - Smyrna
///     A Smyrna - Ankara
///
/// All three units will move.
#[test]
fn test_datc_6_c_1() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_t_ank = f("t", "ank");
    let unit_t_con = a("t", "con");
    let unit_t_smy = a("t", "smy");
    phase.data.units.push(unit_t_ank);
    phase.data.units.push(unit_t_con);
    phase.data.units.push(unit_t_smy);
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("smy")));
    phase.data.orders.push(unit_t_smy.move_to(p("ank")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 3);
    assert!(phase.data.units.contains(&f("t", "con")));
    assert!(phase.data.units.contains(&a("t", "smy")));
    assert!(phase.data.units.contains(&a("t", "ank")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.C.2. TEST CASE, THREE ARMY CIRCULAR MOVEMENT WITH SUPPORT
/// Three units can change place, even when one gets support.
///
/// Turkey:
///     F Ankara - Constantinople
///     A Constantinople - Smyrna
///     A Smyrna - Ankara
///     A Bulgaria Supports F Ankara - Constantinople
///
/// Of course, the three units will move, but knowing how programs are written,
/// this can confuse the adjudicator.
#[test]
fn test_datc_6_c_2() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_t_ank = f("t", "ank");
    let unit_t_con = a("t", "con");
    let unit_t_smy = a("t", "smy");
    let unit_t_bul = a("t", "bul");
    phase.data.units.push(unit_t_ank);
    phase.data.units.push(unit_t_con);
    phase.data.units.push(unit_t_smy);
    phase.data.units.push(unit_t_bul);
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("smy")));
    phase.data.orders.push(unit_t_smy.move_to(p("ank")));
    phase.data.orders.push(unit_t_bul.support_move(unit_t_ank, p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&f("t", "con")));
    assert!(phase.data.units.contains(&a("t", "smy")));
    assert!(phase.data.units.contains(&a("t", "ank")));
    assert!(phase.data.units.contains(&unit_t_bul));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.C.3. TEST CASE, A DISRUPTED THREE ARMY CIRCULAR MOVEMENT
/// When one of the units bounces, the whole circular movement will hold.
///
/// Turkey:
///     F Ankara - Constantinople
///     A Constantinople - Smyrna
///     A Smyrna - Ankara
///     A Bulgaria - Constantinople
///
/// Every unit will keep its place.
#[test]
fn test_datc_6_c_3() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_t_ank = f("t", "ank");
    let unit_t_con = a("t", "con");
    let unit_t_smy = a("t", "smy");
    let unit_t_bul = a("t", "bul");
    phase.data.units.push(unit_t_ank);
    phase.data.units.push(unit_t_con);
    phase.data.units.push(unit_t_smy);
    phase.data.units.push(unit_t_bul);
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("smy")));
    phase.data.orders.push(unit_t_smy.move_to(p("ank")));
    phase.data.orders.push(unit_t_bul.move_to(p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_t_ank));
    assert!(phase.data.units.contains(&unit_t_con));
    assert!(phase.data.units.contains(&unit_t_smy));
    assert!(phase.data.units.contains(&unit_t_bul));
    assert!(phase.data.standoff_codes.contains(&"con".to_string()));
}

/// 6.C.4. TEST CASE, A CIRCULAR MOVEMENT WITH ATTACKED CONVOY
/// When the circular movement contains an attacked convoy, the circular movement succeeds.
/// The adjudication algorithm should handle attack of convoys before calculating circular movement.
///
/// Austria:
///     A Trieste - Serbia
///     A Serbia - Bulgaria
///
/// Turkey:
///     A Bulgaria - Trieste
///     F Aegean Sea Convoys A Bulgaria - Trieste
///     F Ionian Sea Convoys A Bulgaria - Trieste
///     F Adriatic Sea Convoys A Bulgaria - Trieste
///
/// Italy:
///     F Naples - Ionian Sea
///
/// The fleet in the Ionian Sea is attacked but not dislodged.
/// The circular movement succeeds.
/// The Austrian and Turkish armies will advance.
#[test]
fn test_datc_6_c_4() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_tri = a("a", "tri");
    let unit_a_ser = a("a", "ser");
    let unit_t_bul = a("t", "bul");
    let unit_t_aeg = f("t", "aeg");
    let unit_t_ion = f("t", "ion");
    let unit_t_adr = f("t", "adr");
    let unit_i_nap = f("i", "nap");
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_a_ser);
    phase.data.units.push(unit_t_bul);
    phase.data.units.push(unit_t_aeg);
    phase.data.units.push(unit_t_ion);
    phase.data.units.push(unit_t_adr);
    phase.data.units.push(unit_i_nap);
    phase.data.orders.push(unit_a_tri.move_to(p("ser")));
    phase.data.orders.push(unit_a_ser.move_to(p("bul")));
    phase.data.orders.push(unit_t_bul.move_to(p("tri")));
    phase.data.orders.push(unit_t_aeg.convoy(unit_t_bul, p("tri")));
    phase.data.orders.push(unit_t_ion.convoy(unit_t_bul, p("tri")));
    phase.data.orders.push(unit_t_adr.convoy(unit_t_bul, p("tri")));
    phase.data.orders.push(unit_i_nap.move_to(p("ion")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 7);
    assert!(phase.data.units.contains(&a("a", "ser")));
    assert!(phase.data.units.contains(&a("t", "tri")));
    assert!(phase.data.units.contains(&unit_t_aeg));
    assert!(phase.data.units.contains(&unit_t_ion));
    assert!(phase.data.units.contains(&unit_t_adr));
    assert!(phase.data.units.contains(&unit_i_nap));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.C.5. TEST CASE, A DISRUPTED CIRCULAR MOVEMENT DUE TO DISLODGED CONVOY
/// When the circular movement contains a convoy,
/// the circular movement is disrupted when the convoying fleet is dislodged.
/// The adjudication algorithm should disrupt convoys before calculating circular movement.
///
/// Austria:
///     A Trieste - Serbia
///     A Serbia - Bulgaria
///
/// Turkey:
///     A Bulgaria - Trieste
///     F Aegean Sea Convoys A Bulgaria - Trieste
///     F Ionian Sea Convoys A Bulgaria - Trieste
///     F Adriatic Sea Convoys A Bulgaria - Trieste
///
/// Italy:
///     F Naples - Ionian Sea
///     F Tunis Supports F Naples - Ionian Sea
///
/// Due to the dislodged convoying fleet, all Austrian and Turkish armies will not move.
#[test]
fn test_datc_6_c_5() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_a_tri = a("a", "tri");
    let unit_a_ser = a("a", "ser");
    let unit_t_bul = a("t", "bul");
    let unit_t_aeg = f("t", "aeg");
    let mut unit_t_ion = f("t", "ion");
    let unit_t_adr = f("t", "adr");
    let unit_i_nap = f("i", "nap");
    let unit_i_tun = f("i", "tun");
    phase.data.units.push(unit_a_tri);
    phase.data.units.push(unit_a_ser);
    phase.data.units.push(unit_t_bul);
    phase.data.units.push(unit_t_aeg);
    phase.data.units.push(unit_t_ion);
    phase.data.units.push(unit_t_adr);
    phase.data.units.push(unit_i_nap);
    phase.data.units.push(unit_i_tun);
    phase.data.orders.push(unit_a_tri.move_to(p("ser")));
    phase.data.orders.push(unit_a_ser.move_to(p("bul")));
    phase.data.orders.push(unit_t_bul.move_to(p("tri")));
    phase.data.orders.push(unit_t_aeg.convoy(unit_t_bul, p("tri")));
    phase.data.orders.push(unit_t_ion.convoy(unit_t_bul, p("tri")));
    phase.data.orders.push(unit_t_adr.convoy(unit_t_bul, p("tri")));
    phase.data.orders.push(unit_i_nap.move_to(p("ion")));
    phase.data.orders.push(unit_i_tun.support_move(unit_i_nap, p("ion")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Unreachable);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Dislodged);
    assert_eq!(phase.data.orders[5].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[6].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[7].status, OrderStatus::Valid);
    assert_eq!(phase.data.units.len(), 8);
    assert!(phase.data.units.contains(&unit_a_tri));
    assert!(phase.data.units.contains(&unit_a_ser));
    assert!(phase.data.units.contains(&unit_t_bul));
    assert!(phase.data.units.contains(&unit_t_aeg));
    assert!(phase.data.units.contains(&unit_t_ion.set_dislodged_from(Some(p("nap")))));
    assert!(phase.data.units.contains(&unit_t_adr));
    assert!(phase.data.units.contains(&f("i", "ion")));
    assert!(phase.data.units.contains(&unit_i_tun));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.C.6. TEST CASE, TWO ARMIES WITH TWO CONVOYS
/// Two armies can swap places even when they are not adjacent.
///
/// England:
///     F North Sea Convoys A London - Belgium
///     A London - Belgium
///
/// France:
///     F English Channel Convoys A Belgium - London
///     A Belgium - London
///
/// Both convoys should succeed.
#[test]
fn test_datc_6_c_6() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let unit_e_lon = a("e", "lon");
    let unit_f_eng = f("f", "eng");
    let unit_f_bel = a("f", "bel");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_lon);
    phase.data.units.push(unit_f_eng);
    phase.data.units.push(unit_f_bel);
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bel, p("lon")));
    phase.data.orders.push(unit_f_bel.move_to(p("lon")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
    assert_eq!(phase.data.units.len(), 4);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&a("e", "bel")));
    assert!(phase.data.units.contains(&unit_f_eng));
    assert!(phase.data.units.contains(&a("f", "lon")));
    assert!(phase.data.standoff_codes.is_empty());
}

/// 6.C.7. TEST CASE, DISRUPTED UNIT SWAP
/// If in a swap one of the unit bounces, then the swap fails.
///
/// England:
///     F North Sea Convoys A London - Belgium
///     A London - Belgium
///
/// France:
///     F English Channel Convoys A Belgium - London
///     A Belgium - London
///     A Burgundy - Belgium
///
/// None of the units will succeed to move.
#[test]
fn test_datc_6_c_7() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_e_nth = f("e", "nth");
    let unit_e_lon = a("e", "lon");
    let unit_f_eng = f("f", "eng");
    let unit_f_bel = a("f", "bel");
    let unit_f_bur = a("f", "bur");
    phase.data.units.push(unit_e_nth);
    phase.data.units.push(unit_e_lon);
    phase.data.units.push(unit_f_eng);
    phase.data.units.push(unit_f_bel);
    phase.data.units.push(unit_f_bur);
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bel, p("lon")));
    phase.data.orders.push(unit_f_bel.move_to(p("lon")));
    phase.data.orders.push(unit_f_bur.move_to(p("bel")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_e_nth));
    assert!(phase.data.units.contains(&unit_e_lon));
    assert!(phase.data.units.contains(&unit_f_eng));
    assert!(phase.data.units.contains(&unit_f_bel));
    assert!(phase.data.units.contains(&unit_f_bur));
    assert!(phase.data.standoff_codes.contains(&"bel".to_string()));
}

/// 6.C.8. TEST CASE, NO SELF DISLODGEMENT IN DISRUPTED CIRCULAR MOVEMENT
/// Self dislodgement is prohibited as usual in circular movement.
///
/// Turkey:
///     F Constantinople - Black Sea
///     A Bulgaria - Constantinople
///     A Smyrna Supports A Bulgaria - Constantinople
///
/// Russia:
///     F Black Sea - Bulgaria(ec)
///
/// Austria
///     A Serbia - Bulgaria
///
/// None of the units will succeed to move.
#[test]
fn test_datc_6_c_8() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_t_con = f("t", "con");
    let unit_t_bul = a("t", "bul");
    let unit_t_smy = a("t", "smy");
    let unit_r_bla = f("r", "bla");
    let unit_a_ser = a("a", "ser");
    phase.data.units.push(unit_t_con);
    phase.data.units.push(unit_t_bul);
    phase.data.units.push(unit_t_smy);
    phase.data.units.push(unit_r_bla);
    phase.data.units.push(unit_a_ser);
    phase.data.orders.push(unit_t_con.move_to(p("bla")));
    phase.data.orders.push(unit_t_bul.move_to(p("con")));
    phase.data.orders.push(unit_t_smy.support_move(unit_t_bul, p("con")));
    phase.data.orders.push(unit_r_bla.move_to(p("bul_ec")));
    phase.data.orders.push(unit_a_ser.move_to(p("bul")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_t_con));
    assert!(phase.data.units.contains(&unit_t_bul));
    assert!(phase.data.units.contains(&unit_t_smy));
    assert!(phase.data.units.contains(&unit_r_bla));
    assert!(phase.data.units.contains(&unit_a_ser));
    assert!(phase.data.standoff_codes.contains(&"bul".to_string()));
}

/// 6.C.9. TEST CASE, NO HELP IN DISLODGEMENT OF OWN UNIT IN DISRUPTED CIRCULAR MOVEMENT
/// Helping to dislodge your own unit is prohibited as usual in circular movement.
///
/// Turkey:
///     F Constantinople - Black Sea
///     A Smyrna Supports A Bulgaria - Constantinople
///
/// Russia:
///     F Black Sea - Bulgaria(ec)
///
/// Austria
///     A Serbia - Bulgaria
///     A Bulgaria - Constantinople
///
/// None of the units will succeed to move.
#[test]
fn test_datc_6_c_9() {
    let mut phase = Phase::new_spring_main(1900, 1);
    let unit_t_con = f("t", "con");
    let unit_t_smy = a("t", "smy");
    let unit_r_bla = f("r", "bla");
    let unit_a_ser = a("a", "ser");
    let unit_a_bul = a("a", "bul");
    phase.data.units.push(unit_t_con);
    phase.data.units.push(unit_t_smy);
    phase.data.units.push(unit_r_bla);
    phase.data.units.push(unit_a_ser);
    phase.data.units.push(unit_a_bul);
    phase.data.orders.push(unit_t_con.move_to(p("bla")));
    phase.data.orders.push(unit_t_smy.support_move(unit_a_bul, p("con")));
    phase.data.orders.push(unit_r_bla.move_to(p("bul_ec")));
    phase.data.orders.push(unit_a_ser.move_to(p("bul")));
    phase.data.orders.push(unit_a_bul.move_to(p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
    assert_eq!(phase.data.units.len(), 5);
    assert!(phase.data.units.contains(&unit_t_con));
    assert!(phase.data.units.contains(&unit_t_smy));
    assert!(phase.data.units.contains(&unit_r_bla));
    assert!(phase.data.units.contains(&unit_a_ser));
    assert!(phase.data.units.contains(&unit_a_bul));
    assert!(phase.data.standoff_codes.contains(&"bul".to_string()));
}
