#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod auth_request;
mod game_request;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use auth_request::AuthLoginRequest;
pub(crate) use game_request::CreateGameRequest;

// enums
pub(crate) use auth_request::AuthRequestValidationError;
pub(crate) use game_request::CreateGameRequestValidationError;
