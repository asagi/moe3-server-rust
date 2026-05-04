// ============================================================================
// imports
// ============================================================================

use axum::extract::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
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
use super::DeleteTerritoryQueryParams;
use super::DeleteUnitQueryParams;
use super::DiscordIdentityProvider;
use super::GameRepository;
use super::GameService;
use super::GameStatus;
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
use super::SetDrawProposalCommand;
use super::SetDrawProposalError;
use super::SetDrawProposalHandlerError;
use super::SetDrawProposalRequest;
use super::SetDrawProposalRequestBody;
use super::SetDrawProposalRequestValidationError;
use super::SetDrawProposalResponse;
use super::SetTerritoryCommand;
use super::SetTerritoryError;
use super::SetTerritoryHandlerError;
use super::SetTerritoryRequest;
use super::SetTerritoryRequestBody;
use super::SetTerritoryRequestValidationError;
use super::SetTerritoryResponse;
use super::SetUnitCommand;
use super::SetUnitError;
use super::SetUnitHandlerError;
use super::SetUnitRequest;
use super::SetUnitRequestBody;
use super::SetUnitRequestValidationError;
use super::SetUnitResponse;
use super::SqliteMessageRepository;
use super::UnitResponseBody;
use super::UnitSpec;
use super::UnitSpecBody;
use super::User;
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
        keyword: body.keyword,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    match tokio::task::spawn_blocking(move || {
        match handle_create_game(
            &state_clone.game_service,
            state_clone.user_repository.as_ref(),
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
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
    user_repository: &U,
    message_repository: &SqliteMessageRepository,
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
    let keyword = parse_keyword(request.keyword.as_deref(), || {
        invalid_request(CreateGameRequestValidationError::InvalidKeyword)
    })?;
    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .create_game(CreateGameCommand {
            access_token,
            regulation,
            requested_power,
            keyword,
        })
        .map_err(CreateGameHandlerError::Service)?;

    match user_repository.find_by_uuid(result.owner_user_uuid) {
        Ok(Some(owner_record)) => {
            let owner = User {
                uuid: owner_record.uuid,
                discord_user_id: owner_record.discord_user_id,
                username: owner_record.username,
                global_name: owner_record.global_name,
                avatar_hash: owner_record.avatar_hash,
                avatar_url: owner_record.avatar_url,
            };

            if let Err(error) = message_repository.create_game_db_with_game_created_message(result.game.uuid, &owner) {
                eprintln!(
                    "failed to persist GameCreated message (game_uuid={}): {}",
                    result.game.uuid, error
                );
            }
        }
        Ok(None) => {
            eprintln!(
                "skip GameCreated message: owner user not found (owner_user_uuid={}, game_uuid={})",
                result.owner_user_uuid, result.game.uuid
            );
        }
        Err(error) => {
            eprintln!(
                "skip GameCreated message due to user lookup failure (owner_user_uuid={}, game_uuid={}): {}",
                result.owner_user_uuid, result.game.uuid, error
            );
        }
    }

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

/// keyword をバリデートしトリムする。大小英字と数字のみ許可。
fn parse_keyword<E, F>(value: Option<&str>, err: F) -> Result<Option<String>, E>
where
    F: Fn() -> E,
{
    let Some(raw) = value else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(err());
    }
    Ok(Some(trimmed.to_string()))
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
        keyword: body.keyword,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();
    match tokio::task::spawn_blocking(move || {
        match handle_join_game(
            &state_clone.game_service,
            state_clone.user_repository.as_ref(),
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
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
        }
    })
    .await
    {
        Ok(response) => response,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// 卓参加リクエストハンドラ関数
fn handle_join_game<U, G>(
    service: &GameService<U, G>,
    user_repository: &U,
    message_repository: &SqliteMessageRepository,
    request: JoinGameRequest,
) -> Result<JoinGameResponse, JoinGameHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(JoinGameHandlerError::InvalidRequest)?;

    let requested_power = parse_requested_power(request.requested_power.as_deref(), || {
        JoinGameHandlerError::InvalidRequest(JoinGameRequestValidationError::InvalidRequestedPower)
    })?;

    let keyword = parse_keyword(request.keyword.as_deref(), || {
        JoinGameHandlerError::InvalidRequest(JoinGameRequestValidationError::InvalidKeyword)
    })?;

    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .join_game(JoinGameCommand {
            access_token,
            game_uuid: request.game_uuid,
            requested_power,
            keyword,
        })
        .map_err(JoinGameHandlerError::Service)?;

    match user_repository.find_by_uuid(result.user_uuid) {
        Ok(Some(joined_user_record)) => {
            let joined_user = User {
                uuid: joined_user_record.uuid,
                discord_user_id: joined_user_record.discord_user_id,
                username: joined_user_record.username,
                global_name: joined_user_record.global_name,
                avatar_hash: joined_user_record.avatar_hash,
                avatar_url: joined_user_record.avatar_url,
            };

            if let Err(error) = message_repository.append_player_joined_message(result.game.uuid, &joined_user) {
                eprintln!(
                    "failed to persist PlayerJoined message (game_uuid={}, user_uuid={}): {}",
                    result.game.uuid, result.user_uuid, error
                );
            }
        }
        Ok(None) => {
            eprintln!(
                "skip PlayerJoined message: joined user not found (user_uuid={}, game_uuid={})",
                result.user_uuid, result.game.uuid
            );
        }
        Err(error) => {
            eprintln!(
                "skip PlayerJoined message due to user lookup failure (user_uuid={}, game_uuid={}): {}",
                result.user_uuid, result.game.uuid, error
            );
        }
    }

    if result.game.status == GameStatus::Ready
        && let Err(error) = message_repository.append_ready_message(result.game.uuid)
    {
        eprintln!("failed to persist Ready message (game_uuid={}): {}", result.game.uuid, error);
    }

    Ok(JoinGameResponse {
        game_uuid: result.game.uuid,
        user_uuid: result.user_uuid,
        requested_power: result.requested_power.map(|power| power.to_string()),
    })
}

///
/// 和平終了フラグ設定リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_admin_games_draw_proposal<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SetDrawProposalRequestBody>,
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
                    SetDrawProposalHandlerError::InvalidRequest(SetDrawProposalRequestValidationError::InvalidGameUuid)
                        .to_api_error_response(),
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

    let request = SetDrawProposalRequest {
        authorization,
        game_uuid,
        draw_proposal: body.draw_proposal,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        match handle_set_draw_proposal(
            &state_clone.game_service,
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(error) => {
                let status = match &error {
                    SetDrawProposalHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                    SetDrawProposalHandlerError::Service(SetDrawProposalError::Unauthorized) => StatusCode::UNAUTHORIZED,
                    SetDrawProposalHandlerError::Service(SetDrawProposalError::NotFound) => StatusCode::NOT_FOUND,
                    SetDrawProposalHandlerError::Service(SetDrawProposalError::Forbidden(_)) => StatusCode::FORBIDDEN,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, Json(error.to_api_error_response())).into_response()
            }
        }
    })
    .await;
    drop(_game_update_guard);
    match result {
        Ok(response) => response,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// 和平終了フラグ設定リクエストハンドラ関数
fn handle_set_draw_proposal<U, G>(
    service: &GameService<U, G>,
    message_repository: &SqliteMessageRepository,
    request: SetDrawProposalRequest,
) -> Result<SetDrawProposalResponse, SetDrawProposalHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(SetDrawProposalHandlerError::InvalidRequest)?;

    let access_token = request.authorization.trim().trim_start_matches("Bearer ").trim().to_string();

    let result = service
        .set_draw_proposal(SetDrawProposalCommand {
            access_token,
            game_uuid: request.game_uuid,
            draw_proposal: request.draw_proposal,
        })
        .map_err(SetDrawProposalHandlerError::Service)?;

    if result.changed {
        let turn = result.game.current_turn();
        if request.draw_proposal {
            if let Err(error) = message_repository.append_draw_proposed_message(result.game.uuid, &turn) {
                eprintln!(
                    "failed to persist DrawProposed message (game_uuid={}): {}",
                    result.game.uuid, error
                );
            }
        } else if let Err(error) = message_repository.append_draw_rescinded_message(result.game.uuid, &turn) {
            eprintln!(
                "failed to persist DrawRescinded message (game_uuid={}): {}",
                result.game.uuid, error
            );
        }
    }

    Ok(SetDrawProposalResponse {
        game_uuid: result.game.uuid,
        draw_proposal: request.draw_proposal,
    })
}

///
/// ユニット配置制御リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_admin_games_units<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path((game_uuid_str, location)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<SetUnitRequestBody>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    dispatch_set_unit(state, game_uuid_str, location, headers, Some(body.unit), body.season).await
}

///
/// ユニット削除リクエスト Axum ハンドラ関数
///
pub(crate) async fn delete_admin_games_units<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path((game_uuid_str, location)): Path<(String, String)>,
    headers: HeaderMap,
    Query(params): Query<DeleteUnitQueryParams>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    dispatch_set_unit(state, game_uuid_str, location, headers, None, params.season).await
}

