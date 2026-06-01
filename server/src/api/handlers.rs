// ============================================================================
// modules
// ============================================================================

mod auth_handler;
mod error;
mod game_handler;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_handler::get_users_me;
pub(crate) use auth_handler::post_auth_login;
pub(crate) use auth_handler::post_auth_reset_token;
pub(crate) use error::AuthHandlerError;
pub(crate) use error::CreateGameHandlerError;
pub(crate) use error::GetGameHandlerError;
pub(crate) use error::GetGameLogsHandlerError;
pub(crate) use error::GetGamesHandlerError;
pub(crate) use error::GetMeHandlerError;
pub(crate) use error::JoinGameHandlerError;
pub(crate) use error::ResetTokenHandlerError;
pub(crate) use error::SetDrawProposalHandlerError;
pub(crate) use error::SetNextUpdateAtHandlerError;
pub(crate) use error::SetProgressConsensusHandlerError;
pub(crate) use error::SetProgressModeHandlerError;
pub(crate) use error::SetTerritoryHandlerError;
pub(crate) use error::SetUnitHandlerError;
pub(crate) use game_handler::delete_admin_games_territories;
pub(crate) use game_handler::delete_admin_games_units;
pub(crate) use game_handler::get_game;
pub(crate) use game_handler::get_games;
pub(crate) use game_handler::get_games_logs;
pub(crate) use game_handler::post_games;
pub(crate) use game_handler::post_games_players;
pub(crate) use game_handler::put_admin_games_draw_proposal;
pub(crate) use game_handler::put_admin_games_next_update_at;
pub(crate) use game_handler::put_admin_games_progress_mode;
pub(crate) use game_handler::put_admin_games_territories;
pub(crate) use game_handler::put_admin_games_units;
pub(crate) use game_handler::put_games_progress_consensus;

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
pub(crate) use super::AuthResetTokenRequest;
pub(crate) use super::AuthResetTokenRequestValidationError;
pub(crate) use super::AuthResetTokenResponse;
pub(crate) use super::AuthService;
pub(crate) use super::CreateGameCommand;
pub(crate) use super::CreateGameError;
pub(crate) use super::CreateGameRequest;
pub(crate) use super::CreateGameRequestBody;
pub(crate) use super::CreateGameRequestValidationError;
pub(crate) use super::CreateGameResponse;
pub(crate) use super::DeleteTerritoryQueryParams;
pub(crate) use super::DeleteUnitQueryParams;
pub(crate) use super::DiscordClientError;
pub(crate) use super::DiscordIdentityProvider;
pub(crate) use super::DurationType;
pub(crate) use super::FaceType;
pub(crate) use super::GameListItem;
pub(crate) use super::GameListItemRegulation;
pub(crate) use super::GameLogMessage;
pub(crate) use super::GameRepository;
pub(crate) use super::GameService;
pub(crate) use super::GameStatus;
pub(crate) use super::GameStatusFilter;
pub(crate) use super::GetGameError;
pub(crate) use super::GetGameLogsQueryParams;
pub(crate) use super::GetGameLogsResponse;
pub(crate) use super::GetGameResponse;
pub(crate) use super::GetGameResponseGame;
pub(crate) use super::GetGamesQueryParams;
pub(crate) use super::GetGamesRequest;
pub(crate) use super::GetGamesRequestValidationError;
pub(crate) use super::GetGamesResponse;
pub(crate) use super::GetMeResponse;
pub(crate) use super::JoinGameCommand;
pub(crate) use super::JoinGameError;
pub(crate) use super::JoinGameRequest;
pub(crate) use super::JoinGameRequestBody;
pub(crate) use super::JoinGameRequestValidationError;
pub(crate) use super::JoinGameResponse;
pub(crate) use super::ListGamesError;
pub(crate) use super::ListGamesResult;
pub(crate) use super::LoginCommand;
pub(crate) use super::Power;
pub(crate) use super::ProgressMode;
pub(crate) use super::Regulation;
pub(crate) use super::RepositoryError;
pub(crate) use super::SetDrawProposalCommand;
pub(crate) use super::SetDrawProposalError;
pub(crate) use super::SetDrawProposalRequest;
pub(crate) use super::SetDrawProposalRequestBody;
pub(crate) use super::SetDrawProposalRequestValidationError;
pub(crate) use super::SetDrawProposalResponse;
pub(crate) use super::SetNextUpdateAtCommand;
pub(crate) use super::SetNextUpdateAtError;
pub(crate) use super::SetNextUpdateAtRequest;
pub(crate) use super::SetNextUpdateAtRequestBody;
pub(crate) use super::SetNextUpdateAtRequestValidationError;
pub(crate) use super::SetNextUpdateAtResponse;
pub(crate) use super::SetProgressConsensusCommand;
pub(crate) use super::SetProgressConsensusError;
pub(crate) use super::SetProgressConsensusRequest;
pub(crate) use super::SetProgressConsensusRequestBody;
pub(crate) use super::SetProgressConsensusRequestValidationError;
pub(crate) use super::SetProgressConsensusResponse;
pub(crate) use super::SetProgressModeCommand;
pub(crate) use super::SetProgressModeError;
pub(crate) use super::SetProgressModeRequest;
pub(crate) use super::SetProgressModeRequestBody;
pub(crate) use super::SetProgressModeRequestValidationError;
pub(crate) use super::SetProgressModeResponse;
pub(crate) use super::SetTerritoryCommand;
pub(crate) use super::SetTerritoryError;
pub(crate) use super::SetTerritoryRequest;
pub(crate) use super::SetTerritoryRequestBody;
pub(crate) use super::SetTerritoryRequestValidationError;
pub(crate) use super::SetTerritoryResponse;
pub(crate) use super::SetUnitCommand;
pub(crate) use super::SetUnitError;
pub(crate) use super::SetUnitRequest;
pub(crate) use super::SetUnitRequestBody;
pub(crate) use super::SetUnitRequestValidationError;
pub(crate) use super::SetUnitResponse;
pub(crate) use super::SqliteMessageRepository;
pub(crate) use super::UnitResponseBody;
pub(crate) use super::UnitSpec;
pub(crate) use super::UnitSpecBody;
pub(crate) use super::User;
pub(crate) use super::UserRepository;
