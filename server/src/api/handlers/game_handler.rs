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

use super::ApiErrorResponse;
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
use super::DurationType;
use super::FaceType;
use super::GameListItem;
use super::GameListItemRegulation;
use super::GameRepository;
use super::GameService;
use super::GameStatus;
use super::GameStatusFilter;
use super::GetGamesHandlerError;
use super::GetGamesQueryParams;
use super::GetGamesRequest;
use super::GetGamesResponse;
use super::JoinGameCommand;
use super::JoinGameError;
use super::JoinGameHandlerError;
use super::JoinGameRequest;
use super::JoinGameRequestBody;
use super::JoinGameRequestValidationError;
use super::JoinGameResponse;
use super::ListGamesError;
use super::ListGamesResult;
use super::Power;
use super::ProgressMode;
use super::Regulation;
use super::RepositoryError;
use super::SetDrawProposalCommand;
use super::SetDrawProposalError;
use super::SetDrawProposalHandlerError;
use super::SetDrawProposalRequest;
use super::SetDrawProposalRequestBody;
use super::SetDrawProposalRequestValidationError;
use super::SetDrawProposalResponse;
use super::SetNextUpdateAtCommand;
use super::SetNextUpdateAtError;
use super::SetNextUpdateAtHandlerError;
use super::SetNextUpdateAtRequest;
use super::SetNextUpdateAtRequestBody;
use super::SetNextUpdateAtRequestValidationError;
use super::SetNextUpdateAtResponse;
use super::SetProgressConsensusCommand;
use super::SetProgressConsensusError;
use super::SetProgressConsensusHandlerError;
use super::SetProgressConsensusRequest;
use super::SetProgressConsensusRequestBody;
use super::SetProgressConsensusRequestValidationError;
use super::SetProgressConsensusResponse;
use super::SetProgressModeCommand;
use super::SetProgressModeError;
use super::SetProgressModeHandlerError;
use super::SetProgressModeRequest;
use super::SetProgressModeRequestBody;
use super::SetProgressModeRequestValidationError;
use super::SetProgressModeResponse;
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
        progress_mode: body.progress_mode,
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
    let progress_mode = match request.progress_mode {
        Some(value) => parse_enum(value, CreateGameRequestValidationError::InvalidProgressMode)?,
        None => ProgressMode::Scheduled,
    };
    let duration_type = parse_enum(request.duration_type, CreateGameRequestValidationError::InvalidDurationType)?;
    let start_date = NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|_| invalid_request(CreateGameRequestValidationError::InvalidStartDate))?;

    Regulation::new(face_type, progress_mode, duration_type, start_date, request.first_period_hour)
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
        season: body.season,
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
                    SetDrawProposalHandlerError::Service(SetDrawProposalError::PhaseConflict) => StatusCode::CONFLICT,
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
            season: request.season.as_deref().map(str::to_lowercase),
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
            season: request.season.to_lowercase(),
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
        kind: u.kind_code().to_string(),
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
            season: request.season.to_lowercase(),
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

