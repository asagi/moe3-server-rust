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

pub use router::serve;

pub(crate) use middleware::GlobalPreHandler;
pub(crate) use requests::AuthLoginRequest;
pub(crate) use requests::AuthRequestValidationError;
pub(crate) use requests::CreateGameRequest;
pub(crate) use requests::CreateGameRequestBody;
pub(crate) use requests::CreateGameRequestValidationError;
pub(crate) use requests::JoinGameRequest;
pub(crate) use requests::JoinGameRequestBody;
pub(crate) use requests::JoinGameRequestValidationError;
pub(crate) use requests::SetDrawProposalRequest;
pub(crate) use requests::SetDrawProposalRequestBody;
pub(crate) use requests::SetDrawProposalRequestValidationError;
pub(crate) use requests::SetUnitRequest;
pub(crate) use requests::SetUnitRequestBody;
pub(crate) use requests::SetUnitRequestValidationError;
pub(crate) use requests::UnitSpecBody;
pub(crate) use responses::ApiErrorResponse;
pub(crate) use responses::AuthLoginResponse;
pub(crate) use responses::AuthLoginResponseUser;
pub(crate) use responses::CreateGameResponse;
pub(crate) use responses::JoinGameResponse;
pub(crate) use responses::SetDrawProposalResponse;
pub(crate) use responses::SetUnitResponse;
pub(crate) use responses::UnitResponseBody;
pub(crate) use router::AppState;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::AuthError;
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::CreateGameError;
pub(crate) use super::DiscordApiClient;
pub(crate) use super::DiscordClientError;
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::GameProgressionError;
pub(crate) use super::GameProgressionService;
pub(crate) use super::GameRepository;
pub(crate) use super::GameService;
pub(crate) use super::GameStatus;
pub(crate) use super::JoinGameCommand;
pub(crate) use super::JoinGameError;
pub(crate) use super::LoginCommand;
pub(crate) use super::LoginUser;
pub(crate) use super::Power;
pub(crate) use super::ProgressMode;
pub(crate) use super::Regulation;
pub(crate) use super::RepositoryError;
pub(crate) use super::SetDrawProposalCommand;
pub(crate) use super::SetDrawProposalError;
pub(crate) use super::SetUnitCommand;
pub(crate) use super::SetUnitError;
pub(crate) use super::SqliteGameRepository;
pub(crate) use super::SqliteMessageRepository;
pub(crate) use super::SqliteUserRepository;
pub(crate) use super::UnitSpec;
pub(crate) use super::User;
pub(crate) use super::UserRepository;
