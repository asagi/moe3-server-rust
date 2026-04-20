#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// modules
// ============================================================================

mod methods;

// ============================================================================
// re-exports
// ============================================================================

// structs
pub(crate) use super::Order;
pub(crate) use super::PhaseContext;
pub(crate) use super::Power;
pub(crate) use super::Province;
pub(crate) use super::Territory;
pub(crate) use super::Unit;

// enums
pub(crate) use super::OrderKind;

// traits
pub(crate) use super::AdjustmentAdjudicator;
pub(crate) use super::AdjustmentOrderHelper;
pub(crate) use super::MainAdjudicator;
pub(crate) use super::MainOrderHelper;
pub(crate) use super::RetreatAdjudicator;
pub(crate) use super::RetreatOrderHelper;
pub(crate) use super::UnitHelper;

// ============================================================================
// imports
// ============================================================================

// external crates
use serde::Deserialize;
use serde::Serialize;

// ============================================================================
// definitions
// ============================================================================

// constants
/// 制覇勝利に必要な補給都市数
pub(crate) const SUPPLY_CENTERS_FOR_SOLO: usize = 18;

/// フェイズの定義
#[derive(Debug, Clone, PartialEq)]
pub struct Phase {
    pub(crate) game_number: Option<i32>,
    pub(crate) index: i32,
    pub(crate) year: i32,
    pub(crate) orders: Vec<Order>,
    pub(crate) units: Vec<Unit>,
    pub(crate) territories: Vec<Territory>,
    pub(crate) standoff_codes: Vec<String>,
    pub(crate) kind: PhaseKind,
}

/// フェイズの種類
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReadyPhase {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringMainPhase {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringRetreatPhase {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FallMainPhase {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FallRetreatPhase {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentPhase {}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DebriefPhase {}
