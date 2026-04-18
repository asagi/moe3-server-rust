// modules
mod adjudicators;
mod helpers;
mod models;

// types
pub(crate) use models::ConvoyOrder;
pub(crate) use models::MoveOrder;
pub(crate) use models::Order;
pub(crate) use models::OrderKind;
pub(crate) use models::Power;
pub(crate) use models::Province;
pub(crate) use models::Territory;
pub(crate) use models::Unit;

// type aliases
pub(crate) type OrderId = i64;
pub(crate) type PhaseId = i64;
pub(crate) type PlayerId = i64;
pub(crate) type TableId = i64;
pub(crate) type UserId = i64;

// adjudicators
pub(crate) use adjudicators::AdjustmentAdjudicator;
pub(crate) use adjudicators::MainAdjudicator;
pub(crate) use adjudicators::RetreatAdjudicator;

// helpers
pub(crate) use helpers::AdjustmentOrderHelper;
pub(crate) use helpers::MainOrderHelper;
pub(crate) use helpers::RetreatOrderHelper;
pub(crate) use helpers::UnitHelper;

#[cfg(test)]
mod tests;
