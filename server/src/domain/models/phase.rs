// modules
mod context;
mod methods;

// types
pub(crate) use super::Order;
pub(crate) use super::Power;
pub(crate) use super::Province;
pub(crate) use super::Territory;
pub(crate) use super::Unit;
pub(crate) use context::PhaseContext;

// enums
pub(crate) use super::OrderKind;

// type aliases
pub(crate) use super::PhaseId;
pub(crate) use super::TableId;

// adjudicators
pub(crate) use super::AdjustmentAdjudicator;
pub(crate) use super::MainAdjudicator;
pub(crate) use super::RetreatAdjudicator;

// helpers
pub(crate) use super::AdjustmentOrderHelper;
pub(crate) use super::MainOrderHelper;
pub(crate) use super::RetreatOrderHelper;
pub(crate) use super::UnitHelper;

// external crates
use chrono::DateTime;
use chrono::Utc;
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
