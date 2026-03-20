use crate::domain::phase::order_resolution::*;
use crate::domain::phase::*;

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
