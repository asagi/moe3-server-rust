use std::error::Error;
use std::fmt;

use uuid::Uuid;

use crate::repositories::DiscordProfile;
use crate::repositories::NewUser;
use crate::repositories::RepositoryError;
use crate::repositories::UserProfileUpdate;
use crate::repositories::UserRecord;
use crate::repositories::UserRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoginCommand {
    pub discord_access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoginUser {
    pub discord_user_id: String,
    pub display_name: String,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoginResult {
    pub access_token: String,
    pub user: LoginUser,
}

pub(crate) trait DiscordIdentityProvider {
    fn fetch_profile(&self, discord_access_token: &str) -> Result<DiscordProfile, DiscordClientError>;
}

pub(crate) struct AuthService<U, D>
where
    U: UserRepository,
    D: DiscordIdentityProvider,
{
    user_repository: U,
    discord_identity_provider: D,
}

impl<U, D> AuthService<U, D>
where
    U: UserRepository,
    D: DiscordIdentityProvider,
{
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new(user_repository: U, discord_identity_provider: D) -> Self {
        Self {
            user_repository,
            discord_identity_provider,
        }
    }

    pub(crate) fn login(&self, command: LoginCommand) -> Result<LoginResult, AuthError> {
        let discord_access_token = command.discord_access_token.trim().to_string();

        if discord_access_token.is_empty() {
            return Err(AuthError::InvalidRequest("discord_access_token is empty".to_string()));
        }

        let profile = self
            .discord_identity_provider
            .fetch_profile(&discord_access_token)
            .map_err(AuthError::DiscordClient)?;

        let existing = self
            .user_repository
            .find_by_discord_user_id(&profile.discord_user_id)
            .map_err(AuthError::Repository)?;

        if existing.is_some() {
            let update = UserProfileUpdate::from(&profile);
            let updated = self
                .user_repository
                .update_profile(&profile.discord_user_id, update)
                .map_err(AuthError::Repository)?;
            return Ok(Self::from_record(updated));
        }

        let new_user = NewUser {
            discord_user_id: profile.discord_user_id,
            username: profile.username,
            global_name: profile.global_name,
            avatar_hash: profile.avatar_hash,
            avatar_url: profile.avatar_url,
            access_token: Uuid::new_v4().to_string(),
        };

        let created = self.user_repository.insert(new_user).map_err(AuthError::Repository)?;
        Ok(Self::from_record(created))
    }

    fn from_record(record: UserRecord) -> LoginResult {
        let display_name = record.display_name().to_string();

        LoginResult {
            access_token: record.access_token,
            user: LoginUser {
                discord_user_id: record.discord_user_id,
                display_name,
                avatar_hash: record.avatar_hash,
                avatar_url: record.avatar_url,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum DiscordClientError {
    Unauthorized,
    Unavailable(String),
}

impl fmt::Display for DiscordClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "discord token is unauthorized"),
            Self::Unavailable(message) => write!(f, "discord api unavailable: {}", message),
        }
    }
}

impl Error for DiscordClientError {}

#[derive(Debug, Clone)]
pub(crate) enum AuthError {
    InvalidRequest(String),
    DiscordClient(DiscordClientError),
    Repository(RepositoryError),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::DiscordClient(error) => write!(f, "discord client error: {}", error),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

impl Error for AuthError {}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;

    #[derive(Debug, Clone)]
    struct FakeDiscordIdentityProvider {
        profile: DiscordProfile,
    }

    impl DiscordIdentityProvider for FakeDiscordIdentityProvider {
        fn fetch_profile(&self, discord_access_token: &str) -> Result<DiscordProfile, DiscordClientError> {
            if discord_access_token == "invalid" {
                return Err(DiscordClientError::Unauthorized);
            }
            Ok(self.profile.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        state: Rc<RefCell<State>>,
    }

    #[derive(Debug, Clone)]
    struct State {
        next_id: i64,
        rows: HashMap<String, UserRecord>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows.into_iter().map(|row| (row.discord_user_id.clone(), row)).collect();
            Self {
                state: Rc::new(RefCell::new(State { next_id: 100, rows: map })),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_discord_user_id(&self, discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.state.borrow().rows.get(discord_user_id).cloned())
        }

        fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError> {
            let mut state = self.state.borrow_mut();

            if state.rows.contains_key(&new_user.discord_user_id) {
                return Err(RepositoryError::Conflict);
            }

            let row = UserRecord {
                id: state.next_id,
                discord_user_id: new_user.discord_user_id,
                username: new_user.username,
                global_name: new_user.global_name,
                avatar_hash: new_user.avatar_hash,
                avatar_url: new_user.avatar_url,
                access_token: new_user.access_token,
            };
            state.next_id += 1;
            state.rows.insert(row.discord_user_id.clone(), row.clone());
            Ok(row)
        }

        fn update_profile(&self, discord_user_id: &str, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
            let mut state = self.state.borrow_mut();
            let row = state.rows.get_mut(discord_user_id).ok_or(RepositoryError::NotFound)?;
            row.username = profile.username;
            row.global_name = profile.global_name;
            row.avatar_hash = profile.avatar_hash;
            row.avatar_url = profile.avatar_url;
            Ok(row.clone())
        }
    }

    #[test]
    fn login_creates_new_user_and_generates_access_token() {
        let repository = InMemoryUserRepository::new(Vec::new());
        let discord = FakeDiscordIdentityProvider {
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "nemu".to_string(),
                global_name: Some("asagi".to_string()),
                avatar_hash: Some("abc".to_string()),
                avatar_url: Some("https://cdn.discordapp.com/avatar.png".to_string()),
            },
        };
        let service = AuthService::new(repository.clone(), discord);

        let result = service
            .login(LoginCommand {
                discord_access_token: "valid_token".to_string(),
            })
            .expect("login should succeed");

        assert_eq!(result.user.discord_user_id, "1001");
        assert_eq!(result.user.display_name, "asagi");
        assert!(!result.access_token.is_empty());

        let saved = repository
            .find_by_discord_user_id("1001")
            .expect("read should succeed")
            .expect("saved user should exist");
        assert_eq!(saved.access_token, result.access_token);
        assert_eq!(saved.username, "nemu");
        assert_eq!(saved.global_name.as_deref(), Some("asagi"));
    }

    #[test]
    fn login_updates_existing_user_and_reuses_access_token() {
        let repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            discord_user_id: "1001".to_string(),
            username: "old_user".to_string(),
            global_name: Some("old_name".to_string()),
            avatar_hash: Some("old_hash".to_string()),
            avatar_url: Some("https://cdn.discordapp.com/old.png".to_string()),
            access_token: "persisted-token".to_string(),
        }]);

        let discord = FakeDiscordIdentityProvider {
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "new_user".to_string(),
                global_name: Some("new_name".to_string()),
                avatar_hash: Some("new_hash".to_string()),
                avatar_url: Some("https://cdn.discordapp.com/new.png".to_string()),
            },
        };

        let service = AuthService::new(repository.clone(), discord);

        let result = service
            .login(LoginCommand {
                discord_access_token: "valid_token".to_string(),
            })
            .expect("login should succeed");

        assert_eq!(result.access_token, "persisted-token");
        assert_eq!(result.user.display_name, "new_name");

        let saved = repository
            .find_by_discord_user_id("1001")
            .expect("read should succeed")
            .expect("saved user should exist");

        assert_eq!(saved.username, "new_user");
        assert_eq!(saved.global_name.as_deref(), Some("new_name"));
        assert_eq!(saved.avatar_hash.as_deref(), Some("new_hash"));
    }

    #[test]
    fn login_returns_error_when_token_is_invalid() {
        let repository = InMemoryUserRepository::new(Vec::new());
        let discord = FakeDiscordIdentityProvider {
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "asagi".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
            },
        };
        let service = AuthService::new(repository, discord);

        let result = service.login(LoginCommand {
            discord_access_token: "invalid".to_string(),
        });

        assert!(matches!(
            result,
            Err(AuthError::DiscordClient(DiscordClientError::Unauthorized))
        ));
    }

    #[test]
    fn login_trims_token_before_fetching_profile() {
        #[derive(Debug, Clone)]
        struct CapturingDiscordIdentityProvider {
            received_token: Rc<RefCell<Option<String>>>,
            profile: DiscordProfile,
        }

        impl DiscordIdentityProvider for CapturingDiscordIdentityProvider {
            fn fetch_profile(&self, discord_access_token: &str) -> Result<DiscordProfile, DiscordClientError> {
                self.received_token.replace(Some(discord_access_token.to_string()));
                Ok(self.profile.clone())
            }
        }

        let repository = InMemoryUserRepository::new(Vec::new());
        let received_token = Rc::new(RefCell::new(None));
        let discord = CapturingDiscordIdentityProvider {
            received_token: received_token.clone(),
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "asagi".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
            },
        };
        let service = AuthService::new(repository, discord);

        let result = service.login(LoginCommand {
            discord_access_token: "  valid_token  ".to_string(),
        });

        assert!(result.is_ok());
        assert_eq!(received_token.borrow().as_deref(), Some("valid_token"));
    }
}
