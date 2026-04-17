// modules
#[cfg_attr(not(test), allow(dead_code))]
mod sqlite_user_repository;
mod user_repository;

// concrete types
#[allow(unused_imports)]
pub(crate) use sqlite_user_repository::SqliteUserRepository;

// traits
pub(crate) use user_repository::UserRepository;

// types
pub(crate) use user_repository::DiscordProfile;
pub(crate) use user_repository::NewUser;
pub(crate) use user_repository::RepositoryError;
pub(crate) use user_repository::UserProfileUpdate;
pub(crate) use user_repository::UserRecord;
