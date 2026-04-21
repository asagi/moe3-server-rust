// ============================================================================
// modules
// ============================================================================

mod auth_service;
mod discord_api_client;
mod game_progression_service;
mod game_service;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_service::AuthError;
pub(crate) use auth_service::AuthService;
pub(crate) use auth_service::DiscordClientError;
pub(crate) use auth_service::DiscordIdentityProvider;
pub(crate) use auth_service::LoginCommand;
pub(crate) use auth_service::LoginUser;
pub(crate) use game_progression_service::GameProgressionError;
pub(crate) use game_progression_service::GameProgressionService;
pub(crate) use game_service::CreateGameCommand;
pub(crate) use game_service::CreateGameError;
pub(crate) use game_service::GameService;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::DiscordProfile;
pub(crate) use super::Game;
pub(crate) use super::GameRepository;
pub(crate) use super::GameStatus;
pub(crate) use super::NewGame;
pub(crate) use super::NewUser;
pub(crate) use super::Phase;
pub(crate) use super::PhaseContext;
pub(crate) use super::Player;
pub(crate) use super::Power;
pub(crate) use super::Regulation;
pub(crate) use super::RepositoryError;
pub(crate) use super::UserProfileUpdate;
pub(crate) use super::UserRecord;
pub(crate) use super::UserRepository;
