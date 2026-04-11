mod adjustment_adjudicator;
mod adjustment_order_helper;
mod adjustment_order_resolution;
mod main_adjudicator;
mod main_order_helper;
mod main_order_resolution;
mod retreat_adjudicator;
mod retreat_order_helper;
mod retreat_order_resolution;

use super::PhaseId;
use super::TableId;
use super::order::Order;
use super::power::Power;
use super::territory::Territory;
use super::unit::Unit;
use adjustment_order_resolution::resolve_orders_for_adjustment_phase;
use chrono::DateTime;
use chrono::Utc;
use main_order_resolution::resolve_orders_for_main_phase;
use retreat_order_resolution::resolve_orders_for_retreat_phase;
use serde::Deserialize;
use serde::Serialize;

/// フェイズの定義
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Phase {
    pub id: Option<PhaseId>,
    pub table_id: Option<TableId>,
    pub created_at: Option<DateTime<Utc>>,
    pub data: PhaseData,
}

/// フェイズのデータ本体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PhaseData {
    pub index: i32,
    pub year: i32,
    #[serde(flatten)]
    pub kind: PhaseKind,
    pub orders: Vec<Order>,
    pub units: Vec<Unit>,
    pub territories: Vec<Territory>,
    pub standoff_codes: Vec<String>,
}

/// フェイズの種類
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhaseKind {
    Ready(ReadyPhase),                 // 準備
    SpringMain(SpringMainPhase),       // 春命令
    SpringRetreat(SpringRetreatPhase), // 春撤退
    FallMain(FallMainPhase),           // 秋命令
    FallRetreat(FallRetreatPhase),     // 秋撤退
    Adjustment(AdjustmentPhase),       // 調整
    Debrief(DebriefPhase),             // 感想戦
}

/// 各フェイズの詳細な構造体（必要に応じてフィールドを追加）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ReadyPhase {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SpringMainPhase {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SpringRetreatPhase {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FallMainPhase {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct FallRetreatPhase {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdjustmentPhase {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct DebriefPhase {}

/// フェイズのロジック
impl Phase {
    fn new(year: i32, index: i32, phase_type: PhaseKind) -> Self {
        Self {
            id: None,
            table_id: None,
            created_at: None,
            data: PhaseData {
                index,
                year,
                kind: phase_type,
                orders: Vec::new(),
                units: Vec::new(),
                territories: Vec::new(),
                standoff_codes: Vec::new(),
            },
        }
    }

    /// 準備フェイズを生成する。
    pub fn new_ready() -> Self {
        Self::new(1900, 0, PhaseKind::Ready(ReadyPhase {}))
    }

    /// 春メインフェイズを生成する。
    ///
    /// - 春命令は「次年の開始フェイズ」なので、必ず `year = prev_year + 1`。
    /// - このルールは Ready -> SpringOrder / Adjustment -> SpringOrder の両方で共通。
    pub fn new_spring_main(current_year: i32, current_index: i32) -> Self {
        Self::new(current_year + 1, current_index + 1, PhaseKind::SpringMain(SpringMainPhase {}))
    }

    /// 春撤退フェイズを生成する。
    pub fn new_spring_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::SpringRetreat(SpringRetreatPhase {}))
    }

    /// 秋メインフェイズを生成する。
    pub fn new_fall_main(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::FallMain(FallMainPhase {}))
    }

    /// 秋撤退フェイズを生成する。
    pub fn new_fall_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::FallRetreat(FallRetreatPhase {}))
    }

    /// 調整フェイズを生成する。
    pub fn new_adjustment(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::Adjustment(AdjustmentPhase {}))
    }

    /// 感想戦フェイズを生成する。
    pub fn new_debrief(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseKind::Debrief(DebriefPhase {}))
    }

    /// フェイズの年を返す
    pub fn year(&self) -> i32 {
        self.data.year
    }

    /// フェイズ内での通し番号を返す
    pub fn index(&self) -> i32 {
        self.data.index
    }

    /// フェイズの種別を返す
    pub fn phase_type(&self) -> PhaseKind {
        self.data.kind
    }

    /// 指定した国が現在保有する補給都市数を取得する
    pub fn count_supply_centers(&self, power: &Power) -> usize {
        self.data.territories.iter().filter(|t| t.power() == power).count()
    }

    /// 指定した国が現在保有するユニット数を取得する
    pub fn count_units(&self, power: &Power) -> usize {
        self.data.units.iter().filter(|u| &u.power == power).count()
    }

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

/// 各フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for ReadyPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_main(current_phase.year(), current_phase.index()))
    }
}

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

impl PhaseCloseLogic for AdjustmentPhase {
    fn resolve_orders(&self, current_phase: &mut Phase, context: &mut PhaseContext) {
        resolve_orders_for_adjustment_phase(current_phase, context);
    }

    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_spring_main(current_phase.year(), current_phase.index()))
    }
}

impl PhaseCloseLogic for DebriefPhase {
    fn create_next_phase(&self, _current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        None
    }
}

/// フェイズのコンテキストと終了結果（必要に応じてフィールドを追加）
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhaseContext {
    pub phases: Vec<Phase>,
    pub(crate) standoff_codes: Vec<&'static str>,
    pub(crate) last_resolved_units: Vec<Unit>,
}

impl PhaseContext {
    pub fn new() -> Self {
        Self {
            phases: Vec::new(),
            standoff_codes: Vec::new(),
            last_resolved_units: Vec::new(),
        }
    }

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
    use super::*;

    #[test]
    fn test_new_spring_order_always_increments_year() {
        let p = Phase::new_spring_main(1900, 7);
        assert_eq!(p.year(), 1901);
        assert_eq!(p.index(), 8);
        assert!(matches!(p.phase_type(), PhaseKind::SpringMain(_)));
    }
}

#[cfg(test)]
mod test_datc_6_a;
#[cfg(test)]
mod test_datc_6_b;
#[cfg(test)]
mod test_datc_6_c;
#[cfg(test)]
mod test_datc_6_d;
#[cfg(test)]
mod test_datc_6_e;
#[cfg(test)]
mod test_datc_6_f;
#[cfg(test)]
mod test_datc_6_g;
#[cfg(test)]
mod test_datc_6_h;
#[cfg(test)]
mod test_datc_6_i;
