use super::super::phase::order_resolution::*;
use super::super::phase::*;

use super::order_resolution::test_hook;

#[test]
fn spec_new_spring_order_always_increments_year() {
    let p = Phase::new_spring_order(1900, 7);
    assert_eq!(p.year(), 1901);
    assert_eq!(p.index(), 8);
    assert!(matches!(p.phase_type(), PhaseKind::SpringOrder(_)));
}

#[test]
fn spring_order_close_calls_common_order_resolution() {
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
fn fall_order_close_calls_common_order_resolution() {
    test_hook::reset();

    let phase = Phase::new_fall_order(1901, 1);
    let mut context = PhaseContext {
        phases: vec![],
        standoff_provinces: vec![],
    };

    let _ = phase.close(&mut context);

    assert_eq!(test_hook::call_count(), 1);
}

fn make_simple_phase() -> Phase {
    // テスト用ビルダーをここに実装（省略）
    Phase::new_spring_order(1900, 1)
}

#[test]
fn resolves_calls_main_entry() {
    test_hook::reset();
    let mut phase = make_simple_phase();
    resolve_orders_for_order_phase(&mut phase);
    assert_eq!(test_hook::call_count(), 1);
}

// 以降、ヘルパーを使って多くのケースを追加
