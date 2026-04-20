#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod auth_handler;
mod game_handler;

// ============================================================================
// re-exports
// ============================================================================

// structs
pub(crate) use super::ApiErrorResponse;
pub(crate) use super::AuthLoginRequest;
pub(crate) use super::AuthLoginResponse;
pub(crate) use super::AuthLoginUserResponse;
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::CreateGameRequest;
pub(crate) use super::CreateGameResponse;
pub(crate) use super::GameService;
pub(crate) use super::LoginCommand;
pub(crate) use super::LoginUser;
pub(crate) use super::Regulation;

// enums
pub(crate) use super::AuthError;
pub(crate) use super::AuthRequestValidationError;
pub(crate) use super::CreateGameError;
pub(crate) use super::CreateGameRequestValidationError;
pub(crate) use super::DiscordClientError;
pub(crate) use super::Power;

// traits
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::GameRepository;
pub(crate) use super::UserRepository;
