#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod api;
mod domain;
mod repositories;
mod services;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use domain::Game;
pub(crate) use domain::Order;
pub(crate) use domain::Phase;
pub(crate) use domain::Player;
pub(crate) use domain::Province;
pub(crate) use domain::Regulation;
pub(crate) use domain::Territory;
pub(crate) use domain::Unit;
pub(crate) use repositories::DiscordProfile;
pub(crate) use repositories::NewGame;
pub(crate) use repositories::NewUser;
pub(crate) use repositories::RepositoryError;
pub(crate) use repositories::UserProfileUpdate;
pub(crate) use repositories::UserRecord;
pub(crate) use repositories::UserRepository;
pub(crate) use services::AuthService;
pub(crate) use services::CreateGameCommand;
pub(crate) use services::GameService;
pub(crate) use services::LoginCommand;
pub(crate) use services::LoginUser;

// enums
pub(crate) use domain::OrderKind;
pub(crate) use domain::OrderStatus;
pub(crate) use domain::PhaseKind;
pub(crate) use domain::Power;
pub(crate) use services::AuthError;
pub(crate) use services::CreateGameError;
pub(crate) use services::DiscordClientError;

// traits
pub(crate) use repositories::GameRepository;
pub(crate) use services::DiscordIdentityProvider;
