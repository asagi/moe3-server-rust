#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod adjustment_adjudicator;
mod main_adjudicator;
mod retreat_adjudicator;

// ============================================================================
// exports
// ============================================================================

// traits
pub(crate) use adjustment_adjudicator::AdjustmentAdjudicator;
pub(crate) use main_adjudicator::MainAdjudicator;
pub(crate) use retreat_adjudicator::RetreatAdjudicator;

// ============================================================================
// re-exports
// ============================================================================

// structs
pub(crate) use super::models::Order;
pub(crate) use super::models::Path;
pub(crate) use super::models::Phase;
pub(crate) use super::models::Province;
pub(crate) use super::models::Unit;

// enums
pub(crate) use super::models::OrderKind;
pub(crate) use super::models::OrderStatus;
pub(crate) use super::models::Power;
pub(crate) use super::models::UnitKind;

// traits
pub(crate) use super::AdjustmentOrderHelper;
pub(crate) use super::MainOrderHelper;
pub(crate) use super::RetreatOrderHelper;
pub(crate) use super::UnitHelper;
