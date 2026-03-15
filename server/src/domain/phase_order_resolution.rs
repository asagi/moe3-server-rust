use super::phase::Phase;
use super::phase::PhaseContext;

/// 命令フェイズの命令解決処理
///
/// 方針（暫定）:
/// - 現在は戻り値を持たず、`current_phase` への副作用で結果を反映する。
/// - 入力の主対象は `current_phase.orders`。
/// - 解決結果は `current_phase.units` や（将来的に）スタンドオフ情報へ書き戻す。
/// - `context` は参照用（過去フェイズ参照など）を基本とし、不要な更新は避ける。
/// - I/O は行わず、同じ入力に対して同じ結果になる決定的な処理を維持する。
pub fn resolve_orders_for_order_phase(_current_phase: &mut Phase, _context: &mut PhaseContext) {
    #[cfg(test)]
    {
        test_hook::mark_called();
    }

    // TODO: 命令の解決処理
    // - 行軍命令を解決し、スタンドオフが発生した地域を記録する
    // - 解決済み命令からユニットを生成して current_phase.units に追加する
    // - スタンドオフ情報を current_phase に記録する（シグネチャ変更予定）
}

#[cfg(test)]
pub(crate) mod test_hook {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

    pub(crate) fn reset() {
        CALL_COUNT.store(0, Ordering::SeqCst);
    }

    pub(crate) fn mark_called() {
        CALL_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn call_count() -> usize {
        CALL_COUNT.load(Ordering::SeqCst)
    }
}
