// ============================================================================
// imports
// ============================================================================

use axum::extract::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::NaiveDate;
use uuid::Uuid;

use super::AppState;
use super::CreateGameCommand;
use super::CreateGameError;
use super::CreateGameHandlerError;
use super::CreateGameRequest;
use super::CreateGameRequestBody;
use super::CreateGameRequestValidationError;
use super::CreateGameResponse;
use super::DiscordIdentityProvider;
use super::GameRepository;
use super::GameService;
use super::JoinGameCommand;
use super::JoinGameError;
use super::JoinGameHandlerError;
use super::JoinGameRequest;
use super::JoinGameRequestBody;
use super::JoinGameRequestValidationError;
use super::JoinGameResponse;
use super::Power;
use super::ProgressMode;
use super::Regulation;
use super::UserRepository;

// ============================================================================
// functions
// ============================================================================

///
/// 卓作成リクエスト Axum ハンドラ関数
///
pub(crate) async fn post_games<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    headers: HeaderMap,
    Json(body): Json<CreateGameRequestBody>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let request = CreateGameRequest {
        authorization,
        face_type: body.face_type,
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
                CreateGameHandlerError::Service(CreateGameError::InvalidRequest(_)) => StatusCode::BAD_REQUEST,
                CreateGameHandlerError::Service(CreateGameError::Unauthorized) => StatusCode::UNAUTHORIZED,
                CreateGameHandlerError::Service(CreateGameError::Forbidden(_)) => StatusCode::FORBIDDEN,
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

/// 卓作成リクエストハンドラ関数
fn handle_create_game<U, G>(
    service: &GameService<U, G>,
    request: CreateGameRequest,
) -> Result<CreateGameResponse, CreateGameHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(CreateGameHandlerError::InvalidRequest)?;

    let regulation = parse_regulation(&request)?;
    let requested_power = parse_requested_power(request.requested_power.as_deref(), || {
        invalid_request(CreateGameRequestValidationError::InvalidRequestedPower)
    })?;
    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .create_game(CreateGameCommand {
            access_token,
            regulation,
            requested_power,
        })
        .map_err(CreateGameHandlerError::Service)?;

    Ok(CreateGameResponse {
        game_uuid: result.game.uuid,
        owner_user_uuid: result.owner_user_uuid,
        requested_power: result.requested_power.map(|power| power.to_string()),
    })
}

/// 卓作成リクエストパラメータのレギュレーションをパースする
fn parse_regulation(request: &CreateGameRequest) -> Result<Regulation, CreateGameHandlerError> {
    let face_type = parse_enum(request.face_type, CreateGameRequestValidationError::InvalidFaceType)?;
    let duration_type = parse_enum(request.duration_type, CreateGameRequestValidationError::InvalidDurationType)?;
    let start_date = NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|_| invalid_request(CreateGameRequestValidationError::InvalidStartDate))?;

    Regulation::new(
        face_type,
        ProgressMode::Scheduled,
        duration_type,
        start_date,
        request.first_period_hour,
    )
    .map_err(|_| invalid_request(CreateGameRequestValidationError::InvalidFirstPeriodHour))
}

/// 卓作成リクエストパラメータの担当希望国をパースする
fn parse_requested_power<E, F>(value: Option<&str>, err: F) -> Result<Option<Power>, E>
where
    F: Fn() -> E,
{
    value.map(|code| Power::try_from(code).map_err(|_| err())).transpose()
}

/// 卓作成リクエストパラメータの列挙体をパースする
fn parse_enum<T>(value: i32, validation_error: CreateGameRequestValidationError) -> Result<T, CreateGameHandlerError>
where
    T: TryFrom<i32>,
{
    T::try_from(value).map_err(|_| invalid_request(validation_error))
}

/// 卓作成リクエスト無効エラーを生成する
fn invalid_request(validation_error: CreateGameRequestValidationError) -> CreateGameHandlerError {
    CreateGameHandlerError::InvalidRequest(validation_error)
}

