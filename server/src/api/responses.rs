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

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::LoginUser;
