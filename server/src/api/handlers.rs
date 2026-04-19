// modules
mod auth_handler;
mod game_handler;

// functions
pub(crate) use auth_handler::handle_auth_login;
pub(crate) use game_handler::handle_create_game;

// types
pub(crate) use auth_handler::AuthHandlerError;
pub(crate) use game_handler::CreateGameHandlerError;
