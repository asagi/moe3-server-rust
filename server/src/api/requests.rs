// modules
mod auth_request;
mod game_request;

// types
pub(crate) use auth_request::AuthLoginRequest;
pub(crate) use auth_request::AuthRequestValidationError;
pub(crate) use game_request::CreateGameRequest;
pub(crate) use game_request::CreateGameRequestValidationError;
