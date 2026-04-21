// ============================================================================
// modules
// ============================================================================

mod auth_handler;
mod game_handler;

// ============================================================================
// exports
// ============================================================================

pub(crate) use game_handler::post_games;

#[allow(unused_imports)]
pub(crate) use game_handler::CreateGameHandlerError;

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
pub(crate) use super::Regulation;
pub(crate) use super::UserRepository;

#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use super::Game;

#[allow(unused_imports)]
pub(crate) use super::LoginUser;

#[allow(unused_imports)]
pub(crate) use super::NewGame;

#[allow(unused_imports)]
pub(crate) use super::NewUser;

#[allow(unused_imports)]
pub(crate) use super::RepositoryError;

#[allow(unused_imports)]
pub(crate) use super::UserId;

#[allow(unused_imports)]
pub(crate) use super::UserProfileUpdate;

#[allow(unused_imports)]
pub(crate) use super::UserRecord;
