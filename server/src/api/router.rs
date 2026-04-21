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
        game_service: GameService<U, G>,
        auth_service: AuthService<U, D>,
        pre_handler: GlobalPreHandler<G>,
    ) -> Self {
        Self {
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
    let auth_service = AuthService::new(user_repository, DiscordApiClient::new());
    let pre_handler = GlobalPreHandler::new(game_repository);
    let state = AppState::new(game_service, auth_service, pre_handler);
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
    next.run(request).await
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
