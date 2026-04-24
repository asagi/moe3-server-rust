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

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::LoginUser;
