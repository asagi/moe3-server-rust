#![cfg_attr(not(test), allow(unused_imports))]
#![cfg_attr(test, allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod handlers;
mod middleware;
mod requests;
mod responses;
mod router;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use middleware::GlobalPreHandler;
pub(crate) use requests::AuthLoginRequest;
pub(crate) use requests::CreateGameRequest;
pub(crate) use responses::ApiErrorResponse;
pub(crate) use responses::AuthLoginResponse;
pub(crate) use responses::AuthLoginUserResponse;
pub(crate) use responses::CreateGameResponse;
pub(crate) use router::AppState;
pub(crate) use router::create_router;

// enums
pub(crate) use requests::AuthRequestValidationError;
pub(crate) use requests::CreateGameRequestValidationError;

// ============================================================================
// re-exports
// ============================================================================

// structs
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::Game;
pub(crate) use super::GameProgressionService;
pub(crate) use super::GameService;
pub(crate) use super::LoginCommand;
pub(crate) use super::LoginUser;
pub(crate) use super::NewGame;
pub(crate) use super::NewUser;
pub(crate) use super::Regulation;
pub(crate) use super::UserProfileUpdate;
pub(crate) use super::UserRecord;

// enums
pub(crate) use super::AuthError;
pub(crate) use super::CreateGameError;
pub(crate) use super::DiscordClientError;
pub(crate) use super::GameProgressionError;
pub(crate) use super::Power;
pub(crate) use super::RepositoryError;

// traits
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::GameRepository;
pub(crate) use super::UserRepository;

// type aliases
pub(crate) use super::UserId;
