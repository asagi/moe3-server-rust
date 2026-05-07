// ============================================================================
// modules
// ============================================================================

mod auth_response;
mod error;
mod game_response;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_response::AuthLoginResponse;
pub(crate) use auth_response::AuthLoginResponseUser;
pub(crate) use auth_response::AuthResetTokenResponse;
pub(crate) use auth_response::GetMeResponse;
pub(crate) use error::ApiErrorResponse;
pub(crate) use game_response::CreateGameResponse;
pub(crate) use game_response::GameListItem;
pub(crate) use game_response::GameListItemRegulation;
pub(crate) use game_response::GetGamesResponse;
pub(crate) use game_response::JoinGameResponse;
pub(crate) use game_response::SetDrawProposalResponse;
pub(crate) use game_response::SetNextUpdateAtResponse;
pub(crate) use game_response::SetProgressConsensusResponse;
pub(crate) use game_response::SetProgressModeResponse;
pub(crate) use game_response::SetTerritoryResponse;
pub(crate) use game_response::SetUnitResponse;
pub(crate) use game_response::UnitResponseBody;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::LoginUser;
