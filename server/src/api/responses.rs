#![cfg_attr(not(test), allow(unused_imports))]
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
pub(crate) use auth_response::AuthLoginUserResponse;
pub(crate) use game_response::CreateGameResponse;
