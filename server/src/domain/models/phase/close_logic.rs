use super::AdjustmentPhase;
use super::DebriefPhase;
use super::FallMainPhase;
use super::FallRetreatPhase;
use super::Phase;
use super::PhaseCloseResult;
use super::PhaseContext;
use super::PhaseKind;
use super::ReadyPhase;
use super::SpringMainPhase;
use super::SpringRetreatPhase;
use super::resolvers::resolve_orders_for_adjustment_phase;
use super::resolvers::resolve_orders_for_main_phase;
use super::resolvers::resolve_orders_for_retreat_phase;

/// フェイズのロジック
impl Phase {
    /// フェイズを締め切り命令を解決する。
    pub fn close(mut self, context: &mut PhaseContext) -> PhaseCloseResult {
        match self.data.kind {
            PhaseKind::Ready(r) => r.close(&mut self, context),
            PhaseKind::SpringMain(s) => s.close(&mut self, context),
            PhaseKind::SpringRetreat(s) => s.close(&mut self, context),
            PhaseKind::FallMain(f) => f.close(&mut self, context),
            PhaseKind::FallRetreat(f) => f.close(&mut self, context),
            PhaseKind::Adjustment(a) => a.close(&mut self, context),
            PhaseKind::Debrief(d) => d.close(&mut self, context),
        }
    }
}

/// フェイズの終了ロジック
trait PhaseCloseLogic {
    fn close(&self, current_phase: &mut Phase, context: &mut PhaseContext) -> PhaseCloseResult
    where
        Self: Sized,
    {
        // 和平判定
        if self.check_draw_condition(context) {
            return self.close_on_draw(current_phase, context);
        }

        // 解決
        self.resolve_orders(current_phase, context);
        self.occupy(current_phase, context);

        // 制覇判定
        if self.check_resolved_condition(context) {
            return self.close_on_resolution(current_phase, context);
        }

        // 次フェイズ生成
        if let Some(next_phase) = self.create_next_phase(current_phase, context) {
            context.phases.push(current_phase.clone());

            // スキップ判定と再帰
            if self.should_skip_next_phase(context, &next_phase) {
                return next_phase.close(context);
            }
            return context.finalize(&next_phase);
        }
        context.finalize(current_phase)
    }

    fn check_draw_condition(&self, _context: &PhaseContext) -> bool {
        false
    }

    fn close_on_draw(&self, current_phase: &mut Phase, context: &mut PhaseContext) -> PhaseCloseResult {
        // TODO: 和平合意による終了処理
        // - テーブルのステータスを DRAW に変更する
        // - 現在のフェイズ種別に応じて後続フェイズを生成して context.phases に積む
        //   （春命令中なら撤退フェイズ、秋命令中なら撤退→調整フェイズ）
        // - 最後に感想戦フェイズを生成して積む
        // - 期限時刻を考慮して感想戦フェイズの due_time を設定する

        // TODO: 暫定実装
        // 本来は後続フェイズと Debrief を context.phases に積んだうえで結果化する
        context.finalize(current_phase)
    }

    fn resolve_orders(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {}

    fn occupy(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {}

    fn check_resolved_condition(&self, _context: &PhaseContext) -> bool {
        false
    }

    fn close_on_resolution(&self, current_phase: &mut Phase, context: &mut PhaseContext) -> PhaseCloseResult {
        // TODO: 制覇勝利による終了処理
        // - テーブルのステータスを RESOLVED に変更する
        // - 現在のフェイズ種別に応じて後続フェイズを生成して context.phases に積む
        //   （close_on_draw と同様）
        // - 最後に感想戦フェイズを生成して積む

        // TODO: 暫定実装
        // 本来は後続フェイズと Debrief を context.phases に積んだうえで結果化する
        context.finalize(current_phase)
    }

    fn create_next_phase(&self, current_phase: &Phase, context: &mut PhaseContext) -> Option<Phase>;

    /// 次フェイズがスキップ可能な場合に true を返す
    /// true を返す可能性のある場合にのみオーバーライドする
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        false
    }
}

/// メインフェイズの和平合意条件の判定
fn check_draw_condition_for_main_phase(_context: &PhaseContext) -> bool {
    // TODO: 和平合意条件の判定
    // - 有効な勢力のうち、和平に同意しているプレイヤーが過半数を超えたら true を返す

    false
}

/// 撤退フェイズの占領処理
fn occupy_for_retreat_phase(_current_phase: &mut Phase, _context: &mut PhaseContext) {
    // TODO: 占領処理
    // - 撤退命令を解決し、占領が発生した地域を記録する
    // - 占領情報を current_phase に記録する
}

/// 準備フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for ReadyPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_main(current_phase.year(), current_phase.index()))
    }
}

/// 春メインフェイズの終了ロジックの差分実装
impl PhaseCloseLogic for SpringMainPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_retreat(current_phase.year(), current_phase.index()))
    }

    fn check_draw_condition(&self, _context: &PhaseContext) -> bool {
        check_draw_condition_for_main_phase(_context)
    }

    fn resolve_orders(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {
        resolve_orders_for_main_phase(_current_phase);
    }

    /// 撤退指示が必要なユニットが存在しない場合に true を返す
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        // TODO
        false
    }
}

/// 春撤退フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for SpringRetreatPhase {
    fn resolve_orders(&self, current_phase: &mut Phase, context: &mut PhaseContext) {
        resolve_orders_for_retreat_phase(current_phase, context);
    }

    fn occupy(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {
        occupy_for_retreat_phase(_current_phase, _context);
    }

    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_fall_main(current_phase.year(), current_phase.index()))
    }
}

/// 秋メインフェイズの終了ロジックの差分実装
impl PhaseCloseLogic for FallMainPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_fall_retreat(current_phase.year(), current_phase.index()))
    }

    fn check_draw_condition(&self, _context: &PhaseContext) -> bool {
        check_draw_condition_for_main_phase(_context)
    }

    fn resolve_orders(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {
        resolve_orders_for_main_phase(_current_phase);
    }

    /// 撤退指示が必要なユニットが存在しない場合に true を返す
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        // TODO
        false
    }
}

/// 秋撤退フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for FallRetreatPhase {
    fn resolve_orders(&self, _current_phase: &mut Phase, context: &mut PhaseContext) {
        resolve_orders_for_retreat_phase(_current_phase, context);
    }

    fn occupy(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {
        occupy_for_retreat_phase(_current_phase, _context);
    }

    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_adjustment(current_phase.year(), current_phase.index()))
    }

    /// 調整が不要な場合に true を返す
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        // TODO
        false
    }
}

/// 調整フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for AdjustmentPhase {
    fn resolve_orders(&self, current_phase: &mut Phase, _context: &mut PhaseContext) {
        resolve_orders_for_adjustment_phase(current_phase);
    }

    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_main(current_phase.year(), current_phase.index()))
    }
}

/// 感想戦フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for DebriefPhase {
    fn create_next_phase(&self, _current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        None
    }
}