async fn dispatch_set_unit<U, G, D>(
    state: AppState<U, G, D>,
    game_uuid_str: String,
    location: String,
    headers: HeaderMap,
    unit: Option<UnitSpecBody>,
    season: String,
) -> Response
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
                Json(SetUnitHandlerError::InvalidRequest(SetUnitRequestValidationError::InvalidGameUuid).to_api_error_response()),
            )
                .into_response();
        }
    };

    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let request = SetUnitRequest {
        authorization,
        game_uuid,
        unit,
        location,
        season,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        match handle_set_unit(
            &state_clone.game_service,
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(error) => {
                let status = match &error {
                    SetUnitHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                    SetUnitHandlerError::Service(SetUnitError::Unauthorized) => StatusCode::UNAUTHORIZED,
                    SetUnitHandlerError::Service(SetUnitError::NotFound) => StatusCode::NOT_FOUND,
                    SetUnitHandlerError::Service(SetUnitError::Forbidden(_)) => StatusCode::FORBIDDEN,
                    SetUnitHandlerError::Service(SetUnitError::InvalidRequest(_)) => StatusCode::BAD_REQUEST,
                    SetUnitHandlerError::Service(SetUnitError::PhaseConflict) => StatusCode::CONFLICT,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, Json(error.to_api_error_response())).into_response()
            }
        }
    })
    .await;
    drop(_game_update_guard);
    match result {
        Ok(response) => response,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// ユニット配置制御リクエストハンドラ関数
fn handle_set_unit<U, G>(
    service: &GameService<U, G>,
    message_repository: &SqliteMessageRepository,
    request: SetUnitRequest,
) -> Result<SetUnitResponse, SetUnitHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(SetUnitHandlerError::InvalidRequest)?;

    let access_token = request.authorization.trim()[7..].trim().to_string();

    let unit_spec = request.unit.map(|b| UnitSpec {
        power_symbol: b.power,
        kind_str: b.kind,
    });

    let result = service
        .set_unit(SetUnitCommand {
            access_token,
            game_uuid: request.game_uuid,
            location: request.location.clone(),
            unit: unit_spec,
            season: request.season,
        })
        .map_err(SetUnitHandlerError::Service)?;

    let turn = result.game.current_turn();
    let game_uuid = result.game.uuid;

    if result.changed {
        match (result.old_unit, result.new_unit) {
            (None, Some(new_unit)) => {
                if let Err(error) = message_repository.append_unit_placed_message(game_uuid, &turn, new_unit) {
                    eprintln!("failed to persist UnitPlaced message (game_uuid={}): {}", game_uuid, error);
                }
            }
            (Some(old_unit), Some(new_unit)) => {
                if let Err(error) = message_repository.append_unit_replaced_message(game_uuid, &turn, old_unit, new_unit) {
                    eprintln!("failed to persist UnitReplaced message (game_uuid={}): {}", game_uuid, error);
                }
            }
            (Some(old_unit), None) => {
                if let Err(error) = message_repository.append_unit_removed_message(game_uuid, &turn, old_unit) {
                    eprintln!("failed to persist UnitRemoved message (game_uuid={}): {}", game_uuid, error);
                }
            }
            (None, None) => {}
        }
    }

    let unit_body = result.new_unit.map(|u| UnitResponseBody {
        power: u.power.symbol().to_string(),
        kind: u.symbol().to_string(),
    });

    Ok(SetUnitResponse {
        game_uuid,
        location: request.location,
        unit: unit_body,
    })
}

