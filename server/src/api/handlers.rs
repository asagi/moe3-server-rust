#![cfg_attr(not(test), allow(unused_imports))]
#![cfg_attr(test, allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod auth_handler;
mod game_handler;

// ============================================================================
// exports
// ============================================================================

pub(crate) use game_handler::CreateGameHandlerError;
pub(crate) use game_handler::handle_create_game;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::ApiErrorResponse;
pub(crate) use super::AuthError;
pub(crate) use super::AuthLoginRequest;
pub(crate) use super::AuthLoginResponse;
pub(crate) use super::AuthLoginUserResponse;
pub(crate) use super::AuthRequestValidationError;
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::CreateGameError;
pub(crate) use super::CreateGameRequest;
pub(crate) use super::CreateGameRequestValidationError;
pub(crate) use super::CreateGameResponse;
pub(crate) use super::DiscordClientError;
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::Game;
pub(crate) use super::GameRepository;
pub(crate) use super::GameService;
pub(crate) use super::LoginCommand;
pub(crate) use super::LoginUser;
pub(crate) use super::NewGame;
pub(crate) use super::NewUser;
pub(crate) use super::Power;
pub(crate) use super::Regulation;
pub(crate) use super::RepositoryError;
pub(crate) use super::UserId;
pub(crate) use super::UserProfileUpdate;
pub(crate) use super::UserRecord;
pub(crate) use super::UserRepository;
