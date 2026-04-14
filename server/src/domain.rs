// FIXME: 開発中は pub としておく
pub mod models;
pub type OrderId = i64;
pub type PhaseId = i64;
pub type UserId = i64;
pub type PlayerId = i64;
pub type TableId = i64;

// modules
mod adjudicator;
mod helper;

// types
pub(crate) use models::ConvoyOrder;
pub(crate) use models::MoveOrder;
pub(crate) use models::Order;
pub(crate) use models::OrderKind;
pub(crate) use models::Power;
pub(crate) use models::Province;
pub(crate) use models::Territory;
pub(crate) use models::Unit;

// adjudicators
pub(crate) use adjudicator::AdjustmentAdjudicator;
pub(crate) use adjudicator::MainAdjudicator;
pub(crate) use adjudicator::RetreatAdjudicator;

// helpers
pub(crate) use helper::AdjustmentOrderHelper;
pub(crate) use helper::MainOrderHelper;
pub(crate) use helper::RetreatOrderHelper;
pub(crate) use helper::UnitHelper;

#[cfg(test)]
mod tests;
