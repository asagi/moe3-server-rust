#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod adjudicators;
mod helpers;
mod models;

#[cfg(test)]
mod tests;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use models::ConvoyOrder;
pub(crate) use models::Game;
pub(crate) use models::MoveOrder;
pub(crate) use models::Order;
pub(crate) use models::Phase;
pub(crate) use models::PhaseContext;
pub(crate) use models::Player;
pub(crate) use models::Power;
pub(crate) use models::Province;
pub(crate) use models::Regulation;
pub(crate) use models::Territory;
pub(crate) use models::Unit;

// enums
pub(crate) use models::DurationType;
pub(crate) use models::FaceType;
pub(crate) use models::GameStatus;
pub(crate) use models::OrderKind;
pub(crate) use models::OrderStatus;
pub(crate) use models::PhaseKind;
pub(crate) use models::ProgressMode;

// traits
pub(crate) use adjudicators::AdjustmentAdjudicator;
pub(crate) use adjudicators::MainAdjudicator;
pub(crate) use adjudicators::RetreatAdjudicator;
pub(crate) use helpers::AdjustmentOrderHelper;
pub(crate) use helpers::MainOrderHelper;
pub(crate) use helpers::RetreatOrderHelper;
pub(crate) use helpers::UnitHelper;
