// modules
mod adjudicator;
mod helper;
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
pub(crate) type UserId = i64;
pub(crate) type PlayerId = i64;
pub(crate) type TableId = i64;

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
