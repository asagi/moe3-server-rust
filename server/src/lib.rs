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

pub use api::serve;

pub(crate) use domain::DurationType;
pub(crate) use domain::FaceType;
pub(crate) use domain::Game;
pub(crate) use domain::GameStatus;
pub(crate) use domain::Message;
pub(crate) use domain::MessageKind;
pub(crate) use domain::Order;
pub(crate) use domain::OrderKind;
pub(crate) use domain::OrderStatus;
pub(crate) use domain::Phase;
pub(crate) use domain::PhaseContext;
pub(crate) use domain::PhaseKind;
pub(crate) use domain::Player;
pub(crate) use domain::Power;
pub(crate) use domain::ProgressMode;
pub(crate) use domain::Province;
pub(crate) use domain::Regulation;
pub(crate) use domain::SystemNotice;
pub(crate) use domain::SystemNoticeCatalog;
pub(crate) use domain::Territory;
pub(crate) use domain::Unit;
pub(crate) use domain::User;
pub(crate) use repositories::DiscordProfile;
pub(crate) use repositories::GameRepository;
pub(crate) use repositories::NewGame;
pub(crate) use repositories::NewUser;
pub(crate) use repositories::RepositoryError;
pub(crate) use repositories::SqliteGameRepository;
pub(crate) use repositories::SqliteMessageRepository;
pub(crate) use repositories::SqliteUserRepository;
pub(crate) use repositories::UserProfileUpdate;
pub(crate) use repositories::UserRecord;
pub(crate) use repositories::UserRepository;
pub(crate) use services::AuthError;
pub(crate) use services::AuthService;
pub(crate) use services::CreateGameCommand;
pub(crate) use services::CreateGameError;
pub(crate) use services::DiscordApiClient;
pub(crate) use services::DiscordClientError;
pub(crate) use services::DiscordIdentityProvider;
pub(crate) use services::GameProgressionError;
pub(crate) use services::GameProgressionService;
pub(crate) use services::GameService;
pub(crate) use services::JoinGameCommand;
pub(crate) use services::JoinGameError;
pub(crate) use services::LoginCommand;
pub(crate) use services::LoginUser;
