use super::super::phase::order_resolution::*;
use super::super::phase::*;

use super::order_resolution::test_hook;

#[test]
fn test_new_spring_order_always_increments_year() {
    let p = Phase::new_spring_order(1900, 7);
    assert_eq!(p.year(), 1901);
    assert_eq!(p.index(), 8);
    assert!(matches!(p.phase_type(), PhaseKind::SpringOrder(_)));
}

#[test]
fn test_spring_order_close_calls_common_order_resolution() {
    test_hook::reset();

    let phase = Phase::new_spring_order(1900, 0);
    let mut context = PhaseContext {
        phases: vec![],
        standoff_provinces: vec![],
    };

    let _ = phase.close(&mut context);

    assert_eq!(test_hook::call_count(), 1);
}

#[test]
fn test_fall_order_close_calls_common_order_resolution() {
    test_hook::reset();

    let phase = Phase::new_fall_order(1901, 1);
    let mut context = PhaseContext {
        phases: vec![],
        standoff_provinces: vec![],
    };

    let _ = phase.close(&mut context);

    assert_eq!(test_hook::call_count(), 1);
}

#[test]
fn test_resolve_orders_for_order_phase_calls_test_hook() {
    test_hook::reset();
    let mut phase = Phase::new_spring_order(1900, 1);
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(test_hook::call_count(), 1);
}
