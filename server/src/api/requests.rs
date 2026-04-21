#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod auth_request;
mod game_request;

// ============================================================================
// exports
// ============================================================================

pub(crate) use auth_request::AuthLoginRequest;
pub(crate) use auth_request::AuthRequestValidationError;
pub(crate) use game_request::CreateGameRequest;
pub(crate) use game_request::CreateGameRequestValidationError;
