use super::Order;
use super::PhaseId;
use super::Unit;
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
    pub fn new_base(year: i32, index: i32, phase_type: PhaseType) -> Self {
        Self {
            id: None,
            index,
            year,
            phase_type,
            orders: Vec::new(),
            units: Vec::new(),
        }
    }

    pub fn get_next(&self) -> Option<Phase> {
        match &self.phase_type {
            PhaseType::Ready(_s) => None,
            PhaseType::SpringOrder(_s) => None,
            PhaseType::SpringRetreat(_s) => None,
            PhaseType::FallOrder(_s) => None,
            PhaseType::FallRetreat(_s) => None,
            PhaseType::Adjustment(_s) => None,
            PhaseType::Debrief(_s) => None,
        }
    }

    pub fn close(self, context: &mut PhaseContext, result: PhaseCloseResult) -> PhaseCloseResult {
        match self.phase_type {
            PhaseType::Ready(r) => r.close(self.year, self.index, context, result),
            PhaseType::SpringOrder(s) => s.close(self.year, self.index, context, result),
            PhaseType::SpringRetreat(s) => s.close(self.year, self.index, context, result),
            PhaseType::FallOrder(f) => f.close(self.year, self.index, context, result),
            PhaseType::FallRetreat(f) => f.close(self.year, self.index, context, result),
            PhaseType::Adjustment(a) => a.close(self.year, self.index, context, result),
            PhaseType::Debrief(d) => d.close(self.year, self.index, context, result),
        }
    }
}

/// 各フェイズの生成ロジック
impl ReadyPhase {
    pub fn new() -> Phase {
        Phase::new_base(1900, 0, PhaseType::Ready(ReadyPhase {}))
    }
}

impl SpringOrderPhase {
    pub fn new(prev_year: i32, prev_index: i32) -> Phase {
        Phase::new_base(prev_year + 1, prev_index + 1, PhaseType::SpringOrder(SpringOrderPhase {}))
    }
}

impl SpringRetreatPhase {
    pub fn new(current_year: i32, prev_index: i32) -> Phase {
        Phase::new_base(current_year, prev_index + 1, PhaseType::SpringRetreat(SpringRetreatPhase {}))
    }
}

impl FallOrderPhase {
    pub fn new(current_year: i32, prev_index: i32) -> Phase {
        Phase::new_base(current_year, prev_index + 1, PhaseType::FallOrder(FallOrderPhase {}))
    }
}

impl FallRetreatPhase {
    pub fn new(current_year: i32, prev_index: i32) -> Phase {
        Phase::new_base(current_year, prev_index + 1, PhaseType::FallRetreat(FallRetreatPhase {}))
    }
}

impl AdjustmentPhase {
    pub fn new(current_year: i32, prev_index: i32) -> Phase {
        Phase::new_base(current_year, prev_index + 1, PhaseType::Adjustment(AdjustmentPhase {}))
    }
}

impl DebriefPhase {
    pub fn new(current_year: i32, prev_index: i32) -> Phase {
        Phase::new_base(current_year, prev_index + 1, PhaseType::Debrief(DebriefPhase {}))
    }
}

/// フェイズの終了ロジック
pub trait PhaseCloseLogic {
    fn check_draw_condition(&self) -> bool {
        false
    }
    // fn end_in_draw(&self) -> Result<PhaseCloseResult, Error>;
    // fn resolve_orders(&mut self);
    // fn occupy(&mut self);
    // fn check_resolved_condition(&self) -> bool;
    // fn end_due_to_resolution(&self) -> Result<PhaseCloseResult, Error>;
    fn create_next_phase(&self, current_year: i32, current_index: i32, context: &mut PhaseContext) -> Option<Phase>;
    // fn should_skip_next_phase(&self) -> bool;
    // fn open(self) -> Self;

    fn close(self, current_year: i32, current_index: i32, context: &mut PhaseContext, result: PhaseCloseResult) -> PhaseCloseResult
    where
        Self: Sized,
    {
        //     // 1. 和平判定
        //     if self.check_draw_condition() {
        //         return self.end_in_draw();
        //     }

        //     // 2. 解決（selfの状態を書き換える）
        //     self.resolve_orders();
        //     self.occupy();

        //     // 3. 制覇判定
        //     if self.check_resolved_condition() {
        //         return self.end_due_to_resolution();
        //     }

        // 4. 次フェイズ生成
        if let Some(next_phase) = self.create_next_phase(current_year, current_index, context) {
            //     // 5. スキップ判定と再帰
            //     if !next_phase.should_skip_next_phase(active_powers) {
            //         return next_phase.open();
            //     }

            // 次のフェイズをそのまま close() にかける（再帰）
            return next_phase.close(context, result);
        }
        result
    }
}

/// 各フェイズの終了ロジックの差分実装
impl PhaseCloseLogic for ReadyPhase {
    fn create_next_phase(&self, current_year: i32, current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        Some(SpringOrderPhase::new(current_year, current_index))
    }
}

impl PhaseCloseLogic for SpringOrderPhase {
    fn create_next_phase(&self, current_year: i32, current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        Some(SpringRetreatPhase::new(current_year, current_index))
    }
}

impl PhaseCloseLogic for SpringRetreatPhase {
    fn create_next_phase(&self, current_year: i32, current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        Some(FallOrderPhase::new(current_year, current_index))
    }
}

impl PhaseCloseLogic for FallOrderPhase {
    fn create_next_phase(&self, current_year: i32, current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        Some(FallRetreatPhase::new(current_year, current_index))
    }
}

impl PhaseCloseLogic for FallRetreatPhase {
    fn create_next_phase(&self, current_year: i32, current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        Some(AdjustmentPhase::new(current_year, current_index))
    }
}

impl PhaseCloseLogic for AdjustmentPhase {
    fn create_next_phase(&self, current_year: i32, current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        Some(SpringOrderPhase::new(current_year, current_index))
    }
}

impl PhaseCloseLogic for DebriefPhase {
    fn create_next_phase(&self, _current_year: i32, _current_index: i32, _context: &mut PhaseContext) -> Option<Phase> {
        None
    }
}

/// フェイズのコンテキストと終了結果（必要に応じてフィールドを追加）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseContext {}

/// フェイズの終了結果（必要に応じてフィールドを追加）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCloseResult {}
