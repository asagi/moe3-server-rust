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

    pub fn new_ready() -> Self {
        Self::new(1900, 0, PhaseType::Ready(ReadyPhase {}))
    }

    pub fn new_spring_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::SpringRetreat(SpringRetreatPhase {}))
    }

    pub fn new_spring_order(prev_year: i32, prev_index: i32) -> Self {
        Self::new(prev_year + 1, prev_index + 1, PhaseType::SpringOrder(SpringOrderPhase {}))
    }

    pub fn new_fall_order(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::FallOrder(FallOrderPhase {}))
    }

    pub fn new_fall_retreat(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::FallRetreat(FallRetreatPhase {}))
    }

    pub fn new_adjustment(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::Adjustment(AdjustmentPhase {}))
    }

    pub fn new_debrief(current_year: i32, prev_index: i32) -> Self {
        Self::new(current_year, prev_index + 1, PhaseType::Debrief(DebriefPhase {}))
    }

    pub fn close(&self, context: &mut PhaseContext) -> PhaseCloseResult {
        match &self.phase_type {
            PhaseType::Ready(r) => r.close(self, context),
            PhaseType::SpringOrder(s) => s.close(self, context),
            PhaseType::SpringRetreat(s) => s.close(self, context),
            PhaseType::FallOrder(f) => f.close(self, context),
            PhaseType::FallRetreat(f) => f.close(self, context),
            PhaseType::Adjustment(a) => a.close(self, context),
            PhaseType::Debrief(d) => d.close(self, context),
        }
    }
}

/// フェイズの終了ロジック
trait PhaseCloseLogic {
    fn close(&self, current_phase: &Phase, context: &mut PhaseContext) -> PhaseCloseResult
    where
        Self: Sized,
    {
        // 1. 和平判定
        if self.check_draw_condition(context) {
            return self.close_in_draw(context);
        }

        // 2. 解決（selfの状態を書き換える）
        self.resolve_orders(context);
        self.occupy(context);

        // 3. 制覇判定
        if self.check_resolved_condition(context) {
            return self.end_due_to_resolution(context);
        }

        // 4. 次フェイズ生成
        if let Some(next_phase) = self.create_next_phase(current_phase, context) {
            // 5. スキップ判定と再帰
            if !self.should_skip_next_phase(context, &next_phase) {
                context.next_phase = next_phase;
                return context.into_result();
            }

            // 次のフェイズをそのまま close() にかける（再帰）
            return next_phase.close(context);
        }
        context.into_result()
    }

    fn check_draw_condition(&self, _context: &PhaseContext) -> bool {
        false
    }

    fn close_in_draw(&self, _context: &mut PhaseContext) -> PhaseCloseResult {
        // # TODO: self.table のステータスを DRAW に変更
        // ...

        // if not self.due_time:
        //     return self._create_debrief_phase()._open()

        // now = get_current_time()
        // if self.table.due_mode == DueMode.FIXED or now >= self.due_time:
        //     due_time = self.due_time + timedelta(minutes=self.table.get_debrief_phase_duration())
        // else:
        //     due_time = now + timedelta(minutes=self.table.get_debrief_phase_duration())
        // return self._create_debrief_phase(due_time)._open()

        PhaseCloseResult {}
    }

    fn resolve_orders(&self, _context: &mut PhaseContext) {}

    fn occupy(&self, _context: &PhaseContext) {}

    fn check_resolved_condition(&self, _context: &PhaseContext) -> bool {
        false
    }

    fn end_due_to_resolution(&self, _context: &PhaseContext) -> PhaseCloseResult {
        // TODO
        PhaseCloseResult {}
    }

    fn create_next_phase(&self, current_phase: &Phase, context: &mut PhaseContext) -> Option<Phase>;

    fn should_skip_next_phase(&self, _context: &PhaseContext, _next_phase: &Phase) -> bool {
        false
    }

    // fn open(self) -> Self;
}

fn check_draw_condition_for_order_phase(_context: &PhaseContext) -> bool {
    // TODO

    // powers = active_powers if active_powers else Power.all()
    // draw_agreed_players = [p for p in self.table.players if p.power in powers and p.is_draw_agreed]
    // return len(draw_agreed_players) / len(powers) > 0.5

    false
}

fn resolve_orders_for_order_phase(_context: &mut PhaseContext) {
    // TODO

    // standoffs: set[Province] = set()
    // self.resolve_marching_orders(self.orders, standoffs)

    // for order in filter(lambda o: not o.is_assumed(), self.orders):
    //     self.units.append(order.create_unit())

    // for province in standoffs:
    //     self.standoffs.append(Standoff(province))
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

    fn resolve_orders(&self, context: &mut PhaseContext) {
        resolve_orders_for_order_phase(context);
    }
}

impl PhaseCloseLogic for SpringRetreatPhase {
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

    fn resolve_orders(&self, context: &mut PhaseContext) {
        resolve_orders_for_order_phase(context);
    }
}

impl PhaseCloseLogic for FallRetreatPhase {
    fn create_next_phase(&self, current_phase: &Phase, _context: &mut PhaseContext) -> Option<Phase> {
        Some(Phase::new_adjustment(current_phase.year, current_phase.index))
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseContext {
    pub next_phase: Phase,
}

impl PhaseContext {
    pub fn into_result(&self) -> PhaseCloseResult {
        PhaseCloseResult {}
    }
}

/// フェイズの終了結果（必要に応じてフィールドを追加）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCloseResult {}
