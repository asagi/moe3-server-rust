#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod game_repository;
mod sqlite_game_repository;
mod sqlite_user_repository;
mod user_repository;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use game_repository::NewGame;
pub(crate) use user_repository::DiscordProfile;
pub(crate) use user_repository::NewUser;
pub(crate) use user_repository::RepositoryError;
pub(crate) use user_repository::UserProfileUpdate;
pub(crate) use user_repository::UserRecord;

// traits
pub(crate) use game_repository::GameRepository;
pub(crate) use user_repository::UserRepository;

// type aliases
pub(crate) type UserId = i64;

// ============================================================================
// re-exports
// ============================================================================

// structs
pub(crate) use super::Game;
pub(crate) use super::Order;
pub(crate) use super::Phase;
pub(crate) use super::Province;
pub(crate) use super::Territory;
pub(crate) use super::Unit;

// enums
pub(crate) use super::GameStatus;
pub(crate) use super::OrderKind;
pub(crate) use super::OrderStatus;
pub(crate) use super::PhaseKind;
pub(crate) use super::Power;

pub(crate) use super::DurationType;
pub(crate) use super::FaceType;
pub(crate) use super::Player;
pub(crate) use super::ProgressMode;
pub(crate) use super::Regulation;
