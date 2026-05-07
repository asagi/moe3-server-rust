// ============================================================================
// imports
// ============================================================================

use serde::Serialize;
use uuid::Uuid;

use super::AuthError;
use super::DiscordClientError;
use super::DiscordProfile;
use super::NewUser;
use super::RepositoryError;
use super::UserProfileUpdate;
use super::UserRecord;
use super::UserRepository;

// ============================================================================
// definitions
// ============================================================================

///
/// ログインコマンドの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoginCommand {
    pub discord_access_token: String,
}

///
/// トークンリセット結果の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResetTokenResult {
    pub access_token: String,
}

///
/// ログインユーザーの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct LoginUser {
    pub uuid: Uuid,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

///
/// ログイン処理結果の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoginResult {
    pub access_token: String,
    pub user: LoginUser,
}

///
/// ユーザー情報取得結果の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GetMeResult {
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_url: Option<String>,
}

///
/// Discord 認証プロバイダのトレイト
///
pub(crate) trait DiscordIdentityProvider {
    fn fetch_profile(&self, discord_access_token: &str) -> Result<DiscordProfile, DiscordClientError>;
}

///
/// 認証サービスの構造体
///
pub(crate) struct AuthService<U, D>
where
    U: UserRepository,
    D: DiscordIdentityProvider,
{
    user_repository: U,
    discord_identity_provider: D,
}