///
/// 占領情報登録リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_admin_games_territories<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path((game_uuid_str, code)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<SetTerritoryRequestBody>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    dispatch_set_territory(state, game_uuid_str, code, headers, Some(body.power), body.season).await
}

///
/// 占領情報削除リクエスト Axum ハンドラ関数
///
pub(crate) async fn delete_admin_games_territories<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path((game_uuid_str, code)): Path<(String, String)>,
    headers: HeaderMap,
    Query(params): Query<DeleteTerritoryQueryParams>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    dispatch_set_territory(state, game_uuid_str, code, headers, None, params.season).await
}

async fn dispatch_set_territory<U, G, D>(
    state: AppState<U, G, D>,
    game_uuid_str: String,
    code: String,
    headers: HeaderMap,
    power: Option<String>,
    season: String,
) -> Response
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
                    SetTerritoryHandlerError::InvalidRequest(SetTerritoryRequestValidationError::InvalidGameUuid)
                        .to_api_error_response(),
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

    let request = SetTerritoryRequest {
        authorization,
        game_uuid,
        code,
        power,
        season,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        match handle_set_territory(
            &state_clone.game_service,
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(error) => {
                let status = match &error {
                    SetTerritoryHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                    SetTerritoryHandlerError::Service(SetTerritoryError::Unauthorized) => StatusCode::UNAUTHORIZED,
                    SetTerritoryHandlerError::Service(SetTerritoryError::NotFound) => StatusCode::NOT_FOUND,
                    SetTerritoryHandlerError::Service(SetTerritoryError::WaterProvince) => StatusCode::NOT_FOUND,
                    SetTerritoryHandlerError::Service(SetTerritoryError::Forbidden(_)) => StatusCode::FORBIDDEN,
                    SetTerritoryHandlerError::Service(SetTerritoryError::InvalidRequest(_)) => StatusCode::BAD_REQUEST,
                    SetTerritoryHandlerError::Service(SetTerritoryError::PhaseConflict) => StatusCode::CONFLICT,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, Json(error.to_api_error_response())).into_response()
            }
        }
    })
    .await;
    drop(_game_update_guard);
    match result {
        Ok(response) => response,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// 占領情報編集リクエストハンドラ関数
fn handle_set_territory<U, G>(
    service: &GameService<U, G>,
    message_repository: &SqliteMessageRepository,
    request: SetTerritoryRequest,
) -> Result<SetTerritoryResponse, SetTerritoryHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(SetTerritoryHandlerError::InvalidRequest)?;

    let access_token = request.authorization.trim()[7..].trim().to_string();

    let result = service
        .set_territory(SetTerritoryCommand {
            access_token,
            game_uuid: request.game_uuid,
            code: request.code.clone(),
            power: request.power,
            season: request.season,
        })
        .map_err(SetTerritoryHandlerError::Service)?;

    let turn = result.game.current_turn();
    let game_uuid = result.game.uuid;

    if result.changed {
        match (result.old_power, result.new_power) {
            (None, Some(new_power)) => {
                if let Err(error) = message_repository.append_territory_set_message(game_uuid, &turn, result.province, new_power)
                {
                    eprintln!("failed to persist TerritorySet message (game_uuid={}): {}", game_uuid, error);
                }
            }
            (Some(old_power), Some(new_power)) => {
                if let Err(error) =
                    message_repository.append_territory_replaced_message(game_uuid, &turn, result.province, old_power, new_power)
                {
                    eprintln!(
                        "failed to persist TerritoryReplaced message (game_uuid={}): {}",
                        game_uuid, error
                    );
                }
            }
            (Some(old_power), None) => {
                if let Err(error) =
                    message_repository.append_territory_released_message(game_uuid, &turn, result.province, old_power)
                {
                    eprintln!(
                        "failed to persist TerritoryReleased message (game_uuid={}): {}",
                        game_uuid, error
                    );
                }
            }
            (None, None) => {}
        }
    }

    Ok(SetTerritoryResponse {
        game_uuid,
        code: result.province.code().to_string(),
        power: result.new_power.map(|p| p.symbol().to_string()),
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
    use crate::api::requests::UnitSpecBody;
    use crate::domain::Game;
    use crate::domain::GameStatus;
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::repositories::NewGame;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::SqliteMessageRepository;
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

    fn new_test_message_repository() -> SqliteMessageRepository {
        let path = std::env::temp_dir().join(format!("moe3-test-{}", uuid::Uuid::now_v7()));
        let base = path.to_string_lossy().to_string();
        SqliteMessageRepository::new(&format!("{}.messages.db", base))
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
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_create_game(
            &service,
            &user_repository,
            &message_repository,
            CreateGameRequest {
                authorization: "Bearer token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: Some("f".to_string()),
                keyword: None,
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
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        handle_create_game(
            &service,
            &user_repository,
            &message_repository,
            CreateGameRequest {
                authorization: "Bearer token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: None,
                keyword: None,
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
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_create_game(
            &service,
            &user_repository,
            &message_repository,
            CreateGameRequest {
                authorization: "token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date: "2026-04-19".to_string(),
                first_period_hour: 12,
                requested_power: None,
                keyword: None,
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
            keyword: None,
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
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_join_game(
            &service,
            &user_repository,
            &message_repository,
            JoinGameRequest {
                authorization: "Bearer token-2".to_string(),
                game_uuid,
                requested_power: Some("f".to_string()),
                keyword: None,
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
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_join_game(
            &service,
            &user_repository,
            &message_repository,
            JoinGameRequest {
                authorization: "token-1".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                requested_power: None,
                keyword: None,
            },
        )
        .expect_err("join should fail");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn parse_keyword_accepts_alphanumeric() {
        let result = parse_keyword(Some("Abc123"), || "err");
        assert_eq!(result, Ok(Some("Abc123".to_string())));
    }

    #[test]
    fn parse_keyword_trims_whitespace() {
        let result = parse_keyword(Some("  abc  "), || "err");
        assert_eq!(result, Ok(Some("abc".to_string())));
    }

    #[test]
    fn parse_keyword_returns_none_for_whitespace_only() {
        let result = parse_keyword(Some("   "), || "err");
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn parse_keyword_returns_none_when_absent() {
        let result = parse_keyword::<&str, _>(None, || "err");
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn parse_keyword_rejects_symbols() {
        let result = parse_keyword(Some("abc!"), || "err");
        assert_eq!(result, Err("err"));
    }

    #[test]
    fn parse_keyword_rejects_non_ascii() {
        let result = parse_keyword(Some("abcキー"), || "err");
        assert_eq!(result, Err("err"));
    }

    #[test]
    fn handle_create_game_rejects_invalid_keyword() {
        let (start_date, first_period_hour) = future_start_params();
        let user_repository = InMemoryUserRepository::new(Vec::new());
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_create_game(
            &service,
            &user_repository,
            &message_repository,
            CreateGameRequest {
                authorization: "Bearer token-1".to_string(),
                face_type: 1,
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: None,
                keyword: Some("invalid!".to_string()),
            },
        )
        .expect_err("create should fail");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_join_game_rejects_invalid_keyword() {
        let user_repository = InMemoryUserRepository::new(Vec::new());
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_join_game(
            &service,
            &user_repository,
            &message_repository,
            JoinGameRequest {
                authorization: "Bearer token-1".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                requested_power: None,
                keyword: Some("無効キー".to_string()),
            },
        )
        .expect_err("join should fail");

        assert_eq!(error.code(), "invalid_request");
    }

    // ============================================================================
    // handle_set_draw_proposal tests
    // ============================================================================

    fn sample_in_progress_game_for_handler(owner_uuid: uuid::Uuid) -> Game {
        let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let now_jst = Utc::now().with_timezone(&jst);
        let start = now_jst + chrono::Duration::hours(2);
        let regulation = crate::domain::Regulation::new(
            crate::domain::FaceType::Girls,
            crate::domain::ProgressMode::Scheduled,
            crate::domain::DurationType::Short,
            start.date_naive(),
            start.hour() as u8,
        )
        .unwrap();
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: owner_uuid,
                power: None,
                is_accepting_draw: false,
                is_owner: true,
                requested_power: None,
            }],
            phases: vec![ready, spring_main],
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        }
    }

    #[test]
    fn handle_set_draw_proposal_rejects_missing_authorization() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_draw_proposal(
            &service,
            &message_repository,
            SetDrawProposalRequest {
                authorization: "".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                draw_proposal: true,
            },
        )
        .expect_err("should fail without authorization");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_draw_proposal_rejects_non_owner() {
        let owner_uuid = uuid::Uuid::now_v7();
        let other_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            UserRecord {
                id: 1,
                uuid: owner_uuid,
                discord_user_id: "discord-owner".to_string(),
                username: "owner".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-owner".to_string(),
                last_access_at: Utc::now(),
            },
            UserRecord {
                id: 2,
                uuid: other_uuid,
                discord_user_id: "discord-other".to_string(),
                username: "other".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-other".to_string(),
                last_access_at: Utc::now(),
            },
        ]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_draw_proposal(
            &service,
            &message_repository,
            SetDrawProposalRequest {
                authorization: "Bearer token-other".to_string(),
                game_uuid,
                draw_proposal: true,
            },
        )
        .expect_err("should fail for non-owner");

        assert_eq!(error.code(), "forbidden");
    }

    #[test]
    fn handle_set_draw_proposal_returns_no_op_when_unchanged() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.players.iter_mut().find(|p| p.is_owner).unwrap().is_accepting_draw = false;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_draw_proposal(
            &service,
            &message_repository,
            SetDrawProposalRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                draw_proposal: false,
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert!(!response.draw_proposal);
    }

    #[test]
    fn handle_set_draw_proposal_sets_draw_proposal_flag() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_draw_proposal(
            &service,
            &message_repository,
            SetDrawProposalRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                draw_proposal: true,
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert!(response.draw_proposal);
    }

    #[test]
    fn handle_set_unit_rejects_missing_authorization() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                unit: None,
                location: "par".to_string(),
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail without authorization");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_unit_extracts_token_case_insensitively() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        // "bearer" 小文字でも validate() が通る (eq_ignore_ascii_case) が、
        // handler 側の token 抽出は [7..] スライスのため "token-owner" が正しく抽出される
        let result = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                unit: Some(UnitSpecBody {
                    power: "f".to_string(),
                    kind: "a".to_string(),
                }),
                location: "par".to_string(),
                season: "1901s".to_string(),
            },
        );

        // par = Paris (Inland) — army は配置可能
        assert!(result.is_ok(), "should succeed with valid token");
    }

    #[test]
    fn handle_set_unit_places_unit_and_records_message() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                unit: Some(UnitSpecBody {
                    power: "f".to_string(),
                    kind: "a".to_string(),
                }),
                location: "par".to_string(),
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        // 配置されたユニット情報がレスポンスに含まれる
        assert!(response.unit.is_some(), "unit should be present in response");
        assert_eq!(response.location, "par");
    }

    #[test]
    fn handle_set_unit_removes_unit_and_records_message() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        let paris = crate::domain::Province::from_code("par").unwrap();
        let unit = crate::domain::Unit::new_army(crate::domain::Power::France, paris);
        game.phases.last_mut().unwrap().units.push(unit);

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                unit: None,
                location: "par".to_string(),
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert!(response.unit.is_none(), "unit should be absent when removed");
        assert_eq!(response.location, "par");
    }

    #[test]
    fn handle_set_territory_rejects_missing_authorization() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                code: "par".to_string(),
                power: Some("f".to_string()),
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail without authorization");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_territory_sets_territory_and_records_message() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "par".to_string(),
                power: Some("f".to_string()),
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert_eq!(response.code, "par");
        assert_eq!(response.power.as_deref(), Some("f"));
    }

    #[test]
    fn handle_set_territory_replaces_existing_owner() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        // 事前に France が par を保有
        game.phases
            .last_mut()
            .unwrap()
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::France, "par"));
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "par".to_string(),
                power: Some("e".to_string()), // France → England に変更
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.code, "par");
        assert_eq!(response.power.as_deref(), Some("e"));
    }

    #[test]
    fn handle_set_territory_releases_territory() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.phases
            .last_mut()
            .unwrap()
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::France, "par"));
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "par".to_string(),
                power: None, // DELETE: 保有解除
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.code, "par");
        assert!(response.power.is_none(), "power should be null after release");
    }

    #[test]
    fn handle_set_territory_noop_does_not_update_repository() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.phases
            .last_mut()
            .unwrap()
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::France, "par"));
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());
        let message_repository = new_test_message_repository();

        let _response = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "par".to_string(),
                power: Some("f".to_string()), // France → France（変更なし）
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert!(
            game_repository.last_updated().is_none(),
            "repository.update should not be called on no-op"
        );
    }

    #[test]
    fn handle_set_territory_water_province_returns_not_found() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "nth".to_string(), // North Sea = 海洋プロヴィンス
                power: Some("f".to_string()),
                season: "1901s".to_string(),
            },
        )
        .expect_err("water province should be rejected");

        assert_eq!(error.code(), "not_found");
    }

    #[test]
    fn handle_set_unit_rejects_wrong_season() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                unit: None,
                location: "par".to_string(),
                season: "1901f".to_string(), // wrong season (game is 1901s)
            },
        )
        .expect_err("should fail with wrong season");

        assert_eq!(error.code(), "phase_conflict");
    }

    #[test]
    fn handle_set_territory_rejects_wrong_season() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "par".to_string(),
                power: Some("f".to_string()),
                season: "1901f".to_string(), // wrong season (game is 1901s)
            },
        )
        .expect_err("should fail with wrong season");

        assert_eq!(error.code(), "phase_conflict");
    }
}
