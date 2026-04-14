// FIXME: 開発中は pub としておく
pub mod phase;
pub mod player;
pub mod table;
pub use phase::Phase;

// modules
mod order;
mod path;
mod power;
mod province;
mod territory;
mod unit;
mod user;

// types
pub(crate) use order::BuildOrder;
pub(crate) use order::ConvoyOrder;
pub(crate) use order::DisbandOrder;
pub(crate) use order::HoldOrder;
pub(crate) use order::MoveOrder;
pub(crate) use order::Order;
pub(crate) use order::RetreatOrder;
pub(crate) use order::SupportOrder;
pub(crate) use path::Path;
pub(crate) use power::Power;
pub(crate) use province::Province;
pub(crate) use territory::Territory;
pub(crate) use unit::Unit;
pub(crate) use user::User;

// enums
pub(crate) use order::OrderKind;
pub(crate) use order::OrderStatus;
pub(crate) use unit::UnitKind;

// adjudicators
pub(crate) use super::AdjustmentAdjudicator;
pub(crate) use super::MainAdjudicator;
pub(crate) use super::RetreatAdjudicator;

// type aliases
pub(crate) use super::OrderId;
pub(crate) use super::PhaseId;
pub(crate) use super::PlayerId;
pub(crate) use super::TableId;
pub(crate) use super::UserId;

// helpers
pub(crate) use super::AdjustmentOrderHelper;
pub(crate) use super::MainOrderHelper;
pub(crate) use super::RetreatOrderHelper;
pub(crate) use super::UnitHelper;
