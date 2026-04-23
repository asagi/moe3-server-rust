// ============================================================================
// modules
// ============================================================================

mod method;
mod phase;

// ============================================================================
// exports
// ============================================================================

pub(crate) use phase::AdjustmentPhase;
pub(crate) use phase::DebriefPhase;
pub(crate) use phase::FallMainPhase;
pub(crate) use phase::FallRetreatPhase;
pub(crate) use phase::Phase;
pub(crate) use phase::PhaseKind;
pub(crate) use phase::ReadyPhase;
pub(crate) use phase::SUPPLY_CENTERS_FOR_SOLO;
pub(crate) use phase::SpringMainPhase;
pub(crate) use phase::SpringRetreatPhase;

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
