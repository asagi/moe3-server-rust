// ============================================================================
// modules
// ============================================================================

mod auth_response;
mod game_response;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_response::ApiErrorResponse;
pub(crate) use auth_response::AuthLoginResponse;
pub(crate) use auth_response::AuthLoginResponseUser;
pub(crate) use game_response::CreateGameResponse;

// ============================================================================
// re-exports
// ============================================================================

pub(crate) use super::LoginUser;
