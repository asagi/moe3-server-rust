//! # 6. TEST CASES
//!
//! 参照先: [DATC v3.0 Section 6][DATC_6C]
//!
//! * 6.C. TEST CASES, CIRCULAR MOVEMENT
//!
//! [DATC_6C]: https://webdiplomacy.net/doc/DATC_v3_0.html#6.C

use super::super::order::*;
use super::super::phase::main_order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

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
    let unit_t_ank = Unit::new_fleet(Power::Turkey, p("ank"));
    let unit_t_con = Unit::new_army(Power::Turkey, p("con"));
    let unit_t_smy = Unit::new_army(Power::Turkey, p("smy"));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("smy")));
    phase.data.orders.push(unit_t_smy.move_to(p("ank")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
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
    let unit_t_ank = Unit::new_fleet(Power::Turkey, p("ank"));
    let unit_t_con = Unit::new_army(Power::Turkey, p("con"));
    let unit_t_smy = Unit::new_army(Power::Turkey, p("smy"));
    let unit_t_bul = Unit::new_army(Power::Turkey, p("bul"));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("smy")));
    phase.data.orders.push(unit_t_smy.move_to(p("ank")));
    phase.data.orders.push(unit_t_bul.support_move(unit_t_ank, p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Valid);
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
    let unit_t_ank = Unit::new_fleet(Power::Turkey, p("ank"));
    let unit_t_con = Unit::new_army(Power::Turkey, p("con"));
    let unit_t_smy = Unit::new_army(Power::Turkey, p("smy"));
    let unit_t_bul = Unit::new_army(Power::Turkey, p("bul"));
    phase.data.orders.push(unit_t_ank.move_to(p("con")));
    phase.data.orders.push(unit_t_con.move_to(p("smy")));
    phase.data.orders.push(unit_t_smy.move_to(p("ank")));
    phase.data.orders.push(unit_t_bul.move_to(p("con")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
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
    let unit_a_tri = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_ser = Unit::new_army(Power::Austria, p("ser"));
    let unit_t_bul = Unit::new_army(Power::Turkey, p("bul"));
    let unit_t_aeg = Unit::new_fleet(Power::Turkey, p("aeg"));
    let unit_t_ion = Unit::new_fleet(Power::Turkey, p("ion"));
    let unit_t_adr = Unit::new_fleet(Power::Turkey, p("adr"));
    let unit_i_nap = Unit::new_fleet(Power::Italy, p("nap"));
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
    let unit_a_tri = Unit::new_army(Power::Austria, p("tri"));
    let unit_a_ser = Unit::new_army(Power::Austria, p("ser"));
    let unit_t_bul = Unit::new_army(Power::Turkey, p("bul"));
    let unit_t_aeg = Unit::new_fleet(Power::Turkey, p("aeg"));
    let unit_t_ion = Unit::new_fleet(Power::Turkey, p("ion"));
    let unit_t_adr = Unit::new_fleet(Power::Turkey, p("adr"));
    let unit_i_nap = Unit::new_fleet(Power::Italy, p("nap"));
    let unit_i_tun = Unit::new_fleet(Power::Italy, p("tun"));
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
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_bel = Unit::new_army(Power::France, p("bel"));
    phase.data.orders.push(unit_e_nth.convoy(unit_e_lon, p("bel")));
    phase.data.orders.push(unit_e_lon.move_to(p("bel")));
    phase.data.orders.push(unit_f_eng.convoy(unit_f_bel, p("lon")));
    phase.data.orders.push(unit_f_bel.move_to(p("lon")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Success);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Success);
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
    let unit_e_nth = Unit::new_fleet(Power::England, p("nth"));
    let unit_e_lon = Unit::new_army(Power::England, p("lon"));
    let unit_f_eng = Unit::new_fleet(Power::France, p("eng"));
    let unit_f_bel = Unit::new_army(Power::France, p("bel"));
    let unit_f_bur = Unit::new_army(Power::France, p("bur"));
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
    let unit_t_1 = Unit::new_fleet(Power::Turkey, p("con"));
    let unit_t_2 = Unit::new_army(Power::Turkey, p("bul"));
    let unit_t_3 = Unit::new_army(Power::Turkey, p("smy"));
    let unit_r_1 = Unit::new_fleet(Power::Russia, p("bla"));
    let unit_a_1 = Unit::new_army(Power::Austria, p("ser"));
    phase.data.orders.push(unit_t_1.move_to(p("bla")));
    phase.data.orders.push(unit_t_2.move_to(p("con")));
    phase.data.orders.push(unit_t_3.support_move(unit_t_2, p("con")));
    phase.data.orders.push(unit_r_1.move_to(p("bul_ec")));
    phase.data.orders.push(unit_a_1.move_to(p("bul")));
    resolve_orders_for_main_phase(&mut phase);
    assert_eq!(phase.data.orders[0].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[1].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[2].status, OrderStatus::Valid);
    assert_eq!(phase.data.orders[3].status, OrderStatus::Failure);
    assert_eq!(phase.data.orders[4].status, OrderStatus::Failure);
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
    let unit_t_con = Unit::new_fleet(Power::Turkey, p("con"));
    let unit_t_smy = Unit::new_army(Power::Turkey, p("smy"));
    let unit_r_bla = Unit::new_fleet(Power::Russia, p("bla"));
    let unit_a_ser = Unit::new_army(Power::Austria, p("ser"));
    let unit_a_bul = Unit::new_army(Power::Austria, p("bul"));
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
}
