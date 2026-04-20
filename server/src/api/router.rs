// ============================================================================
// imports
// ============================================================================

// standard library
use std::sync::Arc;

// external crates
use axum::Router;
use axum::extract::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::post;
use serde::Deserialize;
use tokio::sync::Mutex;

// structs
use super::ApiErrorResponse;
use super::CreateGameRequest;
use super::GlobalPreHandler;

// enums
use super::CreateGameError;
use super::GameService;

// handler
use super::handlers::CreateGameHandlerError;
use super::handlers::handle_create_game;

// traits
use super::GameRepository;
use super::UserRepository;

// ============================================================================
// definitions
// ============================================================================

pub(crate) struct AppState<U, G>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    game_service: Arc<GameService<U, G>>,
    pre_handler: Arc<GlobalPreHandler<G>>,
    game_update_lock: Arc<Mutex<()>>,
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
            game_update_lock: Arc::new(Mutex::new(())),
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

async fn run_game_write<U, G, F>(state: &AppState<U, G>, operation: F) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    F: FnOnce() -> Response,
{
    let _game_update_guard = state.game_update_lock.lock().await;

    if let Err(error) = state.pre_handler.run() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiErrorResponse {
                code: "pre_handler_failed",
                message: error.to_string(),
            }),
        )
            .into_response();
    }

    operation()
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

    run_game_write(&state, || match handle_create_game(&state.game_service, request) {
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
}

pub(crate) fn create_router<U, G>(state: AppState<U, G>) -> Router
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    Router::new().route("/games", post(post_games::<U, G>)).with_state(state)
}
