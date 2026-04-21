// ============================================================================
// modules
// ============================================================================

mod methods;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::AdjustmentAdjudicator;
pub(crate) use super::AdjustmentOrderHelper;
pub(crate) use super::MainAdjudicator;
pub(crate) use super::MainOrderHelper;
pub(crate) use super::Order;
pub(crate) use super::OrderKind;
pub(crate) use super::PhaseContext;
pub(crate) use super::Power;
pub(crate) use super::Province;
pub(crate) use super::RetreatAdjudicator;
pub(crate) use super::RetreatOrderHelper;
pub(crate) use super::Territory;
pub(crate) use super::Unit;
pub(crate) use super::UnitHelper;

// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;
use serde::Serialize;

// ============================================================================
// definitions
// ============================================================================

/// 制覇勝利に必要な補給都市数
pub(crate) const SUPPLY_CENTERS_FOR_SOLO: usize = 18;

///
/// フェイズの種類の列挙体
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PhaseKind {
    Ready(ReadyPhase),                 // 準備
    SpringMain(SpringMainPhase),       // 春命令
    SpringRetreat(SpringRetreatPhase), // 春撤退
    FallMain(FallMainPhase),           // 秋命令
    FallRetreat(FallRetreatPhase),     // 秋撤退
    Adjustment(AdjustmentPhase),       // 調整
    Debrief(DebriefPhase),             // 感想戦
}

///
/// フェイズの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Phase {
    pub(crate) game_number: Option<i32>,
    pub(crate) index: i32,
    pub(crate) year: i32,
    pub(crate) orders: Vec<Order>,
    pub(crate) units: Vec<Unit>,
    pub(crate) territories: Vec<Territory>,
    pub(crate) standoff_codes: Vec<String>,
    pub(crate) kind: PhaseKind,
}

/// 準備フェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct ReadyPhase {}

/// 春メインフェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct SpringMainPhase {}

/// 春撤退フェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct SpringRetreatPhase {}

/// 秋メインフェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct FallMainPhase {}

/// 秋撤退フェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct FallRetreatPhase {}

/// 調整フェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct AdjustmentPhase {}

/// 感想戦フェイズの構造体
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) struct DebriefPhase {}
