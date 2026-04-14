pub mod models;

pub type OrderId = i64;
pub type PhaseId = i64;
pub type UserId = i64;
pub type PlayerId = i64;
pub type TableId = i64;

// modules
pub(crate) mod adjudicator;
pub(crate) mod helper;

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
pub(crate) use adjudicator::adjustment_adjudicator::AdjustmentAdjudicator;
pub(crate) use adjudicator::main_adjudicator::MainAdjudicator;
pub(crate) use adjudicator::retreat_adjudicator::RetreatAdjudicator;

// helpers
pub(crate) use helper::adjustment_order_helper::AdjustmentOrderHelper;
pub(crate) use helper::main_order_helper::MainOrderHelper;
pub(crate) use helper::retreat_order_helper::RetreatOrderHelper;
pub(crate) use helper::unit_helper::UnitHelper;

#[cfg(test)]
mod tests;
