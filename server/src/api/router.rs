// ============================================================================
// imports
// ============================================================================

use std::error::Error;
use std::fs::File;
use std::io::Error as IoError;
use std::io::ErrorKind::WouldBlock;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use axum::Router;
use axum::extract::Json;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::post;
use chrono::DateTime;
use chrono::Utc;
use fs4::FileExt;
use tokio::sync::Mutex as TokioMutex;

use super::ApiErrorResponse;
use super::AuthService;
use super::DiscordApiClient;
use super::DiscordIdentityProvider;
use super::GameRepository;
use super::GameService;
use super::GlobalPreHandler;
use super::SqliteGameRepository;
use super::SqliteUserRepository;
use super::UserRepository;
use super::handlers::post_auth_login;
use super::handlers::post_games;

// ============================================================================
// definitions
// ============================================================================

/// グローバルロックファイルハンドル
static INSTANCE_LOCK: Mutex<Option<Arc<File>>> = Mutex::new(None);

///
/// ルーター起動パラメータ構造体
///
pub(crate) struct AppState<U, G, D>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    pub user_repository: Arc<U>,
    pub game_service: Arc<GameService<U, G>>,
    pub auth_service: Arc<AuthService<U, D>>,
    pub pre_handler: Arc<GlobalPreHandler<G>>,
    pub game_update_lock: Arc<TokioMutex<()>>,
}

/// ルーター起動パラメータ構造体の実装
impl<U, G, D> AppState<U, G, D>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    pub(crate) fn new(
        user_repository: U,
        game_service: GameService<U, G>,
        auth_service: AuthService<U, D>,
        pre_handler: GlobalPreHandler<G>,
    ) -> Self {
        Self {
            user_repository: Arc::new(user_repository),
            game_service: Arc::new(game_service),
            auth_service: Arc::new(auth_service),
            pre_handler: Arc::new(pre_handler),
            game_update_lock: Arc::new(TokioMutex::new(())),
        }
    }
}

/// ルーター起動パラメータ構造体の実装（Clone トレイト）
impl<U, G, D> Clone for AppState<U, G, D>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            user_repository: Arc::clone(&self.user_repository),
            game_service: Arc::clone(&self.game_service),
            auth_service: Arc::clone(&self.auth_service),
            pre_handler: Arc::clone(&self.pre_handler),
            game_update_lock: Arc::clone(&self.game_update_lock),
        }
    }
}

// ============================================================================
// functions
// ============================================================================

///
/// API サーバーを起動する非同期関数
///
pub async fn serve(addr: SocketAddr, db_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 複数起動防止インスタンスロック獲得
    acquire_instance_lock(db_path)?;

    let user_repository = SqliteUserRepository::new(db_path)?;
    let game_repository = SqliteGameRepository::new(db_path)?;
    let game_service = GameService::new(user_repository.clone(), game_repository.clone());
    let auth_service = AuthService::new(user_repository.clone(), DiscordApiClient::new());
    let pre_handler = GlobalPreHandler::new(game_repository);
    let state = AppState::new(user_repository.clone(), game_service, auth_service, pre_handler);
    let router = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

///
/// グローバルロック獲得関数
///
fn acquire_instance_lock(db_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let lock_path = format!("{}.lock", db_path);
    let file = File::create(&lock_path)?;

    match file.try_lock_exclusive() {
        Ok(()) => {
            let mut lock = INSTANCE_LOCK
                .lock()
                .map_err(|_| IoError::other(format!("instance lock mutex was poisoned while acquiring {}", lock_path)))?;
            *lock = Some(Arc::new(file));
            println!("[LOCK] Acquired exclusive lock on {}", lock_path);
            Ok(())
        }
        Err(err) if err.kind() == WouldBlock => {
            Err(format!("Another instance is already running (lock file: {})", lock_path).into())
        }
        Err(err) => Err(format!("Failed to acquire instance lock on {}: {}", lock_path, err).into()),
    }
}

///
/// グローバルプリハンドラ実行非同期関数
///
async fn run_global_pre_handler<U, G, D>(State(state): State<AppState<U, G, D>>, request: Request, next: Next) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    // Serialize progression updates (not entire request) through lock.
    let _game_update_guard = state.game_update_lock.lock().await;

    let pre_handler = Arc::clone(&state.pre_handler);
    let pre_handler_result = tokio::task::spawn_blocking(move || pre_handler.run()).await;

    match pre_handler_result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "pre_handler_failed",
                    message: error.to_string(),
                }),
            )
                .into_response();
        }
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "pre_handler_failed",
                    message: format!("failed to execute pre-handler task: {}", error),
                }),
            )
                .into_response();
        }
    }

    drop(_game_update_guard);

    let authorization = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let user_repository = Arc::clone(&state.user_repository);
    let touch_result = tokio::task::spawn_blocking(move || {
        touch_last_access_at_from_authorization(user_repository.as_ref(), authorization.as_deref(), Utc::now())
    })
    .await;

    match touch_result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "pre_handler_failed",
                    message: format!("failed to update last_access_at: {}", error),
                }),
            )
                .into_response();
        }
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "pre_handler_failed",
                    message: format!("failed to execute last_access_at task: {}", error),
                }),
            )
                .into_response();
        }
    }

    next.run(request).await
}

