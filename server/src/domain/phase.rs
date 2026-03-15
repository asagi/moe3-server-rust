use super::Order;
use super::PhaseId;
use super::Unit;
use super::phase_order_resolution::resolve_orders_for_order_phase;
use serde::Deserialize;
use serde::Serialize;

/// フェイズの定義
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Phase {
    pub id: Option<PhaseId>,
    pub index: i32,
    pub year: i32,
    pub phase_type: PhaseType,
    pub orders: Vec<Order>,
    pub units: Vec<Unit>,
}

/// フェイズの種類
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhaseType {
    Ready(ReadyPhase),                 // 準備
    SpringOrder(SpringOrderPhase),     // 春命令
    SpringRetreat(SpringRetreatPhase), // 春撤退
    FallOrder(FallOrderPhase),         // 秋命令
    FallRetreat(FallRetreatPhase),     // 秋撤退
    Adjustment(AdjustmentPhase),       // 調整
    Debrief(DebriefPhase),             // 感想戦
}

/// 各フェイズの詳細な構造体（必要に応じてフィールドを追加）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ReadyPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SpringOrderPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SpringRetreatPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FallOrderPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FallRetreatPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdjustmentPhase {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DebriefPhase {}

/// フェイズのロジック
impl Phase {
    fn new(year: i32, index: i32, phase_type: PhaseType) -> Self {
        Self {
            id: None,
            index,
            year,
            phase_type,
            orders: Vec::new(),
            units: Vec::new(),
        }
    }

    /// 準備フェイズを生成する。
    pub fn new_ready() -> Self {
        Self::new(1900, 0, PhaseType::Ready(ReadyPhase {}))
    }

    /// 春命令フェイズを生成する。
    ///
    /// - 春命令は「次年の開始フェイズ」なので、必ず `year = prev_year + 1`。
    /// - このルールは Ready -> SpringOrder / Adjustment -> SpringOrder の両方で共通。
    pub fn new_spring_order(current_year: i32, current_index: i32) -> Self {
        Self::new(current_year + 1, current_index + 1, PhaseType::SpringOrder(SpringOrderPhase {}))
    }

    /// 春撤退フェイズを生成する。
    pub fn new_spring_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::SpringRetreat(SpringRetreatPhase {}))
    }

    /// 秋命令フェイズを生成する。
    pub fn new_fall_order(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::FallOrder(FallOrderPhase {}))
    }

    /// 秋撤退フェイズを生成する。
    pub fn new_fall_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::FallRetreat(FallRetreatPhase {}))
    }

    /// 調整フェイズを生成する。
    pub fn new_adjustment(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::Adjustment(AdjustmentPhase {}))
    }

    /// 感想戦フェイズを生成する。
    pub fn new_debrief(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::Debrief(DebriefPhase {}))
    }

    /// フェイズを締め切り命令を解決する。
    pub fn close(mut self, context: &mut PhaseContext) -> PhaseCloseResult {
        match self.phase_type.clone() {
            PhaseType::Ready(r) => r.close(&mut self, context),
            PhaseType::SpringOrder(s) => s.close(&mut self, context),
            PhaseType::SpringRetreat(s) => s.close(&mut self, context),
            PhaseType::FallOrder(f) => f.close(&mut self, context),
            PhaseType::FallRetreat(f) => f.close(&mut self, context),
            PhaseType::Adjustment(a) => a.close(&mut self, context),
            PhaseType::Debrief(d) => d.close(&mut self, context),
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

/// 命令フェイズの和平合意条件の判定
fn check_draw_condition_for_order_phase(_context: &PhaseContext) -> bool {
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

/// 各フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for ReadyPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_order(current_phase.year, current_phase.index))
    }
}

impl PhaseCloseLogic for SpringOrderPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_retreat(current_phase.year, current_phase.index))
    }

    fn check_draw_condition(&self, _context: &PhaseContext) -> bool {
        check_draw_condition_for_order_phase(_context)
    }

    fn resolve_orders(&self, _current_phase: &mut Phase, context: &mut PhaseContext) {
        resolve_orders_for_order_phase(_current_phase, context);
    }

    /// 撤退指示が必要なユニットが存在しない場合に true を返す
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        // TODO
        false
    }
}

impl PhaseCloseLogic for SpringRetreatPhase {
    fn occupy(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {
        occupy_for_retreat_phase(_current_phase, _context);
    }

    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_fall_order(current_phase.year, current_phase.index))
    }
}

impl PhaseCloseLogic for FallOrderPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_fall_retreat(current_phase.year, current_phase.index))
    }

    fn check_draw_condition(&self, _context: &PhaseContext) -> bool {
        check_draw_condition_for_order_phase(_context)
    }

    fn resolve_orders(&self, _current_phase: &mut Phase, context: &mut PhaseContext) {
        resolve_orders_for_order_phase(_current_phase, context);
    }

    /// 撤退指示が必要なユニットが存在しない場合に true を返す
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        // TODO
        false
    }
}

impl PhaseCloseLogic for FallRetreatPhase {
    fn occupy(&self, _current_phase: &mut Phase, _context: &mut PhaseContext) {
        occupy_for_retreat_phase(_current_phase, _context);
    }

    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_adjustment(current_phase.year, current_phase.index))
    }

    /// 調整が不要な場合に true を返す
    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        // TODO
        false
    }
}

impl PhaseCloseLogic for AdjustmentPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_order(current_phase.year, current_phase.index))
    }
}

impl PhaseCloseLogic for DebriefPhase {
    fn create_next_phase(&self, _current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        None
    }
}

/// フェイズのコンテキストと終了結果（必要に応じてフィールドを追加）
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseContext {
    pub phases: Vec<Phase>,
}

impl PhaseContext {
    pub fn finalize(&mut self, latest_phase: &Phase) -> PhaseCloseResult {
        self.phases.push(latest_phase.clone());
        self.to_result()
    }

    fn to_result(&self) -> PhaseCloseResult {
        PhaseCloseResult {}
    }
}

/// フェイズの終了結果（必要に応じてフィールドを追加）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCloseResult {}

#[cfg(test)]
mod tests {
    use super::super::phase_order_resolution::test_hook;
    use super::*;

    #[test]
    fn spec_new_spring_order_always_increments_year() {
        let p = Phase::new_spring_order(1900, 7);
        assert_eq!(p.year, 1901);
        assert_eq!(p.index, 8);
        assert!(matches!(p.phase_type, PhaseType::SpringOrder(_)));
    }

    #[test]
    fn spring_order_close_calls_common_order_resolution() {
        test_hook::reset();

        let phase = Phase::new_spring_order(1900, 0);
        let mut context = PhaseContext { phases: vec![] };

        let _ = phase.close(&mut context);

        assert_eq!(test_hook::call_count(), 1);
    }

    #[test]
    fn fall_order_close_calls_common_order_resolution() {
        test_hook::reset();

        let phase = Phase::new_fall_order(1901, 1);
        let mut context = PhaseContext { phases: vec![] };

        let _ = phase.close(&mut context);

        assert_eq!(test_hook::call_count(), 1);
    }
}
