#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod handlers;
mod requests;
mod responses;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use requests::AuthLoginRequest;
pub(crate) use requests::CreateGameRequest;
pub(crate) use responses::ApiErrorResponse;
pub(crate) use responses::AuthLoginResponse;
pub(crate) use responses::AuthLoginUserResponse;
pub(crate) use responses::CreateGameResponse;

// enums
pub(crate) use requests::AuthRequestValidationError;
pub(crate) use requests::CreateGameRequestValidationError;

// ============================================================================
// re-exports
// ============================================================================

// structs
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::GameService;
pub(crate) use super::LoginCommand;
pub(crate) use super::LoginUser;
pub(crate) use super::Regulation;

// enums
pub(crate) use super::AuthError;
pub(crate) use super::CreateGameError;
pub(crate) use super::DiscordClientError;
pub(crate) use super::Power;

// traits
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::GameRepository;
pub(crate) use super::UserRepository;
