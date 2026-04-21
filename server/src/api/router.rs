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
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::middleware::from_fn_with_state;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::post;
use fs4::FileExt;
use serde::Deserialize;
use tokio::sync::Mutex as TokioMutex;

use super::ApiErrorResponse;
use super::CreateGameError;
use super::CreateGameRequest;
use super::GameRepository;
use super::GameService;
use super::GlobalPreHandler;
use super::SqliteGameRepository;
use super::SqliteUserRepository;
use super::UserRepository;
use super::handlers::CreateGameHandlerError;
use super::handlers::handle_create_game;

// ============================================================================
// definitions
// ============================================================================

/// Global lock file handle. Held for the lifetime of the server process.
static INSTANCE_LOCK: Mutex<Option<Arc<File>>> = Mutex::new(None);

pub async fn serve(addr: SocketAddr, db_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Ensure single instance
    acquire_instance_lock(db_path)?;

    let user_repository = SqliteUserRepository::new(db_path)?;
    let game_repository = SqliteGameRepository::new(db_path)?;
    let game_service = GameService::new(user_repository, game_repository.clone());
    let pre_handler = GlobalPreHandler::new(game_repository);
    let state = AppState::new(game_service, pre_handler);
    let router = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

pub(crate) fn create_router<U, G>(state: AppState<U, G>) -> Router
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    Router::new()
        .route("/games", post(post_games::<U, G>))
        .with_state(state.clone())
        .layer(from_fn_with_state(state, run_global_pre_handler::<U, G>))
}

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

pub(crate) struct AppState<U, G>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    game_service: Arc<GameService<U, G>>,
    pre_handler: Arc<GlobalPreHandler<G>>,
    game_update_lock: Arc<TokioMutex<()>>,
}

impl<U, G> Clone for AppState<U, G>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            game_service: Arc::clone(&self.game_service),
            pre_handler: Arc::clone(&self.pre_handler),
            game_update_lock: Arc::clone(&self.game_update_lock),
        }
    }
}

impl<U, G> AppState<U, G>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    pub(crate) fn new(game_service: GameService<U, G>, pre_handler: GlobalPreHandler<G>) -> Self {
        Self {
            game_service: Arc::new(game_service),
            pre_handler: Arc::new(pre_handler),
            game_update_lock: Arc::new(TokioMutex::new(())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PostGamesBody {
    face_type: i32,
    progress_mode: i32,
    duration_type: i32,
    start_date: String,
    first_period_hour: u8,
    requested_power: Option<String>,
}

async fn run_global_pre_handler<U, G>(State(state): State<AppState<U, G>>, request: Request, next: Next) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
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

async fn post_games<U, G>(
    State(state): State<AppState<U, G>>,
    headers: HeaderMap,
    Json(body): Json<PostGamesBody>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let request = CreateGameRequest {
        authorization,
        face_type: body.face_type,
        progress_mode: body.progress_mode,
        duration_type: body.duration_type,
        start_date: body.start_date,
        first_period_hour: body.first_period_hour,
        requested_power: body.requested_power,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    match tokio::task::spawn_blocking(move || match handle_create_game(&state_clone.game_service, request_clone) {
        Ok(response) => (StatusCode::CREATED, Json(response)).into_response(),
        Err(error) => {
            let status = match &error {
                CreateGameHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                CreateGameHandlerError::Service(CreateGameError::Unauthorized) => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(error.to_api_error_response())).into_response()
        }
    })
    .await
    {
        Ok(response) => response,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