///
/// 進行モード変更リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_admin_games_progress_mode<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SetProgressModeRequestBody>,
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
                    SetProgressModeHandlerError::InvalidRequest(SetProgressModeRequestValidationError::InvalidGameUuid)
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

    let request = SetProgressModeRequest {
        authorization,
        game_uuid,
        season: body.season,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        match handle_set_progress_mode(
            &state_clone.game_service,
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(error) => {
                let status = match &error {
                    SetProgressModeHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                    SetProgressModeHandlerError::Service(SetProgressModeError::Unauthorized) => StatusCode::UNAUTHORIZED,
                    SetProgressModeHandlerError::Service(SetProgressModeError::NotFound) => StatusCode::NOT_FOUND,
                    SetProgressModeHandlerError::Service(SetProgressModeError::Forbidden(_)) => StatusCode::FORBIDDEN,
                    SetProgressModeHandlerError::Service(SetProgressModeError::PhaseConflict) => StatusCode::CONFLICT,
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

/// 進行モード変更リクエストハンドラ関数
fn handle_set_progress_mode<U, G>(
    service: &GameService<U, G>,
    message_repository: &SqliteMessageRepository,
    request: SetProgressModeRequest,
) -> Result<SetProgressModeResponse, SetProgressModeHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(SetProgressModeHandlerError::InvalidRequest)?;

    let access_token = request.authorization.trim()[7..].trim().to_string();

    let result = service
        .set_progress_mode(SetProgressModeCommand {
            access_token,
            game_uuid: request.game_uuid,
            season: request.season.to_lowercase(),
        })
        .map_err(SetProgressModeHandlerError::Service)?;

    if result.changed {
        let turn = result.game.current_turn();
        if let Err(error) = message_repository.append_progress_mode_changed_message(result.game.uuid, &turn) {
            eprintln!(
                "failed to persist ProgressModeChanged message (game_uuid={}): {}",
                result.game.uuid, error
            );
        }
    }

    Ok(SetProgressModeResponse {
        game_uuid: result.game.uuid,
        progress_mode: "consensus".to_string(),
    })
}

///
/// 即時進行合意設定リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_games_progress_consensus<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SetProgressConsensusRequestBody>,
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
                    SetProgressConsensusHandlerError::InvalidRequest(SetProgressConsensusRequestValidationError::InvalidGameUuid)
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

    let request = SetProgressConsensusRequest {
        authorization,
        game_uuid,
        agreed: body.agreed,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        match handle_set_progress_consensus(
            &state_clone.game_service,
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(error) => {
                let status = match &error {
                    SetProgressConsensusHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                    SetProgressConsensusHandlerError::Service(SetProgressConsensusError::Unauthorized) => {
                        StatusCode::UNAUTHORIZED
                    }
                    SetProgressConsensusHandlerError::Service(SetProgressConsensusError::NotFound) => StatusCode::NOT_FOUND,
                    SetProgressConsensusHandlerError::Service(SetProgressConsensusError::Forbidden(_)) => StatusCode::FORBIDDEN,
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

/// 即時進行合意設定リクエストハンドラ関数
fn handle_set_progress_consensus<U, G>(
    service: &GameService<U, G>,
    message_repository: &SqliteMessageRepository,
    request: SetProgressConsensusRequest,
) -> Result<SetProgressConsensusResponse, SetProgressConsensusHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(SetProgressConsensusHandlerError::InvalidRequest)?;

    let access_token = request.authorization.trim()[7..].trim().to_string();

    let result = service
        .set_progress_consensus(SetProgressConsensusCommand {
            access_token,
            game_uuid: request.game_uuid,
            agreed: request.agreed,
        })
        .map_err(SetProgressConsensusHandlerError::Service)?;

    if result.changed {
        let turn = result.game.current_turn();
        let append_result = if result.agreed {
            message_repository.append_progress_consented_message(result.game.uuid, &turn, result.actor_power)
        } else {
            message_repository.append_progress_consensus_rescinded_message(result.game.uuid, &turn, result.actor_power)
        };
        if let Err(error) = append_result {
            eprintln!(
                "failed to persist progress consensus message (game_uuid={}): {}",
                result.game.uuid, error
            );
        }
    }

    if result.reached_consensus_in_main_phase {
        let turn = result.game.current_turn();
        if let Err(error) = message_repository.append_progress_consensus_reached_message(result.game.uuid, &turn) {
            eprintln!(
                "failed to persist ProgressConsensusReached message (game_uuid={}): {}",
                result.game.uuid, error
            );
        }
    }

    Ok(SetProgressConsensusResponse {
        game_uuid: result.game.uuid,
        agreed: request.agreed,
    })
}

///
/// 次回更新時刻変更リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_admin_games_next_update_at<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<SetNextUpdateAtRequestBody>,
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
                    SetNextUpdateAtHandlerError::InvalidRequest(SetNextUpdateAtRequestValidationError::InvalidGameUuid)
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

    let request = SetNextUpdateAtRequest {
        authorization,
        game_uuid,
        next_update_at: body.next_update_at,
        season: body.season,
    };

    let state_clone = state.clone();
    let request_clone = request.clone();

    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        match handle_set_next_update_at(
            &state_clone.game_service,
            state_clone.message_repository.as_ref(),
            request_clone,
        ) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(error) => {
                let status = match &error {
                    SetNextUpdateAtHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                    SetNextUpdateAtHandlerError::Service(SetNextUpdateAtError::Unauthorized) => StatusCode::UNAUTHORIZED,
                    SetNextUpdateAtHandlerError::Service(SetNextUpdateAtError::NotFound) => StatusCode::NOT_FOUND,
                    SetNextUpdateAtHandlerError::Service(SetNextUpdateAtError::Forbidden(_)) => StatusCode::FORBIDDEN,
                    SetNextUpdateAtHandlerError::Service(SetNextUpdateAtError::InvalidRequest(_)) => StatusCode::BAD_REQUEST,
                    SetNextUpdateAtHandlerError::Service(SetNextUpdateAtError::PhaseConflict) => StatusCode::CONFLICT,
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

/// 次回更新時刻変更リクエストハンドラ関数
fn handle_set_next_update_at<U, G>(
    service: &GameService<U, G>,
    message_repository: &SqliteMessageRepository,
    request: SetNextUpdateAtRequest,
) -> Result<SetNextUpdateAtResponse, SetNextUpdateAtHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(SetNextUpdateAtHandlerError::InvalidRequest)?;

    let authorization = request.authorization.trim();
    let access_token = authorization[7..].trim().to_string();

    let result = service
        .set_next_update_at(SetNextUpdateAtCommand {
            access_token,
            game_uuid: request.game_uuid,
            next_update_at: request.next_update_at,
            season: request.season,
        })
        .map_err(SetNextUpdateAtHandlerError::Service)?;

    if result.changed {
        let turn = result.game.current_turn();
        if let Err(error) =
            message_repository.append_next_update_at_changed_message(result.game.uuid, &turn, &result.next_update_at_jst)
        {
            eprintln!(
                "failed to persist NextUpdateAtChanged message (game_uuid={}): {}",
                result.game.uuid, error
            );
        }
    }

    Ok(SetNextUpdateAtResponse {
        game_uuid: result.game.uuid,
        next_update_at: result.next_update_at_jst,
    })
}

///
/// 卓一覧取得リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_games<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    headers: HeaderMap,
    Query(params): Query<GetGamesQueryParams>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let request = GetGamesRequest {
        authorization,
        status: params.status,
        user: params.user,
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(20),
    };

    let game_service = std::sync::Arc::clone(&state.game_service);
    let result = tokio::task::spawn_blocking(move || handle_get_games(&game_service, request)).await;

    match result {
        Ok(Ok(response)) => (StatusCode::OK, axum::Json(response)).into_response(),
        Ok(Err(error)) => {
            let status = match &error {
                GetGamesHandlerError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
                GetGamesHandlerError::Service(ListGamesError::Unauthorized) => StatusCode::UNAUTHORIZED,
                GetGamesHandlerError::Service(ListGamesError::Forbidden(_)) => StatusCode::FORBIDDEN,
                GetGamesHandlerError::Service(ListGamesError::Repository(_)) => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(error.to_api_error_response())).into_response()
        }
        Err(join_err) => {
            eprintln!("get_games task failed: {}", join_err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ApiErrorResponse {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 卓一覧取得の純粋関数（テスト可能）
pub(crate) fn handle_get_games<U, G>(
    service: &GameService<U, G>,
    request: GetGamesRequest,
) -> Result<GetGamesResponse, GetGamesHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    request.validate().map_err(GetGamesHandlerError::InvalidRequest)?;

    let page = request.page;
    let per_page = request.per_page;

    let filter = match request.status.as_deref() {
        Some("active") => Some(GameStatusFilter::Active),
        Some("closed") => Some(GameStatusFilter::Closed),
        Some("aborted") => Some(GameStatusFilter::Aborted),
        _ => None,
    };

    let result: ListGamesResult = if let Some(ref discord_user_id) = request.user {
        service
            .list_games_by_user(request.access_token(), discord_user_id, filter, page, per_page)
            .map_err(GetGamesHandlerError::Service)?
    } else {
        service
            .list_games_by_status(filter.unwrap_or(GameStatusFilter::Active), page, per_page)
            .map_err(GetGamesHandlerError::Service)?
    };

    let viewer_user_uuid = service
        .find_viewer_user_uuid(optional_bearer_token(request.authorization.as_deref()))
        .map_err(|error| GetGamesHandlerError::Service(ListGamesError::Repository(error)))?;

    let jst = chrono::FixedOffset::east_opt(9 * 60 * 60).expect("JST offset");

    let games = result
        .games
        .iter()
        .map(|game| {
            let next_update_at = game.next_update_at.map(|naive_utc| {
                let jst_dt =
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_utc, chrono::Utc).with_timezone(&jst);
                jst_dt.format("%Y-%m-%d %H:%M").to_string()
            });

            let status = match game.status {
                GameStatus::Preparing => "preparing",
                GameStatus::Ready => "ready",
                GameStatus::InProgress => "in_progress",
                GameStatus::Solo => "solo",
                GameStatus::Draw => "draw",
                GameStatus::Aborted => "aborted",
                GameStatus::Closed => "closed",
            };

            let face_type = match game.regulation.face_type {
                FaceType::Girls => "girls",
                FaceType::Flags => "flags",
            };

            let progress_mode = match game.regulation.progress_mode {
                super::ProgressMode::Scheduled => "scheduled",
                super::ProgressMode::Consensus => "consensus",
            };

            let duration_type = match game.regulation.duration_type {
                DurationType::Short => "short",
                DurationType::Normal => "normal",
            };

            GameListItem {
                game_uuid: game.uuid,
                game_number: game.game_number,
                status: status.to_string(),
                season: game.season_label.clone(),
                next_update_at,
                regulation: GameListItemRegulation {
                    face_type: face_type.to_string(),
                    progress_mode: progress_mode.to_string(),
                    duration_type: duration_type.to_string(),
                },
                player_count: game.player_count,
                is_owner: viewer_user_uuid.is_some() && game.owner_user_uuid == viewer_user_uuid,
            }
        })
        .collect();

    Ok(GetGamesResponse {
        games,
        total: result.total,
        page,
        per_page,
    })
}

///
/// 卓情報取得リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_game<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    headers: HeaderMap,
    Path(game_uuid_str): Path<String>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let game_uuid = match Uuid::parse_str(&game_uuid_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiErrorResponse {
                    code: "invalid_request",
                    message: "game_uuid is invalid".to_string(),
                }),
            )
                .into_response();
        }
    };

    let game_service = std::sync::Arc::clone(&state.game_service);
    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let result = tokio::task::spawn_blocking(move || handle_get_game(&game_service, game_uuid, authorization.as_deref())).await;

    match result {
        Ok(Ok(response)) => (StatusCode::OK, axum::Json(response)).into_response(),
        Ok(Err(error)) => {
            let status = match &error {
                super::GetGameHandlerError::Service(super::GetGameError::NotFound) => StatusCode::NOT_FOUND,
                super::GetGameHandlerError::Service(super::GetGameError::Repository(_)) => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(error.to_api_error_response())).into_response()
        }
        Err(join_err) => {
            eprintln!("get_game task failed: {}", join_err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ApiErrorResponse {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 卓情報取得の純粋関数（テスト可能）
pub(crate) fn handle_get_game<U, G>(
    service: &GameService<U, G>,
    game_uuid: Uuid,
    authorization: Option<&str>,
) -> Result<super::GetGameResponse, super::GetGameHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    let game = service.get_game(game_uuid).map_err(super::GetGameHandlerError::Service)?;

    let viewer_user_uuid = service
        .find_viewer_user_uuid(optional_bearer_token(authorization))
        .map_err(|error| super::GetGameHandlerError::Service(super::GetGameError::Repository(error)))?;
    let is_owner = viewer_user_uuid.is_some_and(|viewer| {
        game.players
            .iter()
            .any(|player| player.is_owner && player.user_uuid == viewer)
    });
    let is_joined = viewer_user_uuid.is_some_and(|viewer| game.players.iter().any(|player| player.user_uuid == viewer));

    let players =
        game.players
            .iter()
            .map(|player| {
                let user = service
                    .find_user_by_uuid(player.user_uuid)
                    .map_err(|error| super::GetGameHandlerError::Service(super::GetGameError::Repository(error)))?
                    .ok_or_else(|| {
                        super::GetGameHandlerError::Service(super::GetGameError::Repository(RepositoryError::Unavailable(
                            format!("user not found for player: {}", player.user_uuid),
                        )))
                    })?;
                // 卓主の希望国は全員に公開し、それ以外の希望国は本人にのみ公開する
                let requested_power = if player.is_owner || viewer_user_uuid == Some(player.user_uuid) {
                    player.requested_power.map(|power| power.symbol().to_string())
                } else {
                    None
                };
                Ok(super::GetGameResponsePlayer {
                    user: super::GetGameResponseUser {
                        discord_user_id: user.discord_user_id,
                        username: user.username,
                        global_name: user.global_name,
                        avatar_url: user.avatar_url,
                    },
                    is_owner: player.is_owner,
                    requested_power,
                    power: player.power.map(|power| power.symbol().to_string()),
                    is_eliminated: player.power.is_some_and(|power| is_power_eliminated(power, &game)),
                })
            })
            .collect::<Result<Vec<_>, super::GetGameHandlerError>>()?;

    let status = match game.status {
        GameStatus::Preparing => "preparing",
        GameStatus::Ready => "ready",
        GameStatus::InProgress => "in_progress",
        GameStatus::Solo => "solo",
        GameStatus::Draw => "draw",
        GameStatus::Aborted => "aborted",
        GameStatus::Closed => "closed",
    };

    let face_type = match game.regulation.face_type {
        FaceType::Girls => "girls",
        FaceType::Flags => "flags",
    };

    let progress_mode = match game.regulation.progress_mode {
        ProgressMode::Scheduled => "scheduled",
        ProgressMode::Consensus => "consensus",
    };

    let duration_type = match game.regulation.duration_type {
        DurationType::Short => "short",
        DurationType::Normal => "normal",
    };

    let seasons = build_seasons(&game);
    let phase_kind = phase_kind_str(&game);

    // 和平終了フラグ（講和の宣言）は卓主の is_accepting_draw。卓主がいない・フェイズがない卓は false
    let draw_proposal = !game.phases.is_empty() && game.players.iter().any(|player| player.is_owner && player.is_accepting_draw);

    let jst = chrono::FixedOffset::east_opt(9 * 60 * 60).expect("JST offset");
    let next_update_at = game.next_update_at.map(|naive_utc| {
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_utc, chrono::Utc)
            .with_timezone(&jst)
            .format("%Y-%m-%d %H:%M")
            .to_string()
    });

    Ok(super::GetGameResponse {
        game: super::GetGameResponseGame {
            game_uuid: game.uuid,
            game_number: game.game_number,
            status: status.to_string(),
            seasons,
            phase_kind: phase_kind.to_string(),
            next_update_at,
            draw_proposal,
            is_private: game.keyword.is_some(),
            is_owner,
            regulation: GameListItemRegulation {
                face_type: face_type.to_string(),
                progress_mode: progress_mode.to_string(),
                duration_type: duration_type.to_string(),
            },
            start_at: format!(
                "{} {:02}:00",
                game.regulation.start_date.format("%Y-%m-%d"),
                game.regulation.first_period_hour
            ),
            players,
            is_joined,
        },
    })
}

/// 認証任意の API 向けに Authorization ヘッダから Bearer トークンを取り出す
///
/// ヘッダ欠落・形式不正の場合は空文字を返す（＝未ログイン扱い）。
pub(super) fn optional_bearer_token(authorization: Option<&str>) -> &str {
    let auth = authorization.unwrap_or("").trim();
    match auth.get(..7) {
        Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
        _ => "",
    }
}

/// フェイズ履歴からシーズン配列を生成する
fn build_seasons(game: &crate::Game) -> Vec<String> {
    let mut seasons: Vec<String> = Vec::new();
    for phase in &game.phases {
        let turn = turn_of_phase(phase);
        if seasons.last() != Some(&turn) {
            seasons.push(turn);
        }
    }
    seasons
}

/// 現在のフェイズ種別文字列を返す
pub(crate) fn phase_kind_str(game: &crate::Game) -> &'static str {
    match game.phases.last().map(|p| &p.kind) {
        Some(crate::PhaseKind::Ready(_)) | None => "ready",
        Some(crate::PhaseKind::SpringMain(_)) | Some(crate::PhaseKind::FallMain(_)) => "main",
        Some(crate::PhaseKind::SpringRetreat(_)) | Some(crate::PhaseKind::FallRetreat(_)) => "retreat",
        Some(crate::PhaseKind::Adjustment(_)) => "adjustment",
        Some(crate::PhaseKind::Debrief(_)) => "debrief",
    }
}

///
/// 外交履歴取得リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_games_logs<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    headers: HeaderMap,
    Path((game_uuid_str, season)): Path<(String, String)>,
    Query(query): Query<super::GetGameLogsQueryParams>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let game_uuid = match Uuid::parse_str(&game_uuid_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiErrorResponse {
                    code: "invalid_request",
                    message: "game_uuid is invalid".to_string(),
                }),
            )
                .into_response();
        }
    };

    let after_uuid = match query.after.as_deref().map(Uuid::parse_str) {
        Some(Ok(uuid)) => Some(uuid),
        Some(Err(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiErrorResponse {
                    code: "invalid_request",
                    message: "after is invalid".to_string(),
                }),
            )
                .into_response();
        }
        None => None,
    };

    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let game_service = std::sync::Arc::clone(&state.game_service);
    let user_repository = std::sync::Arc::clone(&state.user_repository);
    let message_repository = std::sync::Arc::clone(&state.message_repository);

    let result = tokio::task::spawn_blocking(move || {
        handle_get_games_logs(
            &game_service,
            user_repository.as_ref(),
            message_repository.as_ref(),
            game_uuid,
            &season,
            after_uuid,
            authorization.as_deref(),
        )
    })
    .await;

    match result {
        Ok(Ok(response)) => (StatusCode::OK, axum::Json(response)).into_response(),
        Ok(Err(error)) => {
            let status = match &error {
                super::GetGameLogsHandlerError::GameNotFound => StatusCode::NOT_FOUND,
                super::GetGameLogsHandlerError::SeasonNotFound => StatusCode::NOT_FOUND,
                super::GetGameLogsHandlerError::Repository(_) => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(error.to_api_error_response())).into_response()
        }
        Err(join_err) => {
            eprintln!("get_games_logs task failed: {}", join_err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ApiErrorResponse {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 外交履歴取得の純粋関数（テスト可能）
pub(crate) fn handle_get_games_logs<U, G>(
    service: &GameService<U, G>,
    user_repository: &U,
    message_repository: &SqliteMessageRepository,
    game_uuid: Uuid,
    season: &str,
    after_uuid: Option<Uuid>,
    authorization: Option<&str>,
) -> Result<super::GetGameLogsResponse, super::GetGameLogsHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    let game = service.get_game(game_uuid).map_err(|e| match e {
        super::GetGameError::NotFound => super::GetGameLogsHandlerError::GameNotFound,
        super::GetGameError::Repository(r) => super::GetGameLogsHandlerError::Repository(r),
    })?;

    // 指定シーズンがゲームに存在するか確認
    let valid_seasons = build_seasons(&game);
    if !valid_seasons.contains(&season.to_string()) {
        return Err(super::GetGameLogsHandlerError::SeasonNotFound);
    }

    // メッセージ取得
    let records = message_repository
        .find_by_game_and_season(game_uuid, season, after_uuid)
        .map_err(super::GetGameLogsHandlerError::Repository)?;

    // 認証トークンからリクエストユーザーの担当 Power を取得
    let viewer_power = if let Some(auth) = authorization {
        let token = auth.trim().trim_start_matches("Bearer ").trim();
        if token.is_empty() {
            None
        } else {
            let user = user_repository
                .find_by_access_token(token)
                .map_err(super::GetGameLogsHandlerError::Repository)?;
            user.and_then(|u| game.players.iter().find(|p| p.user_uuid == u.uuid).and_then(|p| p.power))
        }
    } else {
        None
    };

    let mut messages = apply_log_access_control(records, &game, viewer_power);
    attach_sender_users(&mut messages, user_repository).map_err(super::GetGameLogsHandlerError::Repository)?;
    Ok(super::GetGameLogsResponse { messages })
}

/// アクセス制御を適用してメッセージ一覧を返す
pub(super) fn apply_log_access_control(
    records: Vec<crate::MessageRecord>,
    game: &crate::Game,
    viewer_power: Option<Power>,
) -> Vec<super::GameLogMessage> {
    if game.status != GameStatus::InProgress {
        return records.into_iter().map(to_game_log_message_full).collect();
    }

    records
        .into_iter()
        .filter_map(|r| match r.kind.as_str() {
            // 亡国会話は公式声明と同じく全員に公開する
            "public" | "ghost" | "system" => Some(to_game_log_message_full(r)),
            "personal" => {
                if viewer_power.is_some() && r.sender_power == viewer_power {
                    Some(to_game_log_message_full(r))
                } else {
                    None
                }
            }
            "confidential" => {
                if let Some(vp) = viewer_power {
                    if r.sender_power == Some(vp) || r.recipients.contains(&vp) {
                        Some(to_game_log_message_full(r))
                    } else {
                        Some(to_game_log_message_masked(r))
                    }
                } else {
                    Some(to_game_log_message_masked(r))
                }
            }
            _ => None,
        })
        .collect()
}

/// 担当 Power が滅亡しているかを判定する
///
/// 最新フェイズにユニットと補給都市がどちらも存在しない場合に滅亡とみなす。
/// 補給都市でない陸地の占領は判定に含めない。開戦前夜・感想戦では滅亡とみなさない。
pub(super) fn is_power_eliminated(power: Power, game: &crate::Game) -> bool {
    game.is_power_eliminated(power)
}

/// 送信者種別が "user" のメッセージについて、送信ユーザーの UUID を返す
///
/// 国の発言・管理人発言では送信ユーザーを公開しないため `None` を返す。
fn sender_user_uuid_of(r: &crate::MessageRecord) -> Option<Uuid> {
    match r.sender_kind {
        crate::MessageSenderKind::User => r.sender_user_uuid,
        _ => None,
    }
}

///
/// メッセージレスポンスの送信ユーザー情報を解決する
///
/// 同じユーザーの発言が続く場合に備えて、検索結果をキャッシュする。
/// ユーザーが見つからない場合は `sender_user` を `None` のままにする。
///
pub(super) fn attach_sender_users<U>(messages: &mut [super::GameLogMessage], user_repository: &U) -> Result<(), RepositoryError>
where
    U: UserRepository,
{
    let mut cache: std::collections::HashMap<Uuid, Option<super::GameLogSenderUser>> = std::collections::HashMap::new();
    for message in messages.iter_mut() {
        let Some(user_uuid) = message.sender_user_uuid else {
            continue;
        };
        let sender_user = match cache.entry(user_uuid) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.get().clone(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let sender_user = user_repository.find_by_uuid(user_uuid)?.map(|user| super::GameLogSenderUser {
                    username: user.username,
                    global_name: user.global_name,
                    avatar_url: user.avatar_url,
                });
                entry.insert(sender_user).clone()
            }
        };
        message.sender_user = sender_user;
    }
    Ok(())
}

/// フルコンテンツのメッセージレスポンスを生成する
pub(super) fn to_game_log_message_full(r: crate::MessageRecord) -> super::GameLogMessage {
    let is_confidential = r.kind == "confidential";
    let recipients = if is_confidential {
        Some(r.recipients.iter().map(|p| p.symbol().to_string()).collect())
    } else {
        None
    };
    let sender_user_uuid = sender_user_uuid_of(&r);
    super::GameLogMessage {
        message_uuid: r.message_uuid,
        sender: r.sender_power.map(|p| p.symbol().to_string()),
        sender_kind: r.sender_kind.as_str().to_string(),
        sender_user: None,
        sender_user_uuid,
        turn: r.turn,
        context: Some(r.context),
        kind: r.kind,
        recipients,
        created_at: utc_rfc3339_to_jst_string(&r.created_at),
        notice: r.notice.map(|notice| super::GameLogNotice {
            notice_type: notice.notice_type,
            params: notice.params,
        }),
    }
}

/// マスクされたメッセージレスポンスを生成する（kind="masked"、context=null）
fn to_game_log_message_masked(r: crate::MessageRecord) -> super::GameLogMessage {
    let recipients = Some(r.recipients.iter().map(|p| p.symbol().to_string()).collect());
    let sender_user_uuid = sender_user_uuid_of(&r);
    super::GameLogMessage {
        message_uuid: r.message_uuid,
        sender: r.sender_power.map(|p| p.symbol().to_string()),
        sender_kind: r.sender_kind.as_str().to_string(),
        sender_user: None,
        sender_user_uuid,
        turn: r.turn,
        context: None,
        kind: "masked".to_string(),
        recipients,
        created_at: utc_rfc3339_to_jst_string(&r.created_at),
        notice: None,
    }
}

/// UTC RFC3339 文字列を JST "YYYY-MM-DD HH:mm" 形式に変換する
fn utc_rfc3339_to_jst_string(utc_rfc3339: &str) -> String {
    let jst = chrono::FixedOffset::east_opt(9 * 60 * 60).expect("JST offset");
    chrono::DateTime::parse_from_rfc3339(utc_rfc3339)
        .map(|dt| dt.with_timezone(&jst).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| utc_rfc3339.to_string())
}

///
/// 盤面取得リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_games_board<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    Query(query): Query<super::GetGameBoardQueryParams>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let game_uuid = match Uuid::parse_str(&game_uuid_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiErrorResponse {
                    code: "invalid_request",
                    message: "game_uuid is invalid".to_string(),
                }),
            )
                .into_response();
        }
    };

    let game_service = std::sync::Arc::clone(&state.game_service);
    let result =
        tokio::task::spawn_blocking(move || handle_get_games_board(&game_service, game_uuid, query.season.as_deref())).await;

    match result {
        Ok(Ok(response)) => (StatusCode::OK, axum::Json(response)).into_response(),
        Ok(Err(error)) => {
            let status = match &error {
                super::GetGameResultHandlerError::GameNotFound | super::GetGameResultHandlerError::SeasonNotFound => {
                    StatusCode::NOT_FOUND
                }
                super::GetGameResultHandlerError::Repository(_) => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(error.to_api_error_response())).into_response()
        }
        Err(join_err) => {
            eprintln!("get_games_board task failed: {}", join_err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ApiErrorResponse {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 盤面取得の純粋関数（テスト可能）
///
/// シーズン指定がなければ最新フェイズ、指定があればそのシーズンの最後のフェイズについて、
/// ユニット配置・占領情報・国ごとの状況を返す。命令は含めない。
pub(crate) fn handle_get_games_board<U, G>(
    service: &GameService<U, G>,
    game_uuid: Uuid,
    season: Option<&str>,
) -> Result<super::GetGameBoardResponse, super::GetGameResultHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    let game = service.get_game(game_uuid).map_err(|e| match e {
        super::GetGameError::NotFound => super::GetGameResultHandlerError::GameNotFound,
        super::GetGameError::Repository(r) => super::GetGameResultHandlerError::Repository(r),
    })?;

    let season = season.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());
    let phase = match &season {
        None => game.phases.last(),
        Some(season) => {
            let phase = game.phases.iter().rev().find(|p| &turn_of_phase(p) == season);
            if phase.is_none() {
                return Err(super::GetGameResultHandlerError::SeasonNotFound);
            }
            phase
        }
    };

    let Some(phase) = phase else {
        return Ok(super::GetGameBoardResponse {
            season: game.current_turn(),
            phase_kind: phase_kind_str(&game).to_string(),
            units: Vec::new(),
            territories: Vec::new(),
            powers: Vec::new(),
        });
    };
    let is_latest = game.phases.last().map(|p| p.index) == Some(phase.index);

    let powers = service
        .power_statuses(&game, phase, is_latest)
        .map_err(super::GetGameResultHandlerError::Repository)?
        .into_iter()
        .map(|s| super::GameBoardPowerResponse {
            power: s.power.symbol().to_string(),
            supply_centers: s.supply_centers,
            armies: s.armies,
            fleets: s.fleets,
            needs_action: s.needs_action,
            progress_consented: s.progress_consented,
            is_idle: s.is_idle,
            absent_minutes: s.absent_minutes,
            is_eliminated: s.is_eliminated,
        })
        .collect();

    Ok(super::GetGameBoardResponse {
        season: turn_of_phase(phase),
        phase_kind: crate::services::phase_kind_of(phase).to_string(),
        units: phase
            .units
            .iter()
            .map(|u| super::GameBoardUnitResponse {
                power: u.power.symbol().to_string(),
                kind: u.kind_code().to_string(),
                location: u.location.code_with_coast().to_string(),
                dislodged: u.dislodged,
                dislodged_from: u.dislodged_from.map(|p| p.code_with_coast().to_string()),
            })
            .collect(),
        territories: phase
            .territories
            .iter()
            .map(|t| super::GameBoardTerritoryResponse {
                power: t.power.symbol().to_string(),
                code: t.code().to_string(),
            })
            .collect(),
        powers,
    })
}

/// フェイズのシーズン文字列を返す（例: ready, 1901s, 1901f, debrief）
fn turn_of_phase(phase: &crate::Phase) -> String {
    match &phase.kind {
        crate::PhaseKind::Ready(_) => "ready".to_string(),
        crate::PhaseKind::SpringMain(_) | crate::PhaseKind::SpringRetreat(_) => format!("{}s", phase.year),
        crate::PhaseKind::FallMain(_) | crate::PhaseKind::FallRetreat(_) | crate::PhaseKind::Adjustment(_) => {
            format!("{}f", phase.year)
        }
        crate::PhaseKind::Debrief(_) => "debrief".to_string(),
    }
}

///
/// 命令解決履歴取得リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_games_result<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path((game_uuid_str, season)): Path<(String, String)>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let game_uuid = match Uuid::parse_str(&game_uuid_str) {
        Ok(uuid) => uuid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ApiErrorResponse {
                    code: "invalid_request",
                    message: "game_uuid is invalid".to_string(),
                }),
            )
                .into_response();
        }
    };

    let game_service = std::sync::Arc::clone(&state.game_service);
    let result = tokio::task::spawn_blocking(move || handle_get_games_result(&game_service, game_uuid, &season)).await;

    match result {
        Ok(Ok(response)) => (StatusCode::OK, axum::Json(response)).into_response(),
        Ok(Err(error)) => {
            let status = match &error {
                super::GetGameResultHandlerError::GameNotFound | super::GetGameResultHandlerError::SeasonNotFound => {
                    StatusCode::NOT_FOUND
                }
                super::GetGameResultHandlerError::Repository(_) => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, axum::Json(error.to_api_error_response())).into_response()
        }
        Err(join_err) => {
            eprintln!("get_games_result task failed: {}", join_err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ApiErrorResponse {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// 命令解決履歴取得の純粋関数（テスト可能）
pub(crate) fn handle_get_games_result<U, G>(
    service: &GameService<U, G>,
    game_uuid: Uuid,
    season: &str,
) -> Result<super::GetGameResultResponse, super::GetGameResultHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    let game = service.get_game(game_uuid).map_err(|e| match e {
        super::GetGameError::NotFound => super::GetGameResultHandlerError::GameNotFound,
        super::GetGameError::Repository(r) => super::GetGameResultHandlerError::Repository(r),
    })?;

    if season == "ready" || season == "debrief" {
        return Err(super::GetGameResultHandlerError::SeasonNotFound);
    }

    let valid_seasons = build_seasons(&game);
    if !valid_seasons.contains(&season.to_string()) {
        return Err(super::GetGameResultHandlerError::SeasonNotFound);
    }

    let suffix = season.chars().last().unwrap_or(' ');
    let is_spring = suffix == 's' || suffix == 'S';
    let year: i32 = season[..season.len() - 1]
        .parse()
        .map_err(|_| super::GetGameResultHandlerError::SeasonNotFound)?;

    let last_phase_index = game.phases.last().map(|p| p.index);

    let season_phases: Vec<&crate::Phase> = game
        .phases
        .iter()
        .filter(|p| {
            p.year == year
                && match &p.kind {
                    crate::PhaseKind::SpringMain(_) | crate::PhaseKind::SpringRetreat(_) => is_spring,
                    crate::PhaseKind::FallMain(_) | crate::PhaseKind::FallRetreat(_) | crate::PhaseKind::Adjustment(_) => {
                        !is_spring
                    }
                    _ => false,
                }
        })
        .collect();

    let phase_status = |phase: &crate::Phase| -> &'static str {
        if Some(phase.index) == last_phase_index {
            "open"
        } else {
            "closed"
        }
    };

    let build_orders = |phase: &crate::Phase| -> Vec<super::GameResultOrderResponse> {
        // 進行中のフェイズの命令は公開しない（自分の命令は命令取得 API で取る）
        if Some(phase.index) == last_phase_index {
            return Vec::new();
        }
        phase
            .orders
            .iter()
            .filter(|o| !o.is_assumed())
            .map(|o| {
                let (order_kind, dest, target_location, target_dest, via_convoy) = match &o.kind {
                    crate::OrderKind::Hold(_) => ("hold", None, None, None, None),
                    crate::OrderKind::Move(m) => (
                        "move",
                        Some(m.dest.code_with_coast().to_string()),
                        None,
                        None,
                        Some(m.via_convoy),
                    ),
                    crate::OrderKind::Support(s) => (
                        "support",
                        None,
                        Some(s.target_unit.location.code_with_coast().to_string()),
                        s.target_dest.map(|p| p.code_with_coast().to_string()),
                        None,
                    ),
                    crate::OrderKind::Convoy(c) => (
                        "convoy",
                        None,
                        Some(c.target_unit.location.code_with_coast().to_string()),
                        Some(c.target_dest.code_with_coast().to_string()),
                        None,
                    ),
                    crate::OrderKind::Retreat(r) => ("retreat", Some(r.dest.code_with_coast().to_string()), None, None, None),
                    crate::OrderKind::Build(_) => ("build", None, None, None, None),
                    crate::OrderKind::Disband(_) => ("disband", None, None, None, None),
                };
                let status = match o.status {
                    crate::OrderStatus::Unresolved => "unresolved",
                    crate::OrderStatus::Failure => "failure",
                    crate::OrderStatus::Success => "success",
                    crate::OrderStatus::Dislodged => "dislodged",
                    crate::OrderStatus::Cut => "cut",
                    crate::OrderStatus::Valid => "valid",
                    crate::OrderStatus::Invalid => "invalid",
                    crate::OrderStatus::Unreachable => "unreachable",
                };
                super::GameResultOrderResponse {
                    power: o.power.symbol().to_string(),
                    unit_kind: o.unit.kind_code().to_string(),
                    location: o.unit.location.code_with_coast().to_string(),
                    order_kind: order_kind.to_string(),
                    status: status.to_string(),
                    dest,
                    target_location,
                    target_dest,
                    via_convoy,
                }
            })
            .collect()
    };

    let main_phase = season_phases
        .iter()
        .find(|p| matches!(&p.kind, crate::PhaseKind::SpringMain(_) | crate::PhaseKind::FallMain(_)))
        .copied()
        .ok_or(super::GetGameResultHandlerError::SeasonNotFound)?;

    let retreat_phase = season_phases
        .iter()
        .find(|p| matches!(&p.kind, crate::PhaseKind::SpringRetreat(_) | crate::PhaseKind::FallRetreat(_)))
        .copied();

    let adjustment_phase = if !is_spring {
        season_phases
            .iter()
            .find(|p| matches!(&p.kind, crate::PhaseKind::Adjustment(_)))
            .copied()
    } else {
        None
    };

    let units = season_phases
        .last()
        .map(|p| {
            p.units
                .iter()
                .map(|u| super::GameResultUnitResponse {
                    power: u.power.symbol().to_string(),
                    kind: u.kind_code().to_string(),
                    location: u.location.code_with_coast().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(super::GetGameResultResponse {
        units,
        main_phase: super::GameResultPhaseResponse {
            status: phase_status(main_phase).to_string(),
            orders: build_orders(main_phase),
        },
        retreat_phase: retreat_phase.map(|p| super::GameResultPhaseResponse {
            status: phase_status(p).to_string(),
            orders: build_orders(p),
        }),
        adjustment_phase: adjustment_phase.map(|p| super::GameResultPhaseResponse {
            status: phase_status(p).to_string(),
            orders: build_orders(p),
        }),
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
    use crate::GameSummary;
    use crate::api::GameBoardTerritoryResponse;
    use crate::api::GameBoardUnitResponse;
    use crate::api::GetGamesRequestValidationError;
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
    use chrono::TimeZone;

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

        fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self
                .rows_by_token
                .borrow()
                .values()
                .find(|row| row.username == username)
                .cloned())
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

        fn update_access_token(
            &self,
            _id: UserId,
            _current_token: &str,
            _new_token: &str,
        ) -> Result<UserRecord, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
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
                        progress_consented: false,
                        is_idle: false,
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

        fn find_paginated_by_status(
            &self,
            filter: GameStatusFilter,
            page: u32,
            per_page: u32,
        ) -> Result<(Vec<GameSummary>, u64), RepositoryError> {
            let all: Vec<GameSummary> = self
                .games
                .borrow()
                .iter()
                .filter(|g| match filter {
                    GameStatusFilter::Active => !matches!(g.status, GameStatus::Closed | GameStatus::Aborted),
                    GameStatusFilter::Closed => matches!(g.status, GameStatus::Closed),
                    GameStatusFilter::Aborted => matches!(g.status, GameStatus::Aborted),
                })
                .map(game_to_summary_test)
                .collect();
            let total = all.len() as u64;
            let start = (page as usize).saturating_sub(1) * per_page as usize;
            let items = all.into_iter().skip(start).take(per_page as usize).collect();
            Ok((items, total))
        }

        fn find_paginated_by_user_uuid(
            &self,
            user_uuid: uuid::Uuid,
            filter: Option<GameStatusFilter>,
            page: u32,
            per_page: u32,
        ) -> Result<(Vec<GameSummary>, u64), RepositoryError> {
            let all: Vec<GameSummary> = self
                .games
                .borrow()
                .iter()
                .filter(|g| g.players.iter().any(|p| p.user_uuid == user_uuid))
                .filter(|g| match filter {
                    None => true,
                    Some(GameStatusFilter::Active) => !matches!(g.status, GameStatus::Closed | GameStatus::Aborted),
                    Some(GameStatusFilter::Closed) => matches!(g.status, GameStatus::Closed),
                    Some(GameStatusFilter::Aborted) => matches!(g.status, GameStatus::Aborted),
                })
                .map(game_to_summary_test)
                .collect();
            let total = all.len() as u64;
            let start = (page as usize).saturating_sub(1) * per_page as usize;
            let items = all.into_iter().skip(start).take(per_page as usize).collect();
            Ok((items, total))
        }
    }

    fn game_to_summary_test(game: &Game) -> GameSummary {
        GameSummary {
            uuid: game.uuid,
            game_number: game.game_number,
            status: game.status,
            next_update_at: game.next_update_at,
            regulation: game.regulation,
            player_count: game.players.len() as u64,
            season_label: game.current_season_label(),
            owner_user_uuid: game.players.iter().find(|p| p.is_owner).map(|p| p.user_uuid),
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
            is_admin: false,
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
                progress_mode: None,
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
    fn handle_create_game_defaults_to_scheduled_progress_mode() {
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
            is_admin: false,
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
                progress_mode: None,
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
    fn handle_create_game_accepts_consensus_progress_mode() {
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
            is_admin: false,
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
                progress_mode: Some(2),
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: None,
                keyword: None,
            },
        )
        .expect("create should succeed");

        let game = repo_clone.last_inserted().expect("a game should have been inserted");
        assert_eq!(game.regulation.progress_mode, ProgressMode::Consensus);
    }

    #[test]
    fn handle_create_game_rejects_invalid_progress_mode() {
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
                progress_mode: Some(3),
                duration_type: 1,
                start_date,
                first_period_hour,
                requested_power: None,
                keyword: None,
            },
        )
        .expect_err("create should fail");

        assert_eq!(error.code(), "invalid_request");
        assert!(matches!(
            error,
            CreateGameHandlerError::InvalidRequest(CreateGameRequestValidationError::InvalidProgressMode)
        ));
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
                progress_mode: None,
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
            is_admin: false,
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
                progress_consented: false,
                is_idle: false,
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
                progress_mode: None,
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
                progress_consented: false,
                is_idle: false,
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

    fn sample_consensus_game_for_handler(owner_uuid: uuid::Uuid, player_uuid: uuid::Uuid) -> Game {
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.regulation.progress_mode = crate::domain::ProgressMode::Consensus;
        game.players[0].power = Some(crate::domain::Power::France);
        game.players.push(Player {
            user_uuid: player_uuid,
            power: Some(crate::domain::Power::England),
            is_accepting_draw: false,
            progress_consented: false,
            is_idle: false,
            is_owner: false,
            requested_power: None,
        });

        let phase = game.phases.last_mut().expect("phase exists");
        phase.territories.clear();
        phase
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::France, "par"));
        phase
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::England, "lon"));

        game
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
                season: None,
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
                is_admin: false,
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
                is_admin: false,
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
                season: None,
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
            is_admin: false,
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
                season: None,
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
            is_admin: false,
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
                season: None,
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert!(response.draw_proposal);
    }

    fn owner_user_record_for_draw_proposal_tests(owner_uuid: uuid::Uuid) -> UserRecord {
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
            is_admin: false,
        }
    }

    fn draw_proposal_request(game_uuid: uuid::Uuid, draw_proposal: bool, season: Option<&str>) -> SetDrawProposalRequest {
        SetDrawProposalRequest {
            authorization: "Bearer token-owner".to_string(),
            game_uuid,
            draw_proposal,
            season: season.map(str::to_string),
        }
    }

    #[test]
    fn handle_get_game_draw_proposal_follows_propose_and_rescind() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record_for_draw_proposal_tests(owner_uuid)]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let before = handle_get_game(&service, game_uuid, None).expect("get should succeed");
        assert!(!before.game.draw_proposal);

        handle_set_draw_proposal(&service, &message_repository, draw_proposal_request(game_uuid, true, None))
            .expect("propose should succeed");
        let proposed = handle_get_game(&service, game_uuid, None).expect("get should succeed");
        assert!(proposed.game.draw_proposal);

        handle_set_draw_proposal(&service, &message_repository, draw_proposal_request(game_uuid, false, None))
            .expect("rescind should succeed");
        let rescinded = handle_get_game(&service, game_uuid, None).expect("get should succeed");
        assert!(!rescinded.game.draw_proposal);
    }

    #[test]
    fn handle_get_game_draw_proposal_true_when_owner_marked_accepting_draw_by_idle() {
        // 卓主の無政府化（mark_owner_accepting_draw_if_idle）で立ったフラグも同じ is_accepting_draw に保存される
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record_for_draw_proposal_tests(owner_uuid)]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.players.iter_mut().find(|p| p.is_owner).unwrap().is_accepting_draw = true;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_game(&service, game_uuid, None).expect("get should succeed");

        assert!(response.game.draw_proposal);
    }

    #[test]
    fn handle_get_game_draw_proposal_false_when_no_phases() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record_for_draw_proposal_tests(owner_uuid)]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.players.iter_mut().find(|p| p.is_owner).unwrap().is_accepting_draw = true;
        game.phases.clear();
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_game(&service, game_uuid, None).expect("get should succeed");

        assert!(!response.game.draw_proposal);
    }

    #[test]
    fn handle_set_draw_proposal_rejects_stale_season_with_phase_conflict() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record_for_draw_proposal_tests(owner_uuid)]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_draw_proposal(
            &service,
            &message_repository,
            draw_proposal_request(game_uuid, true, Some("1900f")),
        )
        .expect_err("stale season should be rejected");

        assert_eq!(error.code(), "phase_conflict");
        let response = handle_get_game(&service, game_uuid, None).expect("get should succeed");
        assert!(!response.game.draw_proposal, "flag must not be set on conflict");
    }

    #[test]
    fn handle_set_draw_proposal_accepts_current_season() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record_for_draw_proposal_tests(owner_uuid)]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let season = game.current_turn();
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_draw_proposal(
            &service,
            &message_repository,
            draw_proposal_request(game_uuid, true, Some(&season)),
        )
        .expect("current season should be accepted");

        assert!(response.draw_proposal);
    }

    #[test]
    fn handle_set_draw_proposal_rejects_invalid_season_format() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_draw_proposal(
            &service,
            &message_repository,
            draw_proposal_request(uuid::Uuid::now_v7(), true, Some("spring")),
        )
        .expect_err("invalid season should be rejected");

        assert_eq!(error.code(), "invalid_request");
    }

    fn sample_in_progress_game_with_next_update(owner_uuid: uuid::Uuid) -> Game {
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let future_jst = (Utc::now() + chrono::Duration::hours(2)).with_timezone(&jst);
        let future_naive_jst = future_jst.naive_local();
        let future_utc = jst
            .from_local_datetime(&future_naive_jst)
            .single()
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();
        game.next_update_at = Some(future_utc);
        game
    }

    #[test]
    fn handle_set_next_update_at_rejects_missing_authorization() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_next_update_at(
            &service,
            &message_repository,
            SetNextUpdateAtRequest {
                authorization: "".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                next_update_at: "2099-01-01 12:00".to_string(),
                season: "ready".to_string(),
            },
        )
        .expect_err("should fail without authorization");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_next_update_at_rejects_non_owner() {
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
                is_admin: false,
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
                is_admin: false,
            },
        ]);
        let game = sample_in_progress_game_with_next_update(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_next_update_at(
            &service,
            &message_repository,
            SetNextUpdateAtRequest {
                authorization: "Bearer token-other".to_string(),
                game_uuid,
                next_update_at: "2099-01-01 12:00".to_string(),
                season: "ready".to_string(),
            },
        )
        .expect_err("should fail for non-owner");

        assert_eq!(error.code(), "forbidden");
    }

    #[test]
    fn handle_set_next_update_at_rejects_non_main_or_ready_phase() {
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
            is_admin: false,
        }]);
        let mut game = sample_in_progress_game_with_next_update(owner_uuid);
        // 撤退フェーズに変更
        let spring_retreat = Phase::new_spring_retreat(1901, 1);
        game.phases.push(spring_retreat);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_next_update_at(
            &service,
            &message_repository,
            SetNextUpdateAtRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                next_update_at: "2099-01-01 12:00".to_string(),
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail during retreat phase");

        assert_eq!(error.code(), "forbidden");
    }

    #[test]
    fn handle_set_next_update_at_rejects_past_time() {
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
            is_admin: false,
        }]);
        let mut game = sample_in_progress_game_with_next_update(owner_uuid);
        // next_update_at を未来にセット（パスト判定のため、リクエストは現在より過去）
        // next_update_at を遠い未来にセット
        let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let far_future = (Utc::now() + chrono::Duration::days(365)).naive_utc();
        game.next_update_at = Some(far_future);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        // 過去の時刻を JST で指定
        // 5分刻みに丸めた過去の時刻を JST で指定
        let past_jst = (Utc::now() - chrono::Duration::hours(1)).with_timezone(&jst);
        let minute = past_jst.minute();
        let rounded_minute = (minute / 5) * 5;
        let past_str = format!("{} {:02}:{:02}", past_jst.format("%Y-%m-%d"), past_jst.hour(), rounded_minute);

        let error = handle_set_next_update_at(
            &service,
            &message_repository,
            SetNextUpdateAtRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                next_update_at: past_str,
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail for past time");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_next_update_at_updates_and_returns_response() {
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
            is_admin: false,
        }]);
        let game = sample_in_progress_game_with_next_update(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_next_update_at(
            &service,
            &message_repository,
            SetNextUpdateAtRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                next_update_at: "2099-06-15 14:00".to_string(),
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert_eq!(response.next_update_at, "2099-06-15 14:00");
    }

    #[test]
    fn handle_set_next_update_at_returns_noop_when_same_time() {
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
            is_admin: false,
        }]);

        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let same_time_jst = "2099-06-15 14:00";
        let same_naive = chrono::NaiveDateTime::parse_from_str(same_time_jst, "%Y-%m-%d %H:%M").unwrap();
        let same_utc = jst
            .from_local_datetime(&same_naive)
            .single()
            .unwrap()
            .with_timezone(&Utc)
            .naive_utc();
        game.next_update_at = Some(same_utc);

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let repo_clone = game_repository.clone();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let response = handle_set_next_update_at(
            &service,
            &message_repository,
            SetNextUpdateAtRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                next_update_at: same_time_jst.to_string(),
                season: "1901s".to_string(),
            },
        )
        .expect("no-op should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert_eq!(response.next_update_at, same_time_jst);
        assert!(repo_clone.last_updated().is_none(), "update should not be called for no-op");
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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
            is_admin: false,
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

    #[test]
    fn handle_set_unit_rejects_invalid_season_format() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        for bad_season in &["", "1901", "1901x", "190s", "19011s", "SPRING"] {
            let error = handle_set_unit(
                &service,
                &message_repository,
                SetUnitRequest {
                    authorization: "Bearer dummy".to_string(),
                    game_uuid: uuid::Uuid::now_v7(),
                    unit: None,
                    location: "par".to_string(),
                    season: bad_season.to_string(),
                },
            )
            .expect_err(&format!("should reject invalid season: {bad_season}"));

            assert_eq!(error.code(), "invalid_request", "season={bad_season}");
        }
    }

    #[test]
    fn handle_set_unit_accepts_uppercase_season() {
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
            is_admin: false,
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        // "1901S" (大文字) は "1901s" に正規化されてフェイズと一致する
        let result = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                unit: None,
                location: "par".to_string(),
                season: "1901S".to_string(),
            },
        );

        assert!(result.is_ok(), "uppercase season should be accepted");
    }

    #[test]
    fn handle_set_territory_rejects_invalid_season_format() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        for bad_season in &["", "1901", "1901x", "190f", "19011f", "FALL"] {
            let error = handle_set_territory(
                &service,
                &message_repository,
                SetTerritoryRequest {
                    authorization: "Bearer dummy".to_string(),
                    game_uuid: uuid::Uuid::now_v7(),
                    code: "par".to_string(),
                    power: None,
                    season: bad_season.to_string(),
                },
            )
            .expect_err(&format!("should reject invalid season: {bad_season}"));

            assert_eq!(error.code(), "invalid_request", "season={bad_season}");
        }
    }

    // ============================================================================
    // handle_set_progress_mode tests
    // ============================================================================

    #[test]
    fn handle_set_progress_mode_rejects_missing_authorization() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail without authorization");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_progress_mode_rejects_invalid_season_format() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer dummy".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                season: "1901x".to_string(),
            },
        )
        .expect_err("should fail with invalid season");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_progress_mode_rejects_unauthorized() {
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
            is_admin: false,
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer invalid-token".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail with unknown token");

        assert_eq!(error.code(), "unauthorized");
    }

    #[test]
    fn handle_set_progress_mode_returns_not_found_for_missing_game() {
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
            is_admin: false,
        }]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail for missing game");

        assert_eq!(error.code(), "not_found");
    }

    #[test]
    fn handle_set_progress_mode_returns_forbidden_for_non_owner() {
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
                is_admin: false,
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
                is_admin: false,
            },
        ]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer token-other".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            },
        )
        .expect_err("should fail for non-owner");

        assert_eq!(error.code(), "forbidden");
    }

    #[test]
    fn handle_set_progress_mode_returns_phase_conflict_for_wrong_season() {
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
            is_admin: false,
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                season: "1901f".to_string(), // wrong season
            },
        )
        .expect_err("should fail for wrong season");

        assert_eq!(error.code(), "phase_conflict");
    }

    #[test]
    fn handle_set_progress_mode_returns_ok_when_already_consensus() {
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
            is_admin: false,
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.regulation.progress_mode = crate::domain::ProgressMode::Consensus;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());
        let message_repository = new_test_message_repository();

        let response = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed for already consensus");

        assert_eq!(response.game_uuid, game_uuid);
        assert_eq!(response.progress_mode, "consensus");
        assert!(
            game_repository.last_updated().is_none(),
            "repository.update should not be called"
        );
    }

    #[test]
    fn handle_set_progress_mode_changes_to_consensus_and_records_message() {
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
            is_admin: false,
        }]);
        let game = sample_in_progress_game_for_handler(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());
        let message_repository = new_test_message_repository();

        let response = handle_set_progress_mode(
            &service,
            &message_repository,
            SetProgressModeRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert_eq!(response.progress_mode, "consensus");
        let updated = game_repository.last_updated().expect("game should be updated");
        assert_eq!(updated.regulation.progress_mode, crate::domain::ProgressMode::Consensus);
    }

    // ============================================================================
    // handle_set_progress_consensus tests
    // ============================================================================

    #[test]
    fn handle_set_progress_consensus_rejects_missing_authorization() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_consensus(
            &service,
            &message_repository,
            SetProgressConsensusRequest {
                authorization: "".to_string(),
                game_uuid: uuid::Uuid::now_v7(),
                agreed: true,
            },
        )
        .expect_err("should fail without authorization");

        assert_eq!(error.code(), "invalid_request");
    }

    #[test]
    fn handle_set_progress_consensus_returns_forbidden_when_mode_not_consensus() {
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
            is_admin: false,
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.players[0].power = Some(crate::domain::Power::France);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_set_progress_consensus(
            &service,
            &message_repository,
            SetProgressConsensusRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                agreed: true,
            },
        )
        .expect_err("should be forbidden");

        assert_eq!(error.code(), "forbidden");
    }

    #[test]
    fn handle_set_progress_consensus_returns_ok_on_noop() {
        let owner_uuid = uuid::Uuid::now_v7();
        let player_uuid = uuid::Uuid::now_v7();
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
                is_admin: false,
            },
            UserRecord {
                id: 2,
                uuid: player_uuid,
                discord_user_id: "discord-player".to_string(),
                username: "player".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-player".to_string(),
                last_access_at: Utc::now(),
                is_admin: false,
            },
        ]);
        let mut game = sample_consensus_game_for_handler(owner_uuid, player_uuid);
        game.players[0].progress_consented = true;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());
        let message_repository = new_test_message_repository();

        let response = handle_set_progress_consensus(
            &service,
            &message_repository,
            SetProgressConsensusRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                agreed: true,
            },
        )
        .expect("no-op should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert!(response.agreed);
        assert!(
            game_repository.last_updated().is_none(),
            "repository.update should not be called"
        );
    }

    #[test]
    fn handle_set_progress_consensus_changes_and_persists() {
        let owner_uuid = uuid::Uuid::now_v7();
        let player_uuid = uuid::Uuid::now_v7();
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
                is_admin: false,
            },
            UserRecord {
                id: 2,
                uuid: player_uuid,
                discord_user_id: "discord-player".to_string(),
                username: "player".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-player".to_string(),
                last_access_at: Utc::now(),
                is_admin: false,
            },
        ]);
        let game = sample_consensus_game_for_handler(owner_uuid, player_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());
        let message_repository = new_test_message_repository();

        let response = handle_set_progress_consensus(
            &service,
            &message_repository,
            SetProgressConsensusRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                agreed: true,
            },
        )
        .expect("should succeed");

        assert_eq!(response.game_uuid, game_uuid);
        assert!(response.agreed);
        let updated = game_repository.last_updated().expect("game should be updated");
        assert!(
            updated
                .players
                .iter()
                .find(|p| p.user_uuid == owner_uuid)
                .expect("owner exists")
                .progress_consented,
            "owner consent should be true"
        );
    }

    fn build_game_for_get_games_tests(game_uuid: uuid::Uuid, owner_uuid: uuid::Uuid, status: GameStatus) -> Game {
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.uuid = game_uuid;
        game.status = status;
        game
    }

    fn user_record_for_is_owner_tests(id: i64, uuid: uuid::Uuid, name: &str) -> UserRecord {
        UserRecord {
            id,
            uuid,
            discord_user_id: format!("{}", 1000 + id),
            username: name.to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: format!("token-{}", name),
            last_access_at: Utc::now(),
            is_admin: false,
        }
    }

    fn get_games_is_owner_for(authorization: Option<&str>) -> bool {
        let owner_uuid = uuid::Uuid::now_v7();
        let other_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            user_record_for_is_owner_tests(1, owner_uuid, "alice"),
            user_record_for_is_owner_tests(2, other_uuid, "bob"),
        ]);
        let game = build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::InProgress);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: authorization.map(|s| s.to_string()),
                status: None,
                user: None,
                page: 1,
                per_page: 20,
            },
        )
        .expect("should succeed");

        assert_eq!(response.games.len(), 1);
        response.games[0].is_owner
    }

    #[test]
    fn handle_get_games_is_owner_true_for_owner() {
        assert!(get_games_is_owner_for(Some("Bearer token-alice")));
    }

    #[test]
    fn handle_get_games_is_owner_false_for_non_owner() {
        assert!(!get_games_is_owner_for(Some("Bearer token-bob")));
    }

    #[test]
    fn handle_get_games_is_owner_false_without_authorization() {
        assert!(!get_games_is_owner_for(None));
    }

    #[test]
    fn handle_get_games_is_owner_false_for_unknown_or_malformed_token() {
        assert!(!get_games_is_owner_for(Some("Bearer token-unknown")));
        assert!(!get_games_is_owner_for(Some("token-alice")));
        assert!(!get_games_is_owner_for(Some("Bearer ")));
    }

    #[test]
    fn handle_get_games_is_owner_true_for_own_user_search() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![user_record_for_is_owner_tests(1, owner_uuid, "alice")]);
        let game = build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::InProgress);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: Some("Bearer token-alice".to_string()),
                status: None,
                user: Some("1001".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect("should succeed");

        assert_eq!(response.games.len(), 1);
        assert!(response.games[0].is_owner);
    }

    fn get_game_is_owner_for(authorization: Option<&str>) -> bool {
        let owner_uuid = uuid::Uuid::now_v7();
        let other_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            user_record_for_is_owner_tests(1, owner_uuid, "alice"),
            user_record_for_is_owner_tests(2, other_uuid, "bob"),
        ]);
        let game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::InProgress);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        handle_get_game(&service, game_uuid, authorization)
            .expect("should succeed")
            .game
            .is_owner
    }

    #[test]
    fn handle_get_game_is_owner_true_for_owner() {
        assert!(get_game_is_owner_for(Some("Bearer token-alice")));
    }

    #[test]
    fn handle_get_game_is_owner_false_for_non_owner() {
        assert!(!get_game_is_owner_for(Some("Bearer token-bob")));
    }

    #[test]
    fn handle_get_game_is_owner_false_without_authorization() {
        assert!(!get_game_is_owner_for(None));
        assert!(!get_game_is_owner_for(Some("Bearer token-unknown")));
    }

    /// France（ユニットあり）・England（ユニットなし）・担当国なしの 3 人がいる卓で、
    /// 卓情報取得 API の players[].is_eliminated を返す
    fn get_game_is_eliminated_for(latest_phase: Option<Phase>) -> Vec<bool> {
        let owner_uuid = uuid::Uuid::now_v7();
        let england_uuid = uuid::Uuid::now_v7();
        let unassigned_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            user_record_for_is_owner_tests(1, owner_uuid, "alice"),
            user_record_for_is_owner_tests(2, england_uuid, "bob"),
            user_record_for_is_owner_tests(3, unassigned_uuid, "carol"),
        ]);
        let mut game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::InProgress);
        game.players[0].power = Some(crate::domain::Power::France);
        for (user_uuid, power) in [(england_uuid, Some(crate::domain::Power::England)), (unassigned_uuid, None)] {
            game.players.push(Player {
                user_uuid,
                power,
                is_accepting_draw: false,
                progress_consented: false,
                is_idle: false,
                is_owner: false,
                requested_power: None,
            });
        }
        let phase = game.phases.last_mut().expect("phase exists");
        phase.units.clear();
        phase.units.push(crate::domain::Unit::new_army(
            crate::domain::Power::France,
            crate::domain::Province::from_code("par").expect("par"),
        ));
        if let Some(latest_phase) = latest_phase {
            game.phases.push(latest_phase);
        }
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        handle_get_game(&service, game_uuid, None)
            .expect("should succeed")
            .game
            .players
            .iter()
            .map(|player| player.is_eliminated)
            .collect()
    }

    #[test]
    fn handle_get_game_is_eliminated_true_only_for_power_without_units() {
        assert_eq!(get_game_is_eliminated_for(None), vec![false, true, false]);
    }

    #[test]
    fn handle_get_game_is_eliminated_false_during_debrief() {
        let debrief = Phase::new_debrief(1901, 1);
        assert_eq!(get_game_is_eliminated_for(Some(debrief)), vec![false, false, false]);
    }

    #[test]
    fn handle_get_games_returns_active_games_by_default() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "alice".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-alice".to_string(),
            last_access_at: Utc::now(),
            is_admin: false,
        }]);
        let game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::InProgress);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: None,
                user: None,
                page: 1,
                per_page: 20,
            },
        )
        .expect("should succeed");

        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 20);
        assert_eq!(response.games.len(), 1);
        assert_eq!(response.games[0].game_uuid, game_uuid);
    }

    #[test]
    fn handle_get_games_filters_own_user_games_by_status() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "alice".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-alice".to_string(),
            last_access_at: Utc::now(),
            is_admin: false,
        }]);
        let active = build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::Solo);
        let closed = build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::Closed);
        let closed_uuid = closed.uuid;
        let game_repository = InMemoryGameRepository::new_with_games(vec![active, closed]);
        let service = GameService::new(user_repository, game_repository);

        let request = |status: Option<&str>| GetGamesRequest {
            authorization: Some("Bearer token-alice".to_string()),
            status: status.map(str::to_string),
            user: Some("1001".to_string()),
            page: 1,
            per_page: 20,
        };

        let all = handle_get_games(&service, request(None)).expect("should succeed");
        assert_eq!(all.total, 2);

        let active_only = handle_get_games(&service, request(Some("active"))).expect("should succeed");
        assert_eq!(active_only.total, 1);
        assert_eq!(active_only.games[0].status, "solo");

        let closed_only = handle_get_games(&service, request(Some("closed"))).expect("should succeed");
        assert_eq!(closed_only.total, 1);
        assert_eq!(closed_only.games[0].game_uuid, closed_uuid);

        let aborted_only = handle_get_games(&service, request(Some("aborted"))).expect("should succeed");
        assert_eq!(aborted_only.total, 0);
        assert!(aborted_only.games.is_empty());
    }

    #[test]
    fn handle_get_games_returns_error_for_invalid_status() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: Some("unknown".to_string()),
                user: None,
                page: 1,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::InvalidStatus)
        ));
    }

    #[test]
    fn handle_get_games_returns_games_for_own_user() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "alice".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-alice".to_string(),
            last_access_at: Utc::now(),
            is_admin: false,
        }]);
        let game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::Solo);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: Some("Bearer token-alice".to_string()),
                status: None,
                user: Some("1001".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect("should succeed");

        assert_eq!(response.games.len(), 1);
        assert_eq!(response.games[0].status, "solo");
    }

    #[test]
    fn handle_get_games_returns_forbidden_for_other_user() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "1001".to_string(),
            username: "alice".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-alice".to_string(),
            last_access_at: Utc::now(),
            is_admin: false,
        }]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: Some("Bearer token-alice".to_string()),
                status: None,
                user: Some("9999".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(error, GetGamesHandlerError::Service(ListGamesError::Forbidden(_))));
    }

    #[test]
    fn handle_get_games_returns_error_for_empty_user() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: Some("Bearer token".to_string()),
                status: None,
                user: Some("".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::InvalidUser)
        ));
    }

    #[test]
    fn handle_get_games_returns_error_when_user_specified_without_auth() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: None,
                user: Some("1001".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::MissingAuthorization)
        ));
    }

    #[test]
    fn handle_get_games_returns_error_for_invalid_auth_scheme() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: Some("Basic abc".to_string()),
                status: None,
                user: Some("1001".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::InvalidAuthorizationScheme)
        ));
    }

    #[test]
    fn handle_get_games_returns_error_for_missing_access_token() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: Some("Bearer ".to_string()),
                status: None,
                user: Some("1001".to_string()),
                page: 1,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::MissingAccessToken)
        ));
    }

    #[test]
    fn handle_get_games_returns_error_for_invalid_page() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: None,
                user: None,
                page: 0,
                per_page: 20,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::InvalidPage)
        ));
    }

    #[test]
    fn handle_get_games_returns_error_for_invalid_per_page() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: None,
                user: None,
                page: 1,
                per_page: 101,
            },
        )
        .expect_err("should fail");

        assert!(matches!(
            error,
            GetGamesHandlerError::InvalidRequest(GetGamesRequestValidationError::InvalidPerPage)
        ));
    }

    #[test]
    fn handle_get_games_filters_by_closed_status() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![]);
        let active_game = build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::InProgress);
        let closed_game = build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::Closed);
        let game_repository = InMemoryGameRepository::new_with_games(vec![active_game, closed_game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: Some("closed".to_string()),
                user: None,
                page: 1,
                per_page: 20,
            },
        )
        .expect("should succeed");

        assert_eq!(response.games.len(), 1);
        assert_eq!(response.games[0].status, "closed");
        assert_eq!(response.total, 1);
    }

    #[test]
    fn handle_get_games_paginates_correctly() {
        let owner_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![]);
        let games: Vec<Game> = (0..5)
            .map(|_| build_game_for_get_games_tests(uuid::Uuid::now_v7(), owner_uuid, GameStatus::InProgress))
            .collect();
        let game_repository = InMemoryGameRepository::new_with_games(games);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: None,
                user: None,
                page: 1,
                per_page: 3,
            },
        )
        .expect("should succeed");

        assert_eq!(response.games.len(), 3);
        assert_eq!(response.total, 5);
        assert_eq!(response.per_page, 3);

        let page2 = handle_get_games(
            &service,
            GetGamesRequest {
                authorization: None,
                status: None,
                user: None,
                page: 2,
                per_page: 3,
            },
        )
        .expect("should succeed");

        assert_eq!(page2.games.len(), 2);
        assert_eq!(page2.total, 5);
    }

    #[test]
    fn handle_get_game_returns_game_for_existing_uuid() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![user_record_for_is_owner_tests(1, owner_uuid, "alice")]);
        let game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::InProgress);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_game(&service, game_uuid, None).expect("should succeed");

        assert_eq!(response.game.game_uuid, game_uuid);
        assert_eq!(response.game.status, "in_progress");
        assert!(!response.game.seasons.is_empty());
        assert_eq!(response.game.seasons[0], "ready");
    }

    #[test]
    fn handle_get_game_returns_not_found_for_missing_uuid() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_game(&service, uuid::Uuid::now_v7(), None).expect_err("should fail");

        assert_eq!(error.code(), "not_found");
    }

    #[test]
    fn handle_get_game_seasons_ends_with_debrief_for_solo() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![user_record_for_is_owner_tests(1, owner_uuid, "alice")]);
        let mut game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::Solo);
        game.phases.push(crate::domain::Phase::new_debrief(1901, 10));
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_game(&service, game_uuid, None).expect("should succeed");

        assert_eq!(response.game.status, "solo");
        assert_eq!(response.game.seasons.last().map(|s| s.as_str()), Some("debrief"));
    }

    #[test]
    fn handle_get_game_is_private_when_keyword_is_set() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![user_record_for_is_owner_tests(1, owner_uuid, "alice")]);
        let mut game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::Preparing);
        game.keyword = Some("secret".to_string());
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_game(&service, game_uuid, None).expect("should succeed");

        assert!(response.game.is_private);
    }

    #[test]
    fn handle_get_game_is_not_private_when_keyword_is_none() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![user_record_for_is_owner_tests(1, owner_uuid, "alice")]);
        let mut game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::Preparing);
        game.keyword = None;
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let response = handle_get_game(&service, game_uuid, None).expect("should succeed");

        assert!(!response.game.is_private);
    }

    fn get_game_with_players_for(authorization: Option<&str>) -> crate::api::GetGameResponseGame {
        let owner_uuid = uuid::Uuid::now_v7();
        let member_uuid = uuid::Uuid::now_v7();
        let outsider_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let mut alice = user_record_for_is_owner_tests(1, owner_uuid, "alice");
        alice.global_name = Some("Alice".to_string());
        alice.avatar_url = Some("https://example.com/alice.png".to_string());
        let user_repository = InMemoryUserRepository::new(vec![
            alice,
            user_record_for_is_owner_tests(2, member_uuid, "bob"),
            user_record_for_is_owner_tests(3, outsider_uuid, "carol"),
        ]);
        let mut game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::Preparing);
        game.regulation.start_date = NaiveDate::from_ymd_opt(2026, 11, 24).expect("valid date");
        game.regulation.first_period_hour = 7;
        game.players[0].requested_power = Some(Power::France);
        game.players.push(Player {
            user_uuid: member_uuid,
            power: Some(Power::England),
            is_accepting_draw: false,
            progress_consented: false,
            is_idle: false,
            is_owner: false,
            requested_power: Some(Power::Turkey),
        });
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        handle_get_game(&service, game_uuid, authorization)
            .expect("should succeed")
            .game
    }

    #[test]
    fn handle_get_game_returns_regulation_start_at_and_players() {
        let game = get_game_with_players_for(None);

        assert_eq!(game.regulation.face_type, "girls");
        assert_eq!(game.regulation.progress_mode, "scheduled");
        assert_eq!(game.regulation.duration_type, "short");
        assert_eq!(game.start_at, "2026-11-24 07:00");
        assert!(!game.is_joined);

        assert_eq!(game.players.len(), 2);
        let owner = &game.players[0];
        assert!(owner.is_owner);
        assert_eq!(owner.user.discord_user_id, "1001");
        assert_eq!(owner.user.username, "alice");
        assert_eq!(owner.user.global_name.as_deref(), Some("Alice"));
        assert_eq!(owner.user.avatar_url.as_deref(), Some("https://example.com/alice.png"));
        assert_eq!(owner.requested_power.as_deref(), Some("f"));
        assert_eq!(owner.power, None);

        let member = &game.players[1];
        assert!(!member.is_owner);
        assert_eq!(member.user.username, "bob");
        assert_eq!(member.user.global_name, None);
        assert_eq!(member.power.as_deref(), Some("e"));
    }

    #[test]
    fn handle_get_game_hides_non_owner_requested_power_from_others() {
        assert_eq!(get_game_with_players_for(None).players[1].requested_power, None);
        assert_eq!(
            get_game_with_players_for(Some("Bearer token-carol")).players[1].requested_power,
            None
        );
        assert_eq!(
            get_game_with_players_for(Some("Bearer token-alice")).players[1].requested_power,
            None
        );
    }

    #[test]
    fn handle_get_game_shows_own_requested_power_to_self() {
        let game = get_game_with_players_for(Some("Bearer token-bob"));

        assert_eq!(game.players[1].requested_power.as_deref(), Some("t"));
        assert_eq!(game.players[0].requested_power.as_deref(), Some("f"));
    }

    #[test]
    fn handle_get_game_is_joined_reflects_viewer() {
        assert!(get_game_with_players_for(Some("Bearer token-alice")).is_joined);
        assert!(get_game_with_players_for(Some("Bearer token-bob")).is_joined);
        assert!(!get_game_with_players_for(Some("Bearer token-carol")).is_joined);
        assert!(!get_game_with_players_for(Some("Bearer token-unknown")).is_joined);
        assert!(!get_game_with_players_for(None).is_joined);
    }

    #[test]
    fn handle_get_game_returns_repository_error_when_player_user_is_missing() {
        let owner_uuid = uuid::Uuid::now_v7();
        let game_uuid = uuid::Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game = build_game_for_get_games_tests(game_uuid, owner_uuid, GameStatus::Preparing);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = handle_get_game(&service, game_uuid, None).expect_err("should fail");

        assert_eq!(error.code(), "repository_error");
    }

    #[test]
    fn join_game_handler_error_code_reflects_forbidden_reason() {
        use crate::services::JoinGameForbiddenReason;
        let cases = [
            (JoinGameForbiddenReason::AlreadyInAnotherGame, "already_in_another_game"),
            (JoinGameForbiddenReason::KeywordMismatch, "keyword_mismatch"),
            (JoinGameForbiddenReason::NotAcceptingPlayers, "not_accepting_players"),
            (JoinGameForbiddenReason::GameFull, "game_full"),
        ];
        for (reason, code) in cases {
            let error = JoinGameHandlerError::Service(JoinGameError::Forbidden(reason));
            assert_eq!(error.code(), code);
            assert_eq!(error.to_api_error_response().message, reason.message());
        }
    }

    // ============================================================================
    // handle_get_games_logs tests
    // ============================================================================

    fn sample_in_progress_game_for_logs(owner_uuid: uuid::Uuid, player_uuid: uuid::Uuid) -> Game {
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
            players: vec![
                Player {
                    user_uuid: owner_uuid,
                    power: Some(crate::domain::Power::France),
                    is_accepting_draw: false,
                    progress_consented: false,
                    is_idle: false,
                    is_owner: true,
                    requested_power: None,
                },
                Player {
                    user_uuid: player_uuid,
                    power: Some(crate::domain::Power::England),
                    is_accepting_draw: false,
                    progress_consented: false,
                    is_idle: false,
                    is_owner: false,
                    requested_power: None,
                },
            ],
            phases: vec![ready, spring_main],
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        }
    }

    fn make_message_record(
        kind: &str,
        sender: Option<crate::domain::Power>,
        recipients: Vec<crate::domain::Power>,
    ) -> crate::MessageRecord {
        crate::MessageRecord {
            message_uuid: uuid::Uuid::now_v7(),
            sender_kind: if kind == "system" {
                crate::MessageSenderKind::System
            } else {
                crate::MessageSenderKind::Power
            },
            sender_power: sender,
            sender_user_uuid: None,
            turn: "1901s".to_string(),
            context: "test message".to_string(),
            kind: kind.to_string(),
            recipients,
            created_at: "2026-05-10T11:00:00Z".to_string(),
            notice: None,
        }
    }

    #[test]
    fn handle_get_games_logs_returns_not_found_for_missing_game() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_get_games_logs(
            &service,
            &user_repository,
            &message_repository,
            uuid::Uuid::now_v7(),
            "1901s",
            None,
            None,
        )
        .expect_err("should fail");

        assert_eq!(error.code(), "not_found");
    }

    #[test]
    fn handle_get_games_logs_returns_not_found_for_invalid_season() {
        let owner_uuid = uuid::Uuid::now_v7();
        let player_uuid = uuid::Uuid::now_v7();
        let game = sample_in_progress_game_for_logs(owner_uuid, player_uuid);
        let game_uuid = game.uuid;
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let error = handle_get_games_logs(
            &service,
            &user_repository,
            &message_repository,
            game_uuid,
            "9999z",
            None,
            None,
        )
        .expect_err("should fail");

        assert_eq!(error.code(), "not_found");
    }

    #[test]
    fn handle_get_games_logs_returns_system_notice_with_codes() {
        let owner_uuid = uuid::Uuid::now_v7();
        let player_uuid = uuid::Uuid::now_v7();
        let game = sample_in_progress_game_for_logs(owner_uuid, player_uuid);
        let game_uuid = game.uuid;
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        let service = GameService::new(user_repository.clone(), game_repository);
        let message_repository = new_test_message_repository();

        let munich = crate::domain::Province::from_code("mun").expect("mun");
        let stp_sc = crate::domain::Province::from_code("stp_sc").expect("stp_sc");
        message_repository
            .append_unit_placed_message(
                game_uuid,
                "1901s",
                crate::domain::Unit::new_army(crate::domain::Power::Germany, munich),
            )
            .expect("unit placed");
        message_repository
            .append_unit_replaced_message(
                game_uuid,
                "1901s",
                crate::domain::Unit::new_army(crate::domain::Power::Germany, munich),
                crate::domain::Unit::new_fleet(crate::domain::Power::Russia, stp_sc),
            )
            .expect("unit replaced");
        message_repository
            .append_territory_replaced_message(
                game_uuid,
                "1901s",
                munich,
                crate::domain::Power::Austria,
                crate::domain::Power::Germany,
            )
            .expect("territory replaced");
        message_repository
            .append_progress_consented_message(game_uuid, "1901s", crate::domain::Power::France)
            .expect("progress consented");
        message_repository
            .append_chat_message(
                game_uuid,
                crate::repositories::NewChatMessage {
                    sender_kind: crate::MessageSenderKind::Power,
                    sender_power: Some(crate::domain::Power::France),
                    sender_user_uuid: player_uuid,
                    turn: "1901s".to_string(),
                    context: "公開".to_string(),
                    kind: crate::MessageKind::public(),
                },
            )
            .expect("chat message");

        let response = handle_get_games_logs(
            &service,
            &user_repository,
            &message_repository,
            game_uuid,
            "1901s",
            None,
            None,
        )
        .expect("should succeed");
        let json = serde_json::to_value(&response).expect("serialize");
        let messages = json["messages"].as_array().expect("messages");

        assert_eq!(messages.len(), 5);
        assert_eq!(messages[0]["kind"], "system");
        assert_eq!(messages[0]["context"], "German A Mun が配置されました。");
        assert_eq!(
            messages[0]["notice"],
            serde_json::json!({
                "type": "unit_placed",
                "params": { "unit": { "power": "g", "kind": "a", "location": "mun" } },
            })
        );
        assert_eq!(
            messages[1]["notice"],
            serde_json::json!({
                "type": "unit_replaced",
                "params": {
                    "old_unit": { "power": "g", "kind": "a", "location": "mun" },
                    "new_unit": { "power": "r", "kind": "f", "location": "stp_sc" },
                },
            })
        );
        assert_eq!(
            messages[2]["notice"],
            serde_json::json!({
                "type": "territory_replaced",
                "params": { "province": "mun", "old_power": "a", "new_power": "g" },
            })
        );
        assert_eq!(
            messages[3]["notice"],
            serde_json::json!({ "type": "progress_consented", "params": { "power": "f" } })
        );
        assert_eq!(messages[4]["kind"], "public");
        assert!(messages[4]["notice"].is_null());
    }

    // ============================================================================
    // apply_log_access_control tests
    // ============================================================================

    fn finished_game() -> Game {
        let owner_uuid = uuid::Uuid::now_v7();
        let player_uuid = uuid::Uuid::now_v7();
        let mut game = sample_in_progress_game_for_logs(owner_uuid, player_uuid);
        game.status = GameStatus::Closed;
        game
    }

    fn in_progress_game() -> Game {
        let owner_uuid = uuid::Uuid::now_v7();
        let player_uuid = uuid::Uuid::now_v7();
        sample_in_progress_game_for_logs(owner_uuid, player_uuid)
    }

    #[test]
    fn access_control_finished_game_returns_all_messages() {
        let game = finished_game();
        let records = vec![
            make_message_record("public", Some(crate::domain::Power::France), vec![]),
            make_message_record(
                "confidential",
                Some(crate::domain::Power::France),
                vec![crate::domain::Power::England],
            ),
            make_message_record("personal", Some(crate::domain::Power::England), vec![]),
            make_message_record("ghost", Some(crate::domain::Power::Austria), vec![]),
            make_message_record("system", None, vec![]),
        ];

        let result = apply_log_access_control(records, &game, None);

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn access_control_in_progress_unauthenticated_public_system_visible() {
        let game = in_progress_game();
        let records = vec![
            make_message_record("public", Some(crate::domain::Power::France), vec![]),
            make_message_record("system", None, vec![]),
        ];

        let result = apply_log_access_control(records, &game, None);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].kind, "public");
        assert_eq!(result[1].kind, "system");
    }

    #[test]
    fn access_control_in_progress_unauthenticated_personal_hidden() {
        let game = in_progress_game();
        let records = vec![make_message_record("personal", Some(crate::domain::Power::France), vec![])];

        let result = apply_log_access_control(records, &game, None);

        assert!(result.is_empty());
    }

    #[test]
    fn access_control_in_progress_unauthenticated_ghost_visible() {
        let game = in_progress_game();
        let records = vec![make_message_record("ghost", Some(crate::domain::Power::France), vec![])];

        let result = apply_log_access_control(records, &game, None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "ghost");
        assert!(result[0].context.is_some());
    }

    #[test]
    fn access_control_in_progress_unauthenticated_confidential_masked() {
        let game = in_progress_game();
        let records = vec![make_message_record(
            "confidential",
            Some(crate::domain::Power::France),
            vec![crate::domain::Power::England],
        )];

        let result = apply_log_access_control(records, &game, None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "masked");
        assert!(result[0].context.is_none());
        assert_eq!(result[0].recipients, Some(vec!["e".to_string()]));
    }

    #[test]
    fn access_control_in_progress_participant_sees_own_personal() {
        let game = in_progress_game();
        let records = vec![make_message_record("personal", Some(crate::domain::Power::France), vec![])];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::France));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "personal");
    }

    #[test]
    fn access_control_in_progress_participant_cannot_see_others_personal() {
        let game = in_progress_game();
        let records = vec![make_message_record("personal", Some(crate::domain::Power::England), vec![])];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::France));

        assert!(result.is_empty());
    }

    #[test]
    fn access_control_in_progress_sender_sees_confidential_full() {
        let game = in_progress_game();
        let records = vec![make_message_record(
            "confidential",
            Some(crate::domain::Power::France),
            vec![crate::domain::Power::England],
        )];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::France));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "confidential");
        assert!(result[0].context.is_some());
    }

    #[test]
    fn access_control_in_progress_recipient_sees_confidential_full() {
        let game = in_progress_game();
        let records = vec![make_message_record(
            "confidential",
            Some(crate::domain::Power::France),
            vec![crate::domain::Power::England],
        )];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::England));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "confidential");
        assert!(result[0].context.is_some());
    }

    #[test]
    fn access_control_in_progress_unrelated_power_sees_confidential_masked() {
        let game = in_progress_game();
        let records = vec![make_message_record(
            "confidential",
            Some(crate::domain::Power::France),
            vec![crate::domain::Power::England],
        )];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::Austria));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "masked");
        assert!(result[0].context.is_none());
    }

    #[test]
    fn access_control_in_progress_participant_ghost_visible_when_not_eliminated() {
        // Austria にユニットを与えて「滅亡していない」状態にする
        let game = game_with_units_for(crate::domain::Power::Austria);
        let records = vec![make_message_record("ghost", Some(crate::domain::Power::France), vec![])];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::Austria));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "ghost");
        assert!(result[0].context.is_some());
    }

    #[test]
    fn access_control_in_progress_ghost_visible_when_eliminated() {
        // England にユニットを与えるが France にはなし → France は滅亡扱い
        let game = game_with_units_for(crate::domain::Power::England);
        let records = vec![make_message_record("ghost", Some(crate::domain::Power::France), vec![])];

        let result = apply_log_access_control(records, &game, Some(crate::domain::Power::France));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].kind, "ghost");
    }

    // ============================================================================
    // is_power_eliminated tests
    // ============================================================================

    fn game_with_units_for(power: crate::domain::Power) -> Game {
        let mut game = in_progress_game();
        let phase = game.phases.last_mut().unwrap();
        let par = crate::domain::Province::from_code("par").unwrap();
        phase.units.push(crate::domain::Unit::new_army(power, par));
        game
    }

    #[test]
    fn is_power_eliminated_false_when_unit_exists() {
        let game = game_with_units_for(crate::domain::Power::France);

        assert!(!is_power_eliminated(crate::domain::Power::France, &game));
    }

    #[test]
    fn is_power_eliminated_true_when_no_unit() {
        let game = game_with_units_for(crate::domain::Power::France);

        assert!(is_power_eliminated(crate::domain::Power::England, &game));
    }

    #[test]
    fn is_power_eliminated_false_when_last_phase_is_ready() {
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
        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        assert!(!is_power_eliminated(crate::domain::Power::France, &game));
    }

    #[test]
    fn is_power_eliminated_false_when_no_unit_but_supply_center_remains() {
        let mut game = game_with_units_for(crate::domain::Power::France);
        let phase = game.phases.last_mut().unwrap();
        phase
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::England, "lon"));

        assert!(!is_power_eliminated(crate::domain::Power::England, &game));
    }

    #[test]
    fn is_power_eliminated_false_when_no_supply_center_but_unit_remains() {
        let mut game = game_with_units_for(crate::domain::Power::France);
        game.phases.last_mut().unwrap().territories.clear();

        assert!(!is_power_eliminated(crate::domain::Power::France, &game));
    }

    #[test]
    fn is_power_eliminated_true_when_only_non_supply_center_land_remains() {
        let mut game = game_with_units_for(crate::domain::Power::France);
        let phase = game.phases.last_mut().unwrap();
        // wal・yor は補給都市でない陸地
        phase
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::England, "wal"));
        phase
            .territories
            .push(crate::domain::Territory::new(crate::domain::Power::England, "yor"));

        assert!(is_power_eliminated(crate::domain::Power::England, &game));
    }

    #[test]
    fn is_power_eliminated_false_when_last_phase_is_debrief() {
        let mut game = in_progress_game();
        game.phases.push(Phase::new_debrief(1901, 2));

        assert!(!is_power_eliminated(crate::domain::Power::France, &game));
    }

    // ============================================================================
    // handle_get_games_board tests
    // ============================================================================

    #[test]
    fn handle_get_games_board_returns_ready_phase_board() {
        let owner_uuid = uuid::Uuid::now_v7();
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.status = GameStatus::Ready;
        game.phases.truncate(1);
        let ready = game.phases[0].clone();
        let game_uuid = game.uuid;
        let service = GameService::new(
            InMemoryUserRepository::new(vec![]),
            InMemoryGameRepository::new_with_games(vec![game]),
        );

        let response = handle_get_games_board(&service, game_uuid, None).expect("should succeed");

        assert_eq!(response.season, "ready");
        assert_eq!(response.phase_kind, "ready");
        assert_eq!(response.units.len(), ready.units.len());
        assert_eq!(response.territories.len(), ready.territories.len());
        // 補給都市 22 か所 + 補給都市でない本国の陸地 20 か所
        assert_eq!(response.territories.len(), 42);
        assert!(response.territories.contains(&GameBoardTerritoryResponse {
            power: "t".to_string(),
            code: "syr".to_string(),
        }));
        assert!(response.units.contains(&GameBoardUnitResponse {
            power: "f".to_string(),
            kind: "a".to_string(),
            location: "par".to_string(),
            dislodged: false,
            dislodged_from: None,
        }));
        assert!(response.units.contains(&GameBoardUnitResponse {
            power: "r".to_string(),
            kind: "f".to_string(),
            location: "stp_sc".to_string(),
            dislodged: false,
            dislodged_from: None,
        }));
        assert!(response.territories.contains(&GameBoardTerritoryResponse {
            power: "f".to_string(),
            code: "par".to_string(),
        }));
    }

    #[test]
    fn handle_get_games_board_returns_latest_main_phase_board_without_orders() {
        let owner_uuid = uuid::Uuid::now_v7();
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        let phase = game.phases.last_mut().unwrap();
        let spa_nc = crate::domain::Province::from_code("spa_nc").unwrap();
        let fleet = crate::domain::Unit::new_fleet(crate::domain::Power::France, spa_nc);
        phase.units = vec![fleet];
        phase.orders = vec![fleet.hold()];
        phase.territories = vec![
            crate::domain::Territory::new(crate::domain::Power::France, "spa"),
            crate::domain::Territory::new(crate::domain::Power::France, "gas"),
        ];
        let game_uuid = game.uuid;
        let service = GameService::new(
            InMemoryUserRepository::new(vec![]),
            InMemoryGameRepository::new_with_games(vec![game]),
        );

        let response = handle_get_games_board(&service, game_uuid, None).expect("should succeed");

        assert_eq!(response.season, "1901s");
        assert_eq!(response.phase_kind, "main");
        assert_eq!(
            response.units,
            vec![GameBoardUnitResponse {
                power: "f".to_string(),
                kind: "f".to_string(),
                location: "spa_nc".to_string(),
                dislodged: false,
                dislodged_from: None,
            }]
        );
        assert_eq!(
            response.territories,
            vec![
                GameBoardTerritoryResponse {
                    power: "f".to_string(),
                    code: "spa".to_string(),
                },
                GameBoardTerritoryResponse {
                    power: "f".to_string(),
                    code: "gas".to_string(),
                },
            ]
        );
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("orders").is_none());
    }

    #[test]
    fn handle_get_games_board_returns_not_found_for_unknown_game() {
        let service = GameService::new(InMemoryUserRepository::new(vec![]), InMemoryGameRepository::new());

        let error = handle_get_games_board(&service, uuid::Uuid::now_v7(), None).expect_err("should fail");

        assert_eq!(error.code(), "not_found");
    }

    #[test]
    fn handle_set_unit_and_territory_accept_ready_season() {
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
            is_admin: false,
        }]);
        let mut game = sample_in_progress_game_for_handler(owner_uuid);
        game.status = GameStatus::Ready;
        game.phases.truncate(1);
        let game_uuid = game.uuid;
        let service = GameService::new(user_repository, InMemoryGameRepository::new_with_games(vec![game]));
        let message_repository = new_test_message_repository();

        let unit_response = handle_set_unit(
            &service,
            &message_repository,
            SetUnitRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                unit: None,
                location: "par".to_string(),
                season: "ready".to_string(),
            },
        )
        .expect("ready season should be accepted for units");
        assert!(unit_response.unit.is_none());

        let territory_response = handle_set_territory(
            &service,
            &message_repository,
            SetTerritoryRequest {
                authorization: "Bearer token-owner".to_string(),
                game_uuid,
                code: "par".to_string(),
                power: None,
                season: "READY".to_string(),
            },
        )
        .expect("ready season should be accepted for territories");
        assert!(territory_response.power.is_none());
    }

    #[test]
    fn utc_rfc3339_to_jst_string_converts_correctly() {
        let result = utc_rfc3339_to_jst_string("2026-05-10T11:00:00Z");
        assert_eq!(result, "2026-05-10 20:00");
    }

    #[test]
    fn utc_rfc3339_to_jst_string_returns_original_on_invalid_input() {
        let result = utc_rfc3339_to_jst_string("not-a-date");
        assert_eq!(result, "not-a-date");
    }

    // ---- handle_get_games_result テスト ----

    use crate::api::handlers::GetGameResultHandlerError;
    use crate::domain::FaceType;
    use crate::domain::ProgressMode;
    use crate::domain::Regulation;

    fn test_regulation() -> Regulation {
        Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            crate::domain::DurationType::Normal,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            21,
        )
        .unwrap()
    }

    fn make_game_for_result(uuid: uuid::Uuid, phases: Vec<Phase>) -> Game {
        Game {
            uuid,
            game_number: None,
            keyword: None,
            regulation: test_regulation(),
            players: vec![],
            phases,
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        }
    }

    fn make_result_service(game: Game) -> (uuid::Uuid, GameService<InMemoryUserRepository, InMemoryGameRepository>) {
        let uuid = game.uuid;
        let service = GameService::new(
            InMemoryUserRepository::new(vec![]),
            InMemoryGameRepository::new_with_games(vec![game]),
        );
        (uuid, service)
    }

    fn army(power: crate::domain::Power, code: &str) -> crate::domain::Unit {
        let province = crate::domain::Province::from_code(code).expect("valid province code");
        crate::domain::Unit::new_army(power, province)
    }

    fn province(code: &str) -> crate::domain::Province {
        crate::domain::Province::from_code(code).expect("valid province code")
    }

    #[test]
    fn handle_get_games_result_returns_open_main_phase_for_current_spring_season() {
        let ready = Phase::new_ready();
        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        let eng_army = army(crate::domain::Power::England, "lon");
        spring_main.units = vec![eng_army];
        spring_main.orders = vec![crate::Order::new_hold(crate::domain::Power::England, eng_army)];

        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        assert_eq!(result.main_phase.status, "open");
        assert!(result.retreat_phase.is_none());
        assert!(result.adjustment_phase.is_none());
        // 進行中のフェイズの命令は返さない
        assert!(result.main_phase.orders.is_empty());
        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].location, "lon");
    }

    #[test]
    fn handle_get_games_result_returns_closed_phases_for_historical_spring_season() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let fall_main = Phase::new_fall_main(spring_retreat.year, spring_retreat.index);

        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat, fall_main]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        assert_eq!(result.main_phase.status, "closed");
        assert_eq!(result.retreat_phase.as_ref().unwrap().status, "closed");
        assert!(result.adjustment_phase.is_none());
    }

    #[test]
    fn handle_get_games_result_returns_retreat_phase_as_open_when_current() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);

        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        assert_eq!(result.main_phase.status, "closed");
        assert_eq!(result.retreat_phase.as_ref().unwrap().status, "open");
        assert!(result.adjustment_phase.is_none());
    }

    #[test]
    fn handle_get_games_result_includes_all_fall_phases_when_complete() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let fall_main = Phase::new_fall_main(spring_retreat.year, spring_retreat.index);
        let fall_retreat = Phase::new_fall_retreat(fall_main.year, fall_main.index);
        let adjustment = Phase::new_adjustment(fall_retreat.year, fall_retreat.index);

        let game = make_game_for_result(
            uuid::Uuid::now_v7(),
            vec![ready, spring_main, spring_retreat, fall_main, fall_retreat, adjustment],
        );
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901f").unwrap();

        assert_eq!(result.main_phase.status, "closed");
        assert_eq!(result.retreat_phase.as_ref().unwrap().status, "closed");
        assert_eq!(result.adjustment_phase.as_ref().unwrap().status, "open");
    }

    #[test]
    fn handle_get_games_result_excludes_assumed_orders() {
        let ready = Phase::new_ready();
        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        let eng_army = army(crate::domain::Power::England, "lon");

        let real_order = crate::Order::new_hold(crate::domain::Power::England, eng_army);
        // France が England のユニットに仮想命令を持つ → is_assumed() = true
        let assumed_order = crate::Order::new_hold(crate::domain::Power::France, eng_army);
        spring_main.orders = vec![real_order, assumed_order];

        // メインフェイズを締め切り済みにするため、後続のフェイズを置く
        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        assert_eq!(result.main_phase.orders.len(), 1);
        assert_eq!(result.main_phase.orders[0].power, "e");
    }

    #[test]
    fn handle_get_games_result_returns_units_from_last_phase_of_season() {
        let ready = Phase::new_ready();
        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        let mut spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);

        spring_main.units = vec![army(crate::domain::Power::England, "lon")];
        spring_retreat.units = vec![army(crate::domain::Power::England, "wal")]; // 撤退後にウェールズへ移動

        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        assert_eq!(result.units.len(), 1);
        assert_eq!(result.units[0].location, "wal");
    }

    #[test]
    fn handle_get_games_result_includes_move_order_fields() {
        let ready = Phase::new_ready();
        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        let eng_army = army(crate::domain::Power::England, "lon");
        let dest = province("wal");
        spring_main.orders = vec![crate::Order::new_move(crate::domain::Power::England, eng_army, dest)];

        // メインフェイズを締め切り済みにするため、後続のフェイズを置く
        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        let order = &result.main_phase.orders[0];
        assert_eq!(order.order_kind, "move");
        assert_eq!(order.dest.as_deref(), Some("wal"));
        assert_eq!(order.via_convoy, Some(false));
    }

    #[test]
    fn handle_get_games_result_uses_board_province_codes() {
        let ready = Phase::new_ready();
        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        let stp = crate::domain::Unit::new_fleet(crate::domain::Power::Russia, province("stp_sc"));
        let fin = army(crate::domain::Power::Russia, "fin");
        let mid = crate::domain::Unit::new_fleet(crate::domain::Power::France, province("mid"));
        let gas = army(crate::domain::Power::France, "gas");
        spring_main.units = vec![stp, fin, mid, gas];
        spring_main.orders = vec![
            crate::Order::new_move(crate::domain::Power::Russia, stp, province("bot")),
            crate::Order::new_support(crate::domain::Power::Russia, fin, stp, Some(province("bot"))),
            crate::Order::new_move(crate::domain::Power::France, mid, province("spa_nc")),
            crate::Order::new_support(crate::domain::Power::France, gas, mid, Some(province("spa_nc"))),
        ];
        let mut spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        spring_retreat.units = spring_main.units.clone();

        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat]);
        let (uuid, service) = make_result_service(game);
        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        let orders = &result.main_phase.orders;
        assert_eq!(orders[0].location, "stp_sc");
        assert_eq!(orders[0].dest.as_deref(), Some("bot"));
        assert_eq!(orders[1].location, "fin");
        assert_eq!(orders[1].target_location.as_deref(), Some("stp_sc"));
        assert_eq!(orders[1].target_dest.as_deref(), Some("bot"));
        assert_eq!(orders[2].dest.as_deref(), Some("spa_nc"));
        assert_eq!(orders[3].target_dest.as_deref(), Some("spa_nc"));
        assert!(result.units.iter().any(|u| u.location == "stp_sc" && u.kind == "f"));
    }

    #[test]
    fn handle_get_games_result_hides_orders_of_open_retreat_and_adjustment_phases() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let mut spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let mut dislodged = army(crate::domain::Power::England, "lon");
        dislodged.set_dislodged_from(Some(province("wal")));
        spring_retreat.units = vec![dislodged];
        spring_retreat.orders = vec![crate::Order::new_retreat(
            crate::domain::Power::England,
            dislodged,
            province("yor"),
        )];

        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready.clone(), spring_main.clone(), spring_retreat]);
        let (uuid, service) = make_result_service(game);
        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();
        assert_eq!(result.retreat_phase.as_ref().unwrap().status, "open");
        assert!(result.retreat_phase.as_ref().unwrap().orders.is_empty());

        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let fall_main = Phase::new_fall_main(spring_retreat.year, spring_retreat.index);
        let fall_retreat = Phase::new_fall_retreat(fall_main.year, fall_main.index);
        let mut adjustment = Phase::new_adjustment(fall_retreat.year, fall_retreat.index);
        adjustment.orders = vec![crate::Order::new_build(
            crate::domain::Power::England,
            army(crate::domain::Power::England, "lon"),
        )];
        let game = make_game_for_result(
            uuid::Uuid::now_v7(),
            vec![ready, spring_main, spring_retreat, fall_main, fall_retreat, adjustment],
        );
        let (uuid, service) = make_result_service(game);
        let result = handle_get_games_result(&service, uuid, "1901f").unwrap();
        assert_eq!(result.adjustment_phase.as_ref().unwrap().status, "open");
        assert!(result.adjustment_phase.as_ref().unwrap().orders.is_empty());
    }

    #[test]
    fn handle_get_games_result_returns_not_found_for_missing_game() {
        let service = GameService::new(InMemoryUserRepository::new(vec![]), InMemoryGameRepository::new());

        let result = handle_get_games_result(&service, uuid::Uuid::now_v7(), "1901s");

        assert!(matches!(result, Err(GetGameResultHandlerError::GameNotFound)));
    }

    #[test]
    fn handle_get_games_result_returns_not_found_for_ready_season() {
        let ready = Phase::new_ready();
        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "ready");

        assert!(matches!(result, Err(GetGameResultHandlerError::SeasonNotFound)));
    }

    #[test]
    fn handle_get_games_result_returns_not_found_for_debrief_season() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let debrief = Phase::new_debrief(spring_main.year, spring_main.index);
        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, debrief]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "debrief");

        assert!(matches!(result, Err(GetGameResultHandlerError::SeasonNotFound)));
    }

    #[test]
    fn handle_get_games_result_returns_not_found_for_unreached_season() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main]);
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901f");

        assert!(matches!(result, Err(GetGameResultHandlerError::SeasonNotFound)));
    }

    #[test]
    fn handle_get_games_result_spring_has_no_adjustment_phase() {
        let ready = Phase::new_ready();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        let spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        let fall_main = Phase::new_fall_main(spring_retreat.year, spring_retreat.index);
        let fall_retreat = Phase::new_fall_retreat(fall_main.year, fall_main.index);
        let adjustment = Phase::new_adjustment(fall_retreat.year, fall_retreat.index);

        let game = make_game_for_result(
            uuid::Uuid::now_v7(),
            vec![ready, spring_main, spring_retreat, fall_main, fall_retreat, adjustment],
        );
        let (uuid, service) = make_result_service(game);

        let result = handle_get_games_result(&service, uuid, "1901s").unwrap();

        assert!(result.adjustment_phase.is_none());
    }

    // ============================================================================
    // 命令取得・確定（GET / PUT /games/:game_uuid/orders）と盤面の国ごとの状況
    // ============================================================================

    use crate::api::handlers::order_handler::handle_get_orders;
    use crate::api::handlers::order_handler::handle_put_orders;
    use crate::services::GetOrdersCommand;
    use crate::services::OrderSpec;
    use crate::services::OrdersError;
    use crate::services::SubmitOrdersCommand;

    fn orders_user(id: i64, name: &str, last_access_at: DateTime<Utc>, is_admin: bool) -> UserRecord {
        UserRecord {
            id,
            uuid: uuid::Uuid::now_v7(),
            discord_user_id: format!("discord-{}", name),
            username: name.to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: format!("token-{}", name),
            last_access_at,
            is_admin,
        }
    }

    struct OrdersFixture {
        service: GameService<InMemoryUserRepository, InMemoryGameRepository>,
        game_repository: InMemoryGameRepository,
        user_repository: InMemoryUserRepository,
        game_uuid: uuid::Uuid,
    }

    /// フランス・イギリス・ドイツ（無政府）の 3 人が参加し、指定したフェイズが最新の卓を作る
    fn orders_fixture(latest: Phase) -> OrdersFixture {
        let france = orders_user(1, "france", Utc::now(), false);
        let england = orders_user(2, "england", Utc::now(), false);
        let germany = orders_user(3, "germany", Utc::now() - chrono::Duration::days(30), false);
        let outsider = orders_user(4, "outsider", Utc::now(), false);
        let admin = orders_user(5, "admin", Utc::now(), true);
        let player = |user: &UserRecord, power: Power| Player {
            user_uuid: user.uuid,
            power: Some(power),
            is_accepting_draw: false,
            progress_consented: false,
            is_idle: false,
            is_owner: power == Power::France,
            requested_power: None,
        };
        let mut game = make_game_for_result(uuid::Uuid::now_v7(), vec![Phase::new_ready(), latest]);
        game.players = vec![
            player(&france, Power::France),
            player(&england, Power::England),
            player(&germany, Power::Germany),
        ];
        game.players[1].progress_consented = true;
        let game_uuid = game.uuid;
        let user_repository = InMemoryUserRepository::new(vec![france, england, germany, outsider, admin]);
        let game_repository = InMemoryGameRepository::new_with_games(vec![game]);
        OrdersFixture {
            service: GameService::new(user_repository.clone(), game_repository.clone()),
            game_repository,
            user_repository,
            game_uuid,
        }
    }

    fn orders_main_phase() -> Phase {
        let ready = Phase::new_ready();
        let mut phase = Phase::new_spring_main(ready.year, ready.index);
        let par = army(Power::France, "par");
        let bre = crate::domain::Unit::new_fleet(Power::France, province("bre"));
        let lon = army(Power::England, "lon");
        let nth = crate::domain::Unit::new_fleet(Power::England, province("nth"));
        let mun = army(Power::Germany, "mun");
        phase.units = vec![par, bre, lon, nth, mun];
        phase.territories = vec![
            crate::domain::Territory::new(Power::France, "par"),
            crate::domain::Territory::new(Power::France, "bre"),
            crate::domain::Territory::new(Power::England, "lon"),
            crate::domain::Territory::new(Power::Germany, "mun"),
        ];
        phase.orders = phase.units.iter().map(|u| u.hold()).collect();
        // イギリスの仮定命令（フランスには見えない）
        phase.orders.push(par.move_to(province("bur")).assumed_by(Power::England));
        // フランスの仮定命令
        phase.orders.push(mun.move_to(province("bur")).assumed_by(Power::France));
        phase
    }

    fn get_orders_as(
        fixture: &OrdersFixture,
        token: &str,
    ) -> Result<crate::api::handlers::order_handler::OrdersResponse, OrdersError> {
        handle_get_orders(
            &fixture.service,
            GetOrdersCommand {
                access_token: token.to_string(),
                game_uuid: fixture.game_uuid,
            },
        )
    }

    fn spec(unit_power: &str, unit_kind: &str, location: &str, order_kind: &str) -> OrderSpec {
        OrderSpec {
            unit_power: unit_power.to_string(),
            unit_kind: unit_kind.to_string(),
            location: location.to_string(),
            order_kind: order_kind.to_string(),
            ..OrderSpec::default()
        }
    }

    fn put_orders_as(
        fixture: &OrdersFixture,
        token: &str,
        season: &str,
        phase_kind: &str,
        orders: Vec<OrderSpec>,
    ) -> Result<crate::api::handlers::order_handler::OrdersResponse, OrdersError> {
        handle_put_orders(
            &fixture.service,
            SubmitOrdersCommand {
                access_token: token.to_string(),
                game_uuid: fixture.game_uuid,
                season: season.to_string(),
                phase_kind: phase_kind.to_string(),
                orders,
            },
        )
    }

    #[test]
    fn get_orders_returns_own_orders_and_own_assumed_orders_only() {
        let fixture = orders_fixture(orders_main_phase());

        let response = get_orders_as(&fixture, "token-france").expect("should succeed");

        assert_eq!(response.season, "1901s");
        assert_eq!(response.phase_kind, "main");
        assert_eq!(response.power, "f");
        assert!(response.needs_action);
        let orders: Vec<(String, String, String, Option<String>)> = response
            .orders
            .iter()
            .map(|o| (o.unit_power.clone(), o.location.clone(), o.order_kind.clone(), o.dest.clone()))
            .collect();
        assert_eq!(
            orders,
            vec![
                ("f".to_string(), "par".to_string(), "hold".to_string(), None),
                ("f".to_string(), "bre".to_string(), "hold".to_string(), None),
                (
                    "g".to_string(),
                    "mun".to_string(),
                    "move".to_string(),
                    Some("bur".to_string())
                ),
            ]
        );
        // 候補は盤面の全ユニット分
        assert_eq!(response.options.len(), 5);
        let lon = response.options.iter().find(|o| o.location == "lon").expect("lon options");
        assert!(lon.convoy_dests.as_ref().unwrap().contains(&"nwy".to_string()));
        assert!(lon.retreat_dests.is_none());
        let par = response.options.iter().find(|o| o.location == "par").expect("par options");
        assert!(par.convoy_dests.as_ref().unwrap().is_empty());
        let nth = response.options.iter().find(|o| o.location == "nth").expect("nth options");
        assert!(
            nth.convoy_targets
                .as_ref()
                .unwrap()
                .iter()
                .any(|c| c.target_location == "lon" && c.target_dests.contains(&"bel".to_string()))
        );
        assert!(response.adjustment.is_none());

        let json = serde_json::to_value(&response).expect("serialize");
        assert_eq!(json["orders"][0]["unit_kind"], "a");
        assert!(json["orders"][0].get("status").is_none());
        assert!(json["orders"][0].get("dest").is_none());
    }

    #[test]
    fn get_orders_rejects_non_participants_and_missing_token() {
        let fixture = orders_fixture(orders_main_phase());

        let error = get_orders_as(&fixture, "token-outsider").expect_err("should fail");
        assert!(matches!(error, OrdersError::Forbidden(_)));

        let error = get_orders_as(&fixture, "").expect_err("should fail");
        assert_eq!(error, OrdersError::Unauthorized);
    }

    #[test]
    fn get_orders_rejects_debrief_phase() {
        let fixture = orders_fixture(Phase::new_debrief(1901, 1));
        let error = get_orders_as(&fixture, "token-france").expect_err("should fail");
        assert!(matches!(error, OrdersError::Forbidden(_)));
    }

    #[test]
    fn get_orders_works_with_impersonated_token() {
        let fixture = orders_fixture(orders_main_phase());

        // 管理者が成り代わると、ミドルウェアは切り替え先のトークンに差し替える
        let token =
            crate::api::router::resolve_impersonated_access_token(&fixture.user_repository, "token-admin", Some("england"))
                .expect("should resolve")
                .expect("admin can impersonate");
        let response = get_orders_as(&fixture, &token).expect("should succeed");
        assert_eq!(response.power, "e");
        // イギリスの仮定命令が見える
        assert!(response.orders.iter().any(|o| o.unit_power == "f" && o.location == "par"));

        // 切り替え先が参加者なら命令も確定できる
        let mut order = spec("e", "a", "lon", "move");
        order.dest = Some("wal".to_string());
        let response = put_orders_as(&fixture, &token, "1901s", "main", vec![order]).expect("should succeed");
        assert!(
            response
                .orders
                .iter()
                .any(|o| o.location == "lon" && o.dest.as_deref() == Some("wal"))
        );

        // 成り代わらない管理者は参加者でないので使えない
        let error = get_orders_as(&fixture, "token-admin").expect_err("should fail");
        assert!(matches!(error, OrdersError::Forbidden(_)));
    }

    #[test]
    fn put_orders_replaces_all_orders_of_the_power() {
        let fixture = orders_fixture(orders_main_phase());

        let mut bre = spec("f", "f", "bre", "move");
        bre.dest = Some("eng".to_string());
        let mut lon = spec("e", "a", "lon", "move");
        lon.dest = Some("bel".to_string());
        lon.via_convoy = Some(true);
        let response = put_orders_as(&fixture, "token-france", "1901s", "main", vec![bre, lon]).expect("should succeed");

        let orders: Vec<(String, String, String)> = response
            .orders
            .iter()
            .map(|o| (o.unit_power.clone(), o.location.clone(), o.order_kind.clone()))
            .collect();
        // 含まれていない自国ユニット（パリ）は維持、含まれていない仮定命令（ミュンヘン）は消える
        assert_eq!(
            orders,
            vec![
                ("f".to_string(), "par".to_string(), "hold".to_string()),
                ("f".to_string(), "bre".to_string(), "move".to_string()),
                ("e".to_string(), "lon".to_string(), "move".to_string()),
            ]
        );

        let saved = fixture.game_repository.last_updated().expect("game should be updated");
        let phase = saved.phases.last().unwrap();
        // イギリスの仮定命令と、ほかの国の命令は残る
        assert!(phase.orders.iter().any(|o| o.power == Power::England && o.is_assumed()));
        assert!(phase.orders.iter().any(|o| o.power == Power::Germany && !o.is_assumed()));
        // 仮定命令は命令列の末尾にまとまる
        let first_assumed = phase.orders.iter().position(|o| o.is_assumed()).unwrap();
        assert!(phase.orders[first_assumed..].iter().all(|o| o.is_assumed()));
    }

    #[test]
    fn put_orders_rejects_unavailable_order_without_changes() {
        let fixture = orders_fixture(orders_main_phase());

        let mut bre = spec("f", "f", "bre", "move");
        bre.dest = Some("eng".to_string());
        let mut par = spec("f", "a", "par", "move");
        par.dest = Some("mun".to_string());
        let error = put_orders_as(&fixture, "token-france", "1901s", "main", vec![bre, par]).expect_err("should fail");

        match error {
            OrdersError::InvalidOrder { location, .. } => assert_eq!(location.as_deref(), Some("par")),
            other => panic!("unexpected error: {:?}", other),
        }
        assert!(fixture.game_repository.last_updated().is_none());
    }

    #[test]
    fn put_orders_rejects_stale_phase() {
        let fixture = orders_fixture(orders_main_phase());

        let error = put_orders_as(&fixture, "token-france", "1901s", "retreat", vec![]).expect_err("should fail");
        assert_eq!(error, OrdersError::PhaseConflict);
        let error = put_orders_as(&fixture, "token-france", "1901f", "main", vec![]).expect_err("should fail");
        assert_eq!(error, OrdersError::PhaseConflict);
        assert!(fixture.game_repository.last_updated().is_none());
    }

    #[test]
    fn put_orders_in_retreat_phase_defaults_to_disband() {
        let ready = Phase::new_ready();
        let main = Phase::new_spring_main(ready.year, ready.index);
        let mut phase = Phase::new_spring_retreat(main.year, main.index);
        let mut bur = army(Power::France, "bur");
        bur.set_dislodged_from(Some(province("mun")));
        let mut bre = crate::domain::Unit::new_fleet(Power::France, province("bre"));
        bre.set_dislodged_from(Some(province("eng")));
        phase.units = vec![
            bur,
            bre,
            army(Power::Germany, "bur"),
            crate::domain::Unit::new_fleet(Power::England, province("bre")),
        ];
        phase.territories = vec![crate::domain::Territory::new(Power::France, "par")];
        phase.standoff_codes = vec!["pic".to_string()];
        phase.orders = vec![bur.disband(), bre.disband()];
        let fixture = orders_fixture(phase);

        let response = get_orders_as(&fixture, "token-france").expect("should succeed");
        assert_eq!(response.phase_kind, "retreat");
        let bur_options = response.options.iter().find(|o| o.location == "bur").expect("bur options");
        assert_eq!(
            bur_options.retreat_dests.as_ref().unwrap(),
            &vec![
                "bel".to_string(),
                "gas".to_string(),
                "mar".to_string(),
                "par".to_string(),
                "ruh".to_string()
            ]
        );
        assert!(bur_options.move_dests.is_none());
        // ほかの国のユニットの候補は出さない
        assert_eq!(response.options.len(), 2);

        let mut retreat = spec("f", "a", "bur", "retreat");
        retreat.dest = Some("gas".to_string());
        let response = put_orders_as(&fixture, "token-france", "1901s", "retreat", vec![retreat]).expect("should succeed");
        let orders: Vec<(String, String)> = response
            .orders
            .iter()
            .map(|o| (o.location.clone(), o.order_kind.clone()))
            .collect();
        assert_eq!(
            orders,
            vec![
                ("bur".to_string(), "retreat".to_string()),
                ("bre".to_string(), "disband".to_string())
            ]
        );

        // 操作が要らない国（イギリス）は確定できない
        let error = put_orders_as(&fixture, "token-england", "1901s", "retreat", vec![]).expect_err("should fail");
        assert!(matches!(error, OrdersError::Forbidden(_)));
    }

    #[test]
    fn put_orders_in_adjustment_phase_validates_counts() {
        let ready = Phase::new_ready();
        let mut phase = Phase::new_adjustment(1901, ready.index + 4);
        phase.units = vec![
            army(Power::France, "par"),
            army(Power::England, "lon"),
            army(Power::England, "wal"),
        ];
        phase.territories = vec![
            crate::domain::Territory::new(Power::France, "par"),
            crate::domain::Territory::new(Power::France, "bre"),
            crate::domain::Territory::new(Power::France, "mar"),
            crate::domain::Territory::new(Power::England, "lon"),
        ];
        phase.orders = vec![army(Power::England, "wal").disband()];
        let fixture = orders_fixture(phase);

        let response = get_orders_as(&fixture, "token-france").expect("should succeed");
        let adjustment = response.adjustment.as_ref().expect("adjustment exists");
        assert_eq!(adjustment.build_count, 2);
        assert_eq!(adjustment.disband_count, 0);
        let bre = adjustment
            .buildable
            .iter()
            .find(|b| b.location == "bre")
            .expect("bre buildable");
        assert_eq!(bre.unit_kinds, vec!["a".to_string(), "f".to_string()]);
        assert!(!adjustment.buildable.iter().any(|b| b.location == "par"));

        // 増設が多すぎる
        let error = put_orders_as(
            &fixture,
            "token-france",
            "1901f",
            "adjustment",
            vec![
                spec("f", "f", "bre", "build"),
                spec("f", "a", "mar", "build"),
                spec("f", "a", "par", "build"),
            ],
        )
        .expect_err("should fail");
        assert!(matches!(error, OrdersError::InvalidOrder { .. }));

        let response = put_orders_as(
            &fixture,
            "token-france",
            "1901f",
            "adjustment",
            vec![spec("f", "f", "bre", "build")],
        )
        .expect("should succeed");
        assert_eq!(response.orders.len(), 1);
        assert_eq!(response.orders[0].order_kind, "build");

        // 解体の数が合わない
        let error = put_orders_as(&fixture, "token-england", "1901f", "adjustment", vec![]).expect_err("should fail");
        assert!(matches!(error, OrdersError::InvalidOrder { location: None, .. }));
        let response = put_orders_as(
            &fixture,
            "token-england",
            "1901f",
            "adjustment",
            vec![spec("e", "a", "lon", "disband")],
        )
        .expect("should succeed");
        assert_eq!(response.orders[0].location, "lon");
    }

    #[test]
    fn handle_get_games_board_returns_power_statuses() {
        let ready = Phase::new_ready();
        let main = Phase::new_spring_main(ready.year, ready.index);
        let mut retreat = Phase::new_spring_retreat(main.year, main.index);
        let mut bur = army(Power::France, "bur");
        bur.set_dislodged_from(Some(province("mun")));
        let mut bre = crate::domain::Unit::new_fleet(Power::France, province("bre"));
        bre.set_dislodged_via_convoy();
        retreat.units = vec![bur, bre, army(Power::Germany, "bur"), army(Power::England, "bre")];
        retreat.territories = vec![
            crate::domain::Territory::new(Power::France, "par"),
            crate::domain::Territory::new(Power::England, "lon"),
            crate::domain::Territory::new(Power::England, "edi"),
            crate::domain::Territory::new(Power::Germany, "mun"),
        ];
        let fixture = orders_fixture(retreat);
        // イギリスの担当者は 42 分 30 秒前に最終アクセスした
        fixture
            .user_repository
            .update_last_access_at_by_access_token("token-england", Utc::now() - chrono::Duration::seconds(42 * 60 + 30))
            .expect("update last access");

        // 認証なしで取得する
        let response = handle_get_games_board(&fixture.service, fixture.game_uuid, None).expect("should succeed");

        assert_eq!(response.season, "1901s");
        assert_eq!(response.phase_kind, "retreat");
        let unit = |location: &str, power: &str| {
            response
                .units
                .iter()
                .find(|u| u.location == location && u.power == power)
                .expect("unit exists")
                .clone()
        };
        assert!(unit("bur", "f").dislodged);
        assert_eq!(unit("bur", "f").dislodged_from.as_deref(), Some("mun"));
        assert!(unit("bre", "f").dislodged);
        assert_eq!(unit("bre", "f").dislodged_from, None);
        assert!(!unit("bur", "g").dislodged);

        assert_eq!(response.powers.len(), 7);
        let status = |power: &str| response.powers.iter().find(|p| p.power == power).expect("power").clone();
        let france = status("f");
        assert_eq!((france.supply_centers, france.armies, france.fleets), (1, 1, 1));
        assert!(france.needs_action);
        assert_eq!(france.progress_consented, Some(false));
        assert_eq!(france.is_idle, Some(false));
        assert_eq!(france.absent_minutes, Some(0));
        assert!(!france.is_eliminated);
        let england = status("e");
        assert_eq!(england.supply_centers, 2);
        assert!(!england.needs_action);
        assert_eq!(england.progress_consented, Some(true));
        assert_eq!(england.is_idle, Some(false));
        // 経過分数は切り捨て
        assert_eq!(england.absent_minutes, Some(42));
        // 担当者が無政府（30 日前に最終アクセス）
        let germany = status("g");
        assert_eq!(germany.is_idle, Some(true));
        assert_eq!(germany.absent_minutes, Some(30 * 24 * 60));
        // 担当者がいない国
        let italy = status("i");
        assert_eq!(italy.is_idle, Some(true));
        assert_eq!(italy.absent_minutes, None);
        assert!(italy.is_eliminated);
    }

    #[test]
    fn handle_get_games_board_returns_null_absent_minutes_for_past_phase() {
        let fixture = orders_fixture(orders_main_phase());
        let mut game = fixture.service.get_game(fixture.game_uuid).expect("game");
        let spring = game.phases.last().expect("phase").clone();
        let retreat = Phase::new_spring_retreat(spring.year, spring.index);
        let fall = Phase::new_fall_main(retreat.year, retreat.index);
        game.phases.push(retreat);
        game.phases.push(fall);
        fixture.game_repository.update(&game).expect("update game");

        // 過去のシーズンは担当者がいても null
        let response = handle_get_games_board(&fixture.service, fixture.game_uuid, Some("1901s")).expect("should succeed");
        assert!(response.powers.iter().all(|p| p.absent_minutes.is_none()));

        // 最新のシーズンは担当者のいる国だけ値が入る
        let response = handle_get_games_board(&fixture.service, fixture.game_uuid, None).expect("should succeed");
        let absent = |power: &str| {
            response
                .powers
                .iter()
                .find(|p| p.power == power)
                .expect("power")
                .absent_minutes
        };
        assert_eq!(absent("f"), Some(0));
        assert_eq!(absent("e"), Some(0));
        assert!(absent("g").is_some());
        assert_eq!(absent("i"), None);
    }

    #[test]
    fn handle_get_games_board_returns_past_season() {
        let ready = Phase::new_ready();
        let mut spring_main = Phase::new_spring_main(ready.year, ready.index);
        spring_main.units = vec![army(Power::France, "par")];
        let mut spring_retreat = Phase::new_spring_retreat(spring_main.year, spring_main.index);
        spring_retreat.units = vec![army(Power::France, "bur")];
        spring_retreat.territories = vec![crate::domain::Territory::new(Power::France, "par")];
        let mut fall_main = Phase::new_fall_main(spring_retreat.year, spring_retreat.index);
        fall_main.units = vec![army(Power::France, "mar")];
        let mut game = make_game_for_result(uuid::Uuid::now_v7(), vec![ready, spring_main, spring_retreat, fall_main]);
        game.players = vec![];
        let (uuid, service) = make_result_service(game);

        let response = handle_get_games_board(&service, uuid, Some("1901s")).expect("should succeed");
        assert_eq!(response.season, "1901s");
        // シーズンの最後のフェイズ（撤退フェイズ）の盤面
        assert_eq!(response.phase_kind, "retreat");
        assert_eq!(response.units.len(), 1);
        assert_eq!(response.units[0].location, "bur");
        let france = response.powers.iter().find(|p| p.power == "f").expect("france");
        assert_eq!(france.supply_centers, 1);
        assert_eq!(france.progress_consented, None);
        assert_eq!(france.is_idle, None);
        assert_eq!(france.absent_minutes, None);
        assert!(!france.needs_action);

        let response = handle_get_games_board(&service, uuid, Some("1901f")).expect("should succeed");
        assert_eq!(response.units[0].location, "mar");
        assert!(response.powers.iter().all(|p| p.is_idle.is_some()));

        let error = handle_get_games_board(&service, uuid, Some("1905s")).expect_err("should fail");
        assert!(matches!(error, GetGameResultHandlerError::SeasonNotFound));
    }
}
