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
pub(crate) use game_request::CreateGameRequest;
pub(crate) use game_request::CreateGameRequestBody;
pub(crate) use game_request::JoinGameRequest;
pub(crate) use game_request::JoinGameRequestBody;
pub(crate) use game_request::SetDrawProposalRequest;
pub(crate) use game_request::SetDrawProposalRequestBody;
