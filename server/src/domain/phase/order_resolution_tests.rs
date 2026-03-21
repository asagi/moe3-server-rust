use super::super::phase::*;

#[test]
fn test_new_spring_order_always_increments_year() {
    let p = Phase::new_spring_order(1900, 7);
    assert_eq!(p.year(), 1901);
    assert_eq!(p.index(), 8);
    assert!(matches!(p.phase_type(), PhaseKind::SpringOrder(_)));
}
