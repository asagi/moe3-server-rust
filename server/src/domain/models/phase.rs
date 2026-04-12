mod close_logic;
mod context;
mod resolvers;

use super::super::PhaseId;
use super::super::TableId;
use super::order::Order;
use super::power::Power;
use super::territory::Territory;
use super::unit::Unit;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

pub use context::PhaseCloseResult;
pub use context::PhaseContext;
pub use resolvers::resolve_orders_for_adjustment_phase;
pub use resolvers::resolve_orders_for_main_phase;
pub use resolvers::resolve_orders_for_retreat_phase;

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

/// 各フェイズの詳細な構造体
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
}

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
