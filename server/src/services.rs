#![cfg_attr(not(test), allow(unused_imports, dead_code))]

// modules
mod auth_service;
mod discord_api_client;
mod game_service;

// traits
pub(crate) use auth_service::DiscordIdentityProvider;

// types
pub(crate) use auth_service::AuthError;
pub(crate) use auth_service::AuthService;
pub(crate) use auth_service::DiscordClientError;
pub(crate) use auth_service::LoginCommand;
pub(crate) use auth_service::LoginUser;
pub(crate) use game_service::CreateGameCommand;
pub(crate) use game_service::CreateGameError;
pub(crate) use game_service::GameService;
