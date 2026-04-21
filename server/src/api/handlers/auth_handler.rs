// ============================================================================
// imports
// ============================================================================

use std::sync::Arc;

use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::AppState;
use super::AuthError;
use super::AuthHandlerError;
use super::AuthLoginRequest;
use super::AuthLoginResponse;
use super::AuthLoginResponseUser;
use super::AuthService;
use super::DiscordClientError;
use super::DiscordIdentityProvider;
use super::GameRepository;
use super::LoginCommand;
use super::RepositoryError;
use super::UserRepository;

// ============================================================================
// functions
// ============================================================================

///
/// ログインリクエスト Axum ハンドラ関数
///
pub(crate) async fn post_auth_login<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Json(body): Json<AuthLoginRequest>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let auth_service = Arc::clone(&state.auth_service);
    match tokio::task::spawn_blocking(move || handle_auth_login(&auth_service, body)).await {
        Ok(Ok(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(Err(error)) => {
            let status = match &error {
                AuthHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                AuthHandlerError::Service(AuthError::DiscordClient(DiscordClientError::Unauthorized)) => StatusCode::UNAUTHORIZED,
                AuthHandlerError::Service(AuthError::DiscordClient(DiscordClientError::Unavailable(_))) => {
                    StatusCode::BAD_GATEWAY
                }
                AuthHandlerError::Service(AuthError::Repository(RepositoryError::Conflict)) => StatusCode::CONFLICT,
                AuthHandlerError::Service(AuthError::Repository(RepositoryError::Unavailable(_))) => {
                    StatusCode::SERVICE_UNAVAILABLE
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(error.to_api_error_response())).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

///
/// ログインリクエストハンドラ関数
///
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
        user: AuthLoginResponseUser::from(result.user),
    })
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;
    use crate::api::requests::AuthRequestValidationError;
    use crate::repositories::DiscordProfile;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct FakeDiscordIdentityProvider {
        profile: DiscordProfile,
    }

    impl DiscordIdentityProvider for FakeDiscordIdentityProvider {
        fn fetch_profile(&self, _discord_access_token: &str) -> Result<DiscordProfile, DiscordClientError> {
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

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self
                .state
                .borrow()
                .rows
                .values()
                .find(|row| row.access_token == access_token)
                .cloned())
        }

        fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError> {
            let mut state = self.state.borrow_mut();
            if state.rows.contains_key(&new_user.discord_user_id) {
                return Err(RepositoryError::Conflict);
            }

            let row = UserRecord {
                id: 2,
                uuid: new_user.uuid,
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

        fn update_profile(&self, id: UserId, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
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
    fn handle_auth_login_returns_login_response() {
        let repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: uuid::Uuid::now_v7(),
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
        assert_eq!(response.user.discord_user_id, "1001");
        assert_eq!(response.user.username, "nemu");
        assert_eq!(response.user.global_name.as_deref(), Some("asagi"));
        assert_eq!(response.user.avatar_hash.as_deref(), Some("hash"));
        assert_eq!(
            response.user.avatar_url.as_deref(),
            Some("https://cdn.discordapp.com/avatar.png")
        );
        assert_eq!(
            response
                .user
                .global_name
                .as_deref()
                .unwrap_or(response.user.username.as_str()),
            "asagi"
        );
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