///
/// 卓参加リクエスト Axum ハンドラ関数
///
pub(crate) async fn post_games_players<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<JoinGameRequestBody>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let game_uuid = match game_uuid_str.parse::<Uuid>() {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    JoinGameHandlerError::InvalidRequest(JoinGameRequestValidationError::InvalidGameUuid).to_api_error_response(),
                ),
            )
                .into_response();
        }
    };

    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let request = JoinGameRequest {
        authorization,
        game_uuid,
        requested_power: body.requested_power,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();
    match tokio::task::spawn_blocking(move || match handle_join_game(&state_clone.game_service, request_clone) {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            let status = match &error {
                JoinGameHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                JoinGameHandlerError::Service(JoinGameError::InvalidRequest(_)) => StatusCode::BAD_REQUEST,
                JoinGameHandlerError::Service(JoinGameError::Unauthorized) => StatusCode::UNAUTHORIZED,
                JoinGameHandlerError::Service(JoinGameError::NotFound) => StatusCode::NOT_FOUND,
                JoinGameHandlerError::Service(JoinGameError::Forbidden(_)) => StatusCode::FORBIDDEN,
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

/// 卓参加リクエストハンドラ関数
fn handle_join_game<U, G>(service: &GameService<U, G>, request: JoinGameRequest) -> Result<JoinGameResponse, JoinGameHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(JoinGameHandlerError::InvalidRequest)?;

    let requested_power = parse_requested_power(request.requested_power.as_deref(), || {
        JoinGameHandlerError::InvalidRequest(JoinGameRequestValidationError::InvalidRequestedPower)
    })?;

    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .join_game(JoinGameCommand {
            access_token,
            game_uuid: request.game_uuid,
            requested_power,
        })
        .map_err(JoinGameHandlerError::Service)?;

    Ok(JoinGameResponse {
        game_uuid: result.game.uuid,
        user_uuid: result.user_uuid,
        requested_power: result.requested_power.map(|power| power.to_string()),
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

    use chrono::DateTime;
    use chrono::FixedOffset;
    use chrono::Timelike;
    use chrono::Utc;

    use super::*;
    use crate::domain::Game;
    use crate::domain::GameStatus;
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::repositories::NewGame;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        rows_by_token: Rc<RefCell<HashMap<String, UserRecord>>>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows
                .into_iter()
                .map(|row| (row.access_token.clone(), row))
                .collect::<HashMap<_, _>>();
            Self {
                rows_by_token: Rc::new(RefCell::new(map)),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_uuid(&self, user_uuid: uuid::Uuid) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self
                .rows_by_token
                .borrow()
                .values()
                .find(|row| row.uuid == user_uuid)
                .cloned())
        }

        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows_by_token.borrow().get(access_token).cloned())
        }

        fn update_last_access_at_by_access_token(
            &self,
            access_token: &str,
            last_access_at: DateTime<Utc>,
        ) -> Result<bool, RepositoryError> {
            let mut rows = self.rows_by_token.borrow_mut();
            let Some(row) = rows.get_mut(access_token) else {
                return Ok(false);
            };

            row.last_access_at = last_access_at;
            Ok(true)
        }

        fn insert(&self, _new_user: NewUser) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn update_profile(&self, _id: UserId, _profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }
    }

    #[derive(Debug, Clone)]
    struct InMemoryGameRepository {
        inserted: Rc<RefCell<Vec<Game>>>,
        games: Rc<RefCell<Vec<Game>>>,
        updated: Rc<RefCell<Vec<Game>>>,
    }

    impl InMemoryGameRepository {
        fn new() -> Self {
            Self {
                inserted: Rc::new(RefCell::new(Vec::new())),
                games: Rc::new(RefCell::new(Vec::new())),
                updated: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn new_with_games(games: Vec<Game>) -> Self {
            Self {
                inserted: Rc::new(RefCell::new(Vec::new())),
                games: Rc::new(RefCell::new(games)),
                updated: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn last_inserted(&self) -> Option<Game> {
            self.inserted.borrow().last().cloned()
        }

        fn last_updated(&self) -> Option<Game> {
            self.updated.borrow().last().cloned()
        }
    }

    fn future_start_params() -> (String, u8) {
        let jst = FixedOffset::east_opt(9 * 60 * 60).expect("valid JST offset");
        let now_jst = Utc::now().with_timezone(&jst);
        let start_jst = now_jst + chrono::Duration::hours(2);
        (start_jst.format("%Y-%m-%d").to_string(), start_jst.hour() as u8)
    }

    impl GameRepository for InMemoryGameRepository {
        fn add_player(
            &self,
            game_uuid: uuid::Uuid,
            user_uuid: uuid::Uuid,
            requested_power: Option<Power>,
        ) -> Result<(), RepositoryError> {
            let mut games = self.games.borrow_mut();
            if let Some(game) = games.iter_mut().find(|g| g.uuid == game_uuid) {
                if !game.players.iter().any(|p| p.user_uuid == user_uuid) {
                    game.players.push(Player {
                        user_uuid,
                        power: None,
                        is_accepting_draw: false,
                        is_owner: false,
                        requested_power,
                    });
                }
                Ok(())
            } else {
                Err(RepositoryError::NotFound)
            }
        }
        fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError> {
            self.inserted.borrow_mut().push(new_game.game.clone());
            Ok(new_game.game)
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(Vec::new())
        }

        fn find_by_uuid(&self, game_uuid: uuid::Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(self.games.borrow().iter().find(|g| g.uuid == game_uuid).cloned())
        }

        fn find_progress_candidates(&self, _now: chrono::NaiveDateTime) -> Result<Vec<uuid::Uuid>, RepositoryError> {
            Ok(Vec::new())
        }

        fn exists_active_game_for_user(&self, _user_uuid: uuid::Uuid) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        fn update(&self, game: &Game) -> Result<(), RepositoryError> {
            self.updated.borrow_mut().push(game.clone());
            // active_games も更新して find_by_uuid が最新状態を返せるようにする
            let mut games = self.games.borrow_mut();
            if let Some(existing) = games.iter_mut().find(|g| g.uuid == game.uuid) {
                *existing = game.clone();
            }
            Ok(())
        }

        fn assign_game_number(&self, game_uuid: uuid::Uuid) -> Result<i32, RepositoryError> {
            // 既に採番済みか確認
            if let Some(n) = self
                .games
                .borrow()
                .iter()
                .find(|g| g.uuid == game_uuid)
                .and_then(|g| g.game_number)
            {
                return Ok(n);
            }
            // 現在の最大値を取得
            let max = self.games.borrow().iter().filter_map(|g| g.game_number).max().unwrap_or(0);
            let next = max + 1;
            let mut games = self.games.borrow_mut();
            let game = games
                .iter_mut()
                .find(|g| g.uuid == game_uuid)
                .ok_or(RepositoryError::NotFound)?;
            game.game_number = Some(next);
            Ok(next)
        }
    }

    #[test]
    fn handle_create_game_creates_owner_and_ready_phase() {
        let (start_date, first_period_hour) = future_start_params();

        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);

        let response = handle_create_game(
            &service,
            CreateGameRequest {
                authorization: "Bearer token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: Some("f".to_string()),
            },
        )
        .expect("create should succeed");

        assert_eq!(response.owner_user_uuid, owner_uuid);
        assert_eq!(response.requested_power.as_deref(), Some("France"));
    }

    #[test]
    fn handle_create_game_uses_scheduled_progress_mode() {
        let (start_date, first_period_hour) = future_start_params();

        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game_repository = InMemoryGameRepository::new();
        let repo_clone = game_repository.clone();
        let service = GameService::new(user_repository, game_repository);

        handle_create_game(
            &service,
            CreateGameRequest {
                authorization: "Bearer token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: None,
            },
        )
        .expect("create should succeed");

        let game = repo_clone.last_inserted().expect("a game should have been inserted");
        assert_eq!(game.regulation.progress_mode, ProgressMode::Scheduled);
    }

    #[test]
    fn handle_create_game_rejects_invalid_authorization() {
        let user_repository = InMemoryUserRepository::new(Vec::new());
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);

        let error = handle_create_game(
            &service,
            CreateGameRequest {
                authorization: "token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date: "2026-04-19".to_string(),
                first_period_hour: 12,
                requested_power: None,
            },
        )
        .expect_err("create should fail");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_join_game_registers_player() {
        let owner_uuid = uuid::Uuid::now_v7();
        let joiner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 2,
            uuid: joiner_uuid,
            discord_user_id: "1002".to_string(),
            username: "joiner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-2".to_string(),
            last_access_at: Utc::now(),
        }]);

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            regulation: {
                use crate::domain::DurationType;
                use crate::domain::FaceType;
                use crate::domain::Regulation;
                let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
                let now_jst = Utc::now().with_timezone(&jst);
                let start = now_jst + chrono::Duration::hours(2);
                Regulation::new(
                    FaceType::Girls,
                    ProgressMode::Scheduled,
                    DurationType::Short,
                    start.date_naive(),
                    start.hour() as u8,
                )
                .unwrap()
            },
            players: vec![Player {
                user_uuid: owner_uuid,
                power: None,
                is_accepting_draw: false,
                is_owner: true,
                requested_power: None,
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };
        let game_uuid = game.uuid;

        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let repo_clone = game_repository.clone();
        let service = GameService::new(user_repository, game_repository);

        let response = handle_join_game(
            &service,
            JoinGameRequest {
                authorization: "Bearer token-2".to_string(),
                game_uuid,
                requested_power: Some("f".to_string()),
            },
        )
        .expect("join should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert_eq!(response.user_uuid, joiner_uuid);
        assert_eq!(response.requested_power.as_deref(), Some("France"));

        let updated = repo_clone.last_updated().expect("game should have been updated");
        assert_eq!(updated.players.len(), 2);
    }

    #[test]
    fn handle_join_game_rejects_invalid_authorization() {
        let user_repository = InMemoryUserRepository::new(Vec::new());
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);

        let error = handle_join_game(
            &service,
            JoinGameRequest {
                authorization: "token-1".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                requested_power: None,
            },
        )
        .expect_err("join should fail");

        assert_eq!(error.code(), "invalid_request");
    }
}
