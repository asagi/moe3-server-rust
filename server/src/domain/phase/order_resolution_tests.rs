use super::super::phase::order_resolution::*;
use super::super::phase::*;
use super::super::power::*;
use super::super::province::*;
use super::super::unit::*;

fn p(code: &str) -> Province {
    Province::from_code(code).expect("valid province code")
}

#[test]
fn test_new_spring_order_always_increments_year() {
    let p = Phase::new_spring_order(1900, 7);
    assert_eq!(p.year(), 1901);
    assert_eq!(p.index(), 8);
    assert!(matches!(p.phase_type(), PhaseKind::SpringOrder(_)));
}

// 6. TEST CASES
//    https://webdiplomacy.net/doc/DATC_v3_0.html#6
//
// 6.A. TEST CASES, BASIC CHECKS

/// 6.A.1. TEST CASE, MOVING TO AN AREA THAT IS NOT A NEIGHBOUR
///  Check if an illegal move (without convoy) will fail.
///
///  England:
///  F North Sea - Picardy
///  Order should fail.
#[test]
fn test_illegal_move_without_convoy_fails() {
    let mut phase = Phase::new_spring_order(1900, 1);
    let order = Order::new_move(Power::England, Unit::new_fleet(Power::England, p("nth")), p("pic"));
    phase.data.orders.push(order);
    resolve_orders_for_order_phase(&mut phase);
    assert!(phase.data.orders[0].is_invalid());
}
