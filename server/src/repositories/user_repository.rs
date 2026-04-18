use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscordProfile {
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserRecord {
    pub id: i64,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewUser {
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserProfileUpdate {
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

impl From<&DiscordProfile> for UserProfileUpdate {
    fn from(profile: &DiscordProfile) -> Self {
        Self {
            username: profile.username.clone(),
            global_name: profile.global_name.clone(),
            avatar_hash: profile.avatar_hash.clone(),
            avatar_url: profile.avatar_url.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum RepositoryError {
    NotFound,
    Conflict,
    Unavailable(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "record not found"),
            Self::Conflict => write!(f, "record conflict"),
            Self::Unavailable(message) => write!(f, "repository unavailable: {}", message),
        }
    }
}

impl Error for RepositoryError {}

pub(crate) trait UserRepository {
    fn find_by_discord_user_id(&self, discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError>;

    fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError>;

    fn update_profile(&self, discord_user_id: &str, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError>;
}
