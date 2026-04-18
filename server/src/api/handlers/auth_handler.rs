use crate::api::requests::AuthLoginRequest;
use crate::api::requests::AuthRequestValidationError;
use crate::api::responses::ApiErrorResponse;
use crate::api::responses::AuthLoginResponse;
use crate::api::responses::AuthLoginUserResponse;
use crate::repositories::UserRepository;
use crate::services::AuthError;
use crate::services::AuthService;
use crate::services::DiscordClientError;
use crate::services::DiscordIdentityProvider;
use crate::services::LoginCommand;

pub(crate) fn handle_auth_login<U, D>(
    service: &AuthService<U, D>,
    request: AuthLoginRequest,
) -> Result<AuthLoginResponse, AuthHandlerError>
where
    U: UserRepository,
    D: DiscordIdentityProvider,
{
    request.validate().map_err(AuthHandlerError::InvalidRequest)?;

    let result = service
        .login(LoginCommand {
            discord_access_token: request.discord_access_token,
        })
        .map_err(AuthHandlerError::Service)?;

    Ok(AuthLoginResponse {
        access_token: result.access_token,
        user: AuthLoginUserResponse {
            discord_user_id: result.user.discord_user_id,
            username: result.user.username,
            global_name: result.user.global_name,
            avatar_hash: result.user.avatar_hash,
            avatar_url: result.user.avatar_url,
        },
    })
}

#[derive(Debug)]
pub(crate) enum AuthHandlerError {
    InvalidRequest(AuthRequestValidationError),
    Service(AuthError),
}

impl AuthHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(AuthError::InvalidRequest(_)) => "invalid_request",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unauthorized)) => "unauthorized",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unavailable(_))) => "discord_unavailable",
            Self::Service(AuthError::Repository(_)) => "repository_error",
        }
    }

    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(AuthRequestValidationError::MissingDiscordAccessToken) => {
                "discord_access_token is required".to_string()
            }
            Self::Service(error) => error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;
    use crate::repositories::DiscordProfile;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct FakeDiscordIdentityProvider {
        profile: DiscordProfile,
    }

    impl DiscordIdentityProvider for FakeDiscordIdentityProvider {
        fn fetch_profile(&self, _discord_access_token: &str) -> Result<DiscordProfile, crate::services::DiscordClientError> {
            Ok(self.profile.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        state: Rc<RefCell<State>>,
    }

    #[derive(Debug, Clone)]
    struct State {
        rows: HashMap<String, UserRecord>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows.into_iter().map(|row| (row.discord_user_id.clone(), row)).collect();
            Self {
                state: Rc::new(RefCell::new(State { rows: map })),
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
                id: 2,
                discord_user_id: new_user.discord_user_id,
                username: new_user.username,
                global_name: new_user.global_name,
                avatar_hash: new_user.avatar_hash,
                avatar_url: new_user.avatar_url,
                access_token: new_user.access_token,
            };
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
    fn handle_auth_login_returns_login_response() {
        let repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            discord_user_id: "1001".to_string(),
            username: "old_user".to_string(),
            global_name: Some("old".to_string()),
            avatar_hash: Some("old_hash".to_string()),
            avatar_url: Some("https://cdn.discordapp.com/old.png".to_string()),
            access_token: "token-1".to_string(),
        }]);

        let discord = FakeDiscordIdentityProvider {
            profile: DiscordProfile {
                discord_user_id: "1001".to_string(),
                username: "nemu".to_string(),
                global_name: Some("asagi".to_string()),
                avatar_hash: Some("hash".to_string()),
                avatar_url: Some("https://cdn.discordapp.com/avatar.png".to_string()),
            },
        };

        let service = AuthService::new(repository, discord);

        let response = handle_auth_login(
            &service,
            AuthLoginRequest {
                discord_access_token: "valid-token".to_string(),
            },
        )
        .expect("handler should succeed");

        assert_eq!(response.access_token, "token-1");
        assert_eq!(response.user.global_name.as_deref().unwrap_or(response.user.username.as_str()), "asagi");
    }

    #[test]
    fn handle_auth_login_rejects_empty_token() {
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

        let error = handle_auth_login(
            &service,
            AuthLoginRequest {
                discord_access_token: "  ".to_string(),
            },
        )
        .expect_err("handler should fail");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn auth_handler_error_builds_api_error_response() {
        let error = AuthHandlerError::InvalidRequest(AuthRequestValidationError::MissingDiscordAccessToken);
        let response = error.to_api_error_response();

        assert_eq!(response.code, "invalid_request");
        assert_eq!(response.message, "discord_access_token is required");
    }
}
