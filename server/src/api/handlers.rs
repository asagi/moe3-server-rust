// ============================================================================
// modules
// ============================================================================

mod auth_handler;
mod error;
mod game_handler;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_handler::post_auth_login;
pub(crate) use error::AuthHandlerError;
pub(crate) use error::CreateGameHandlerError;
pub(crate) use game_handler::post_games;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::ApiErrorResponse;
pub(crate) use super::AppState;
pub(crate) use super::AuthError;
pub(crate) use super::AuthLoginRequest;
pub(crate) use super::AuthLoginResponse;
pub(crate) use super::AuthLoginResponseUser;
pub(crate) use super::AuthRequestValidationError;
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::CreateGameError;
pub(crate) use super::CreateGameRequest;
pub(crate) use super::CreateGameRequestBody;
pub(crate) use super::CreateGameRequestValidationError;
pub(crate) use super::CreateGameResponse;
pub(crate) use super::DiscordClientError;
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::GameRepository;
pub(crate) use super::GameService;
pub(crate) use super::LoginCommand;
pub(crate) use super::Power;
pub(crate) use super::ProgressMode;
pub(crate) use super::Regulation;
pub(crate) use super::RepositoryError;
pub(crate) use super::UserRepository;