fn touch_last_access_at_from_authorization<U>(
    user_repository: &U,
    authorization: Option<&str>,
    last_access_at: DateTime<Utc>,
) -> Result<(), super::RepositoryError>
where
    U: UserRepository,
{
    let Some(access_token) = extract_bearer_access_token(authorization) else {
        return Ok(());
    };

    let _ = user_repository.update_last_access_at_by_access_token(&access_token, last_access_at)?;
    Ok(())
}

fn extract_bearer_access_token(authorization: Option<&str>) -> Option<String> {
    let auth = authorization?.trim();
    if !auth.starts_with("Bearer ") {
        return None;
    }

    let token = auth.trim_start_matches("Bearer ").trim();
    if token.is_empty() {
        return None;
    }

    Some(token.to_string())
}

///
/// API ルーター生成関数
///
fn create_router<U, G, D>(state: AppState<U, G, D>) -> Router
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    Router::new()
        .route("/games", post(post_games::<U, G, D>))
        .route("/auth/login", post(post_auth_login::<U, G, D>))
        .with_state(state.clone())
        .layer(from_fn_with_state(state, run_global_pre_handler::<U, G, D>))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::repositories::NewUser;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        user: std::sync::Arc<std::sync::Mutex<Option<UserRecord>>>,
    }

    impl InMemoryUserRepository {
        fn new(user: Option<UserRecord>) -> Self {
            Self {
                user: std::sync::Arc::new(std::sync::Mutex::new(user)),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, super::super::RepositoryError> {
            Ok(self.user.lock().expect("lock should succeed").clone())
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, super::super::RepositoryError> {
            Ok(self
                .user
                .lock()
                .expect("lock should succeed")
                .clone()
                .filter(|user| user.access_token == access_token))
        }

        fn update_last_access_at_by_access_token(
            &self,
            access_token: &str,
            last_access_at: DateTime<Utc>,
        ) -> Result<bool, super::super::RepositoryError> {
            let mut user = self.user.lock().expect("lock should succeed");
            let Some(row) = user.as_mut().filter(|row| row.access_token == access_token) else {
                return Ok(false);
            };

            row.last_access_at = last_access_at;
            Ok(true)
        }

        fn insert(&self, _new_user: NewUser) -> Result<UserRecord, super::super::RepositoryError> {
            Err(super::super::RepositoryError::Unavailable("not used".to_string()))
        }

        fn update_profile(&self, _id: UserId, _profile: UserProfileUpdate) -> Result<UserRecord, super::super::RepositoryError> {
            Err(super::super::RepositoryError::Unavailable("not used".to_string()))
        }
    }

    #[test]
    fn touch_last_access_at_from_authorization_updates_known_user() {
        let initial = Utc::now() - chrono::TimeDelta::minutes(10);
        let repository = InMemoryUserRepository::new(Some(UserRecord {
            id: 1,
            uuid: uuid::Uuid::now_v7(),
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: initial,
        }));
        let touched_at = Utc::now();

        touch_last_access_at_from_authorization(&repository, Some("Bearer token-1"), touched_at).expect("touch should succeed");

        let updated = repository
            .find_by_access_token("token-1")
            .expect("find should succeed")
            .expect("user should exist");
        assert_eq!(updated.last_access_at, touched_at);
    }

    #[test]
    fn touch_last_access_at_from_authorization_ignores_unknown_token() {
        let initial = Utc::now() - chrono::TimeDelta::minutes(10);
        let repository = InMemoryUserRepository::new(Some(UserRecord {
            id: 1,
            uuid: uuid::Uuid::now_v7(),
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: initial,
        }));

        touch_last_access_at_from_authorization(&repository, Some("Bearer missing-token"), Utc::now())
            .expect("touch should succeed");

        let unchanged = repository
            .find_by_access_token("token-1")
            .expect("find should succeed")
            .expect("user should exist");
        assert_eq!(unchanged.last_access_at, initial);
    }

    #[test]
    fn extract_bearer_access_token_rejects_invalid_authorization() {
        assert_eq!(extract_bearer_access_token(None), None);
        assert_eq!(extract_bearer_access_token(Some("Basic abc")), None);
        assert_eq!(extract_bearer_access_token(Some("Bearer   ")), None);
        assert_eq!(
            extract_bearer_access_token(Some("Bearer token-1")),
            Some("token-1".to_string())
        );
    }
}
