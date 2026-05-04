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
pub(crate) use error::AuthRequestValidationError;
pub(crate) use error::CreateGameRequestValidationError;
pub(crate) use error::JoinGameRequestValidationError;
pub(crate) use error::SetDrawProposalRequestValidationError;
pub(crate) use error::SetTerritoryRequestValidationError;
pub(crate) use error::SetUnitRequestValidationError;
pub(crate) use game_request::CreateGameRequest;
pub(crate) use game_request::CreateGameRequestBody;
pub(crate) use game_request::JoinGameRequest;
pub(crate) use game_request::JoinGameRequestBody;
pub(crate) use game_request::SetDrawProposalRequest;
pub(crate) use game_request::SetDrawProposalRequestBody;
pub(crate) use game_request::SetTerritoryRequest;
pub(crate) use game_request::SetTerritoryRequestBody;
pub(crate) use game_request::SetUnitRequest;
pub(crate) use game_request::SetUnitRequestBody;
pub(crate) use game_request::DeleteUnitQueryParams;
pub(crate) use game_request::DeleteTerritoryQueryParams;
// テストコード (game_handler.rs) からのみ参照されるため unused_imports 警告を抑制する
#[allow(unused_imports)]
pub(crate) use game_request::UnitSpecBody;