/// 認証サービスの構造体の実装
impl<U, D> AuthService<U, D>
where
    U: UserRepository,
    D: DiscordIdentityProvider,
{
    ///
    /// new 関数
    ///
    pub(crate) fn new(user_repository: U, discord_identity_provider: D) -> Self {
        Self {
            user_repository,
            discord_identity_provider,
        }
    }

    ///
    /// ログイン処理を実行する
    ///
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

        if let Some(existing) = existing {
            let update = UserProfileUpdate::from(&profile);
            let updated = self
                .user_repository
                .update_profile(existing.id, update)
                .map_err(AuthError::Repository)?;
            return Ok(Self::from_record(updated));
        }

        let new_user = NewUser {
            uuid: Uuid::now_v7(),
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

    /// ユーザーレコードからログイン結果に変換する
    fn from_record(record: UserRecord) -> LoginResult {
        LoginResult {
            access_token: record.access_token,
            user: LoginUser {
                uuid: record.uuid,
                discord_user_id: record.discord_user_id,
                username: record.username,
                global_name: record.global_name,
                avatar_hash: record.avatar_hash,
                avatar_url: record.avatar_url,
            },
        }
    }

    ///
    /// 自身のユーザー情報を取得する
    ///
    pub(crate) fn get_me(&self, access_token: &str) -> Result<GetMeResult, AuthError> {
        let user = self
            .user_repository
            .find_by_access_token(access_token)
            .map_err(AuthError::Repository)?
            .ok_or(AuthError::Unauthorized)?;

        Ok(GetMeResult {
            discord_user_id: user.discord_user_id,
            username: user.username,
            global_name: user.global_name,
            avatar_url: user.avatar_url,
        })
    }

    ///
    /// トークンリセット処理を実行する
    ///
    pub(crate) fn reset_token(&self, current_token: &str) -> Result<ResetTokenResult, AuthError> {
        let user = self
            .user_repository
            .find_by_access_token(current_token)
            .map_err(AuthError::Repository)?
            .ok_or(AuthError::Unauthorized)?;

        let new_token = Uuid::new_v4().to_string();
        let updated = self
            .user_repository
            .update_access_token(user.id, current_token, &new_token)
            .map_err(|e| match e {
                RepositoryError::NotFound => AuthError::Unauthorized,
                e => AuthError::Repository(e),
            })?;

        Ok(ResetTokenResult {
            access_token: updated.access_token,
        })
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use chrono::DateTime;
    use chrono::Utc;

    use super::*;
    use crate::repositories::RepositoryError;

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
        fn find_by_uuid(&self, user_uuid: Uuid) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.state.borrow().rows.values().find(|row| row.uuid == user_uuid).cloned())
        }

        fn find_by_discord_user_id(&self, discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.state.borrow().rows.get(discord_user_id).cloned())
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self
                .state
                .borrow()
                .rows
                .values()
                .find(|row| row.access_token == access_token)
                .cloned())
        }

        fn update_last_access_at_by_access_token(
            &self,
            access_token: &str,
            last_access_at: DateTime<Utc>,
        ) -> Result<bool, RepositoryError> {
            let mut state = self.state.borrow_mut();
            let Some(row) = state.rows.values_mut().find(|row| row.access_token == access_token) else {
                return Ok(false);
            };

            row.last_access_at = last_access_at;
            Ok(true)
        }

        fn update_access_token(&self, id: i64, current_token: &str, new_token: &str) -> Result<UserRecord, RepositoryError> {
            let mut state = self.state.borrow_mut();
            let row = state
                .rows
                .values_mut()
                .find(|r| r.id == id && r.access_token == current_token)
                .ok_or(RepositoryError::NotFound)?;
            row.access_token = new_token.to_string();
            Ok(row.clone())
        }

        fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError> {
            let mut state = self.state.borrow_mut();

            if state.rows.contains_key(&new_user.discord_user_id) {
                return Err(RepositoryError::Conflict);
            }

            let row = UserRecord {
                id: state.next_id,
                uuid: new_user.uuid,
                discord_user_id: new_user.discord_user_id,
                username: new_user.username,
                global_name: new_user.global_name,
                avatar_hash: new_user.avatar_hash,
                avatar_url: new_user.avatar_url,
                access_token: new_user.access_token,
                last_access_at: Utc::now(),
            };
            state.next_id += 1;
            state.rows.insert(row.discord_user_id.clone(), row.clone());
            Ok(row)
        }

        fn update_profile(&self, id: i64, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
            let mut state = self.state.borrow_mut();
            let row = state
                .rows
                .values_mut()
                .find(|r| r.id == id)
                .ok_or(RepositoryError::NotFound)?;
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
        assert_eq!(result.user.username, "nemu");
        assert_eq!(result.user.global_name.as_deref(), Some("asagi"));
        assert_eq!(
            result.user.global_name.as_deref().unwrap_or(result.user.username.as_str()),
            "asagi"
        );
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
            uuid: Uuid::now_v7(),
            discord_user_id: "1001".to_string(),
            username: "old_user".to_string(),
            global_name: Some("old_name".to_string()),
            avatar_hash: Some("old_hash".to_string()),
            avatar_url: Some("https://cdn.discordapp.com/old.png".to_string()),
            access_token: "persisted-token".to_string(),
            last_access_at: Utc::now(),
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
        assert_eq!(result.user.username, "new_user");
        assert_eq!(result.user.global_name.as_deref(), Some("new_name"));
        assert_eq!(
            result.user.global_name.as_deref().unwrap_or(result.user.username.as_str()),
            "new_name"
        );

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

    #[test]
    fn reset_token_issues_new_token_and_invalidates_old_one() {
        let uuid = uuid::Uuid::now_v7();
        let repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid,
            discord_user_id: "1001".to_string(),
            username: "nemu".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "old-token".to_string(),
            last_access_at: Utc::now(),
        }]);
        let discord = FakeDiscordIdentityProvider {
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "nemu".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
            },
        };
        let service = AuthService::new(repository.clone(), discord);

        let result = service.reset_token("old-token").expect("reset should succeed");

        assert_ne!(result.access_token, "old-token");
        assert!(!result.access_token.is_empty());

        // 旧トークンでは取得できない
        let old_lookup = repository.find_by_access_token("old-token").expect("find should succeed");
        assert!(old_lookup.is_none());

        // 新トークンで取得できる
        let new_lookup = repository
            .find_by_access_token(&result.access_token)
            .expect("find should succeed")
            .expect("user should exist");
        assert_eq!(new_lookup.uuid, uuid);
    }

    #[test]
    fn reset_token_returns_unauthorized_for_unknown_token() {
        let repository = InMemoryUserRepository::new(Vec::new());
        let discord = FakeDiscordIdentityProvider {
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "nemu".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
            },
        };
        let service = AuthService::new(repository, discord);

        let result = service.reset_token("unknown-token");

        assert!(matches!(result, Err(AuthError::Unauthorized)));
    }
}
