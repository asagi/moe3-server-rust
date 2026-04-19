// modules
mod auth_service;
#[allow(dead_code)]
mod discord_api_client;
mod game_service;

// concrete types
#[allow(unused_imports)]
pub(crate) use discord_api_client::DiscordApiClient;

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
#[allow(unused_imports)]
pub(crate) use game_service::CreateGameResult;
pub(crate) use game_service::GameService;
