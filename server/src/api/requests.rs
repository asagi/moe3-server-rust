// ============================================================================
// modules
// ============================================================================

mod auth_request;
mod error;
mod game_request;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_request::AuthLoginRequest;
pub(crate) use auth_request::AuthResetTokenRequest;
pub(crate) use error::AuthRequestValidationError;
pub(crate) use error::AuthResetTokenRequestValidationError;
pub(crate) use error::CreateGameRequestValidationError;
pub(crate) use error::GetGamesRequestValidationError;
pub(crate) use error::JoinGameRequestValidationError;
pub(crate) use error::SetDrawProposalRequestValidationError;
pub(crate) use error::SetNextUpdateAtRequestValidationError;
pub(crate) use error::SetProgressConsensusRequestValidationError;
pub(crate) use error::SetProgressModeRequestValidationError;
pub(crate) use error::SetTerritoryRequestValidationError;
pub(crate) use error::SetUnitRequestValidationError;
pub(crate) use game_request::CreateGameRequest;
pub(crate) use game_request::CreateGameRequestBody;
pub(crate) use game_request::DeleteTerritoryQueryParams;
pub(crate) use game_request::DeleteUnitQueryParams;
pub(crate) use game_request::GetGamesQueryParams;
pub(crate) use game_request::GetGamesRequest;
pub(crate) use game_request::JoinGameRequest;
pub(crate) use game_request::JoinGameRequestBody;
pub(crate) use game_request::SetDrawProposalRequest;
pub(crate) use game_request::SetDrawProposalRequestBody;
pub(crate) use game_request::SetNextUpdateAtRequest;
pub(crate) use game_request::SetNextUpdateAtRequestBody;
pub(crate) use game_request::SetProgressConsensusRequest;
pub(crate) use game_request::SetProgressConsensusRequestBody;
pub(crate) use game_request::SetProgressModeRequest;
pub(crate) use game_request::SetProgressModeRequestBody;
pub(crate) use game_request::SetTerritoryRequest;
pub(crate) use game_request::SetTerritoryRequestBody;
pub(crate) use game_request::SetUnitRequest;
pub(crate) use game_request::SetUnitRequestBody;
#[allow(unused_imports)]
pub(crate) use game_request::UnitSpecBody;
