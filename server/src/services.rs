#![allow(unused_imports)]
#![allow(dead_code)]

// modules
mod auth_service;
mod discord_api_client;

// concrete types
pub(crate) use discord_api_client::DiscordApiClient;

// traits
pub(crate) use auth_service::DiscordIdentityProvider;

// types
pub(crate) use auth_service::AuthError;
pub(crate) use auth_service::AuthService;
pub(crate) use auth_service::DiscordClientError;
pub(crate) use auth_service::LoginCommand;
pub(crate) use auth_service::LoginResult;
pub(crate) use auth_service::LoginUser;
