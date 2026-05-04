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
pub(crate) use error::ApiErrorResponse;
pub(crate) use game_response::CreateGameResponse;
pub(crate) use game_response::JoinGameResponse;
pub(crate) use game_response::SetDrawProposalResponse;
pub(crate) use game_response::SetUnitResponse;
pub(crate) use game_response::UnitResponseBody;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::LoginUser;
