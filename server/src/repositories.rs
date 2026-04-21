// ============================================================================
// modules
// ============================================================================

mod error;
mod game_repository;
mod sqlite_game_repository;
mod sqlite_user_repository;
mod user_repository;

// ============================================================================
// exports
// ============================================================================

pub(crate) use error::RepositoryError;
pub(crate) use game_repository::GameRepository;
pub(crate) use game_repository::NewGame;
pub(crate) use sqlite_game_repository::SqliteGameRepository;
pub(crate) use sqlite_user_repository::SqliteUserRepository;
pub(crate) use user_repository::DiscordProfile;
pub(crate) use user_repository::NewUser;
pub(crate) use user_repository::UserProfileUpdate;
pub(crate) use user_repository::UserRecord;
pub(crate) use user_repository::UserRepository;

pub(crate) type UserId = i64;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::DurationType;
pub(crate) use super::FaceType;
pub(crate) use super::Game;
pub(crate) use super::GameStatus;
pub(crate) use super::Order;
pub(crate) use super::OrderKind;
pub(crate) use super::OrderStatus;
pub(crate) use super::Phase;
pub(crate) use super::PhaseKind;
pub(crate) use super::Player;
pub(crate) use super::Power;
pub(crate) use super::ProgressMode;
pub(crate) use super::Province;
pub(crate) use super::Regulation;
pub(crate) use super::Territory;
pub(crate) use super::Unit;
