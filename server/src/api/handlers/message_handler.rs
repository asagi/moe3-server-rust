// ============================================================================
// imports
// ============================================================================

use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::response::sse::Event;
use axum::response::sse::KeepAlive;
use axum::response::sse::Sse;
use futures_util::stream;
use serde::Serialize;
use strum::IntoEnumIterator;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::ApiErrorResponse;
use super::AppState;
use super::DiscordIdentityProvider;
use super::GameLogMessage;
use super::GameRepository;
use super::GameService;
use super::GameStatus;
use super::GetGameMessagesStreamQueryParams;
use super::PostGameMessageHandlerError;
use super::PostGameMessageRequestBody;
use super::Power;
use super::RepositoryError;
use super::SqliteMessageRepository;
use super::UserRepository;
use super::game_handler::apply_log_access_control;
use super::game_handler::attach_sender_users;
use super::game_handler::is_power_eliminated;
use super::game_handler::optional_bearer_token;
use super::game_handler::to_game_log_message_full;

// ============================================================================
// definitions
// ============================================================================

/// 発言本文の最大文字数
const MAX_CONTEXT_CHARS: usize = 2000;

/// SSE の接続維持用コメントの送信間隔
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

/// 成り代わり先ユーザー名を指定するヘッダ名
const IMPERSONATE_USER_NAME_HEADER: &str = "x-impersonate-user-name";

/// SSE の再接続時に最後に受け取ったイベント ID を指定するヘッダ名
const LAST_EVENT_ID_HEADER: &str = "last-event-id";

///
/// 発言投稿リクエストの構造体
///
#[derive(Debug, Clone)]
pub(crate) struct PostGameMessageRequest {
    pub authorization: Option<String>,
    /// 管理者による成り代わり（X-Impersonate-User-Name）付きのリクエストかどうか
    pub is_impersonating: bool,
    pub game_uuid: Uuid,
    pub body: PostGameMessageRequestBody,
}

///
/// 発言種別（投稿用）の列挙体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PostKind {
    Public,
    Confidential,
    Personal,
}

///
/// SSE の season イベントのデータ構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
struct SeasonEventData {
    season: String,
}

///
/// SSE 配信の差分取得結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
struct StreamUpdate {
    /// 次回の差分取得の起点となるメッセージ UUID（非公開のメッセージも含めて進める）
    cursor: Option<Uuid>,
    /// シーズンが切り替わった場合の新しいシーズン
    new_season: Option<String>,
    /// 受信者に配信するメッセージ
    messages: Vec<GameLogMessage>,
}

///
/// SSE 配信の状態の構造体
///
struct StreamState<U, G>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    game_service: Arc<GameService<U, G>>,
    user_repository: Arc<U>,
    message_repository: Arc<SqliteMessageRepository>,
    receiver: broadcast::Receiver<Uuid>,
    game_uuid: Uuid,
    viewer_user_uuid: Option<Uuid>,
    cursor: Option<Uuid>,
    season: String,
    pending: VecDeque<Event>,
    needs_fetch: bool,
}

// ============================================================================
// functions
// ============================================================================

///
/// 発言投稿リクエスト Axum ハンドラ関数
///
pub(crate) async fn post_games_messages<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<PostGameMessageRequestBody>,
) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let Ok(game_uuid) = Uuid::parse_str(&game_uuid_str) else {
        let error = PostGameMessageHandlerError::InvalidGameUuid;
        return (error.status(), Json(error.to_api_error_response())).into_response();
    };

    let request = PostGameMessageRequest {
        authorization: headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string()),
        is_impersonating: is_impersonating(&headers),
        game_uuid,
        body,
    };

    let state_clone = state.clone();

    // 投稿先シーズンの判定と保存の間にフェイズが進まないよう、卓の更新ロックを取得する
    let _game_update_guard = state.game_update_lock.lock().await;
    let result = tokio::task::spawn_blocking(move || {
        handle_post_games_messages(
            &state_clone.game_service,
            state_clone.user_repository.as_ref(),
            state_clone.message_repository.as_ref(),
            request,
        )
    })
    .await;
    drop(_game_update_guard);

    match result {
        Ok(Ok(message)) => (StatusCode::CREATED, Json(message)).into_response(),
        Ok(Err(error)) => (error.status(), Json(error.to_api_error_response())).into_response(),
        Err(join_err) => {
            eprintln!("post_games_messages task failed: {}", join_err);
            internal_error_response()
        }
    }
}

///
/// 発言投稿の純粋関数（テスト可能）
///
/// - 通常の発言は、卓が進行中なら担当国の発言、開戦前夜・感想戦ならユーザーの発言として記録する。
/// - 管理人発言（`as_admin: true`）は、成り代わりなしの管理者本人だけが公式声明として投稿できる。
///
pub(crate) fn handle_post_games_messages<U, G>(
    service: &GameService<U, G>,
    user_repository: &U,
    message_repository: &SqliteMessageRepository,
    request: PostGameMessageRequest,
) -> Result<GameLogMessage, PostGameMessageHandlerError>
where
    U: UserRepository,
    G: GameRepository,
{
    // 認証
    let access_token = optional_bearer_token(request.authorization.as_deref());
    if access_token.is_empty() {
        return Err(PostGameMessageHandlerError::Unauthorized);
    }
    let user = user_repository
        .find_by_access_token(access_token)
        .map_err(PostGameMessageHandlerError::Repository)?
        .ok_or(PostGameMessageHandlerError::Unauthorized)?;

    // リクエストボディの形式チェック
    let body = request.body;
    let kind = parse_post_kind(&body.kind)?;
    validate_context(&body.context)?;
    let recipients = match (kind, body.recipients.as_deref()) {
        (PostKind::Confidential, Some(symbols)) => parse_recipients(symbols)?,
        (PostKind::Confidential, None) => {
            return Err(PostGameMessageHandlerError::InvalidRecipients(
                "recipients is required for confidential".to_string(),
            ));
        }
        (_, Some(_)) => {
            return Err(PostGameMessageHandlerError::InvalidRecipients(
                "recipients is allowed only for confidential".to_string(),
            ));
        }
        (_, None) => Vec::new(),
    };

    // 卓の状態チェック
    let game = service.get_game(request.game_uuid).map_err(|error| match error {
        crate::GetGameError::NotFound => PostGameMessageHandlerError::NotFound,
        crate::GetGameError::Repository(error) => PostGameMessageHandlerError::Repository(error),
    })?;
    if matches!(game.status, GameStatus::Closed | GameStatus::Aborted) {
        return Err(PostGameMessageHandlerError::GameNotAcceptingMessages);
    }
    let turn = game.current_turn();

    let new_message = if body.as_admin {
        // 管理人発言
        if request.is_impersonating {
            return Err(PostGameMessageHandlerError::AdminPostNotAllowed(
                "admin post is not allowed while impersonating".to_string(),
            ));
        }
        if !user.is_admin {
            return Err(PostGameMessageHandlerError::AdminPostNotAllowed(
                "only administrators can post as admin".to_string(),
            ));
        }
        if kind != PostKind::Public {
            return Err(PostGameMessageHandlerError::InvalidKind(
                "admin post must be public".to_string(),
            ));
        }
        crate::NewChatMessage {
            sender_kind: crate::MessageSenderKind::Admin,
            sender_power: None,
            sender_user_uuid: user.uuid,
            turn,
            context: body.context,
            kind: crate::MessageKind::public(),
        }
    } else {
        // 通常の発言
        let player = game
            .players
            .iter()
            .find(|player| player.user_uuid == user.uuid)
            .ok_or(PostGameMessageHandlerError::NotAPlayer)?;

        if game.status == GameStatus::InProgress {
            // 進行中は担当国の発言
            let power = player.power.ok_or(PostGameMessageHandlerError::NotAPlayer)?;
            if is_power_eliminated(power, &game) {
                return Err(PostGameMessageHandlerError::PowerEliminated);
            }
            if recipients.contains(&power) {
                return Err(PostGameMessageHandlerError::InvalidRecipients(
                    "recipients must not include sender's own power".to_string(),
                ));
            }
            let message_kind = match kind {
                PostKind::Public => crate::MessageKind::public(),
                PostKind::Confidential => crate::MessageKind::confidential(recipients),
                PostKind::Personal => crate::MessageKind::personal(),
            };
            crate::NewChatMessage {
                sender_kind: crate::MessageSenderKind::Power,
                sender_power: Some(power),
                sender_user_uuid: user.uuid,
                turn,
                context: body.context,
                kind: message_kind,
            }
        } else {
            // 開戦前夜・感想戦はユーザーの発言で、公式声明のみ投稿できる
            if kind != PostKind::Public {
                return Err(PostGameMessageHandlerError::InvalidKind(
                    "only public messages are allowed before the game starts or during debrief".to_string(),
                ));
            }
            crate::NewChatMessage {
                sender_kind: crate::MessageSenderKind::User,
                sender_power: None,
                sender_user_uuid: user.uuid,
                turn,
                context: body.context,
                kind: crate::MessageKind::public(),
            }
        }
    };

    let record = message_repository
        .append_chat_message(game.uuid, new_message)
        .map_err(PostGameMessageHandlerError::Repository)?;

    let mut messages = vec![to_game_log_message_full(record)];
    attach_sender_users(&mut messages, user_repository).map_err(PostGameMessageHandlerError::Repository)?;
    Ok(messages.remove(0))
}

/// 成り代わり（X-Impersonate-User-Name）付きのリクエストかどうかを返す
fn is_impersonating(headers: &HeaderMap) -> bool {
    headers
        .get(IMPERSONATE_USER_NAME_HEADER)
        .map(|value| !String::from_utf8_lossy(value.as_bytes()).trim().is_empty())
        .unwrap_or(false)
}

/// 投稿用の発言種別を解析する
fn parse_post_kind(kind: &str) -> Result<PostKind, PostGameMessageHandlerError> {
    match kind {
        "public" => Ok(PostKind::Public),
        "confidential" => Ok(PostKind::Confidential),
        "personal" => Ok(PostKind::Personal),
        _ => Err(PostGameMessageHandlerError::InvalidKind(
            "kind must be one of public, confidential, personal".to_string(),
        )),
    }
}

/// 本文を検証する
fn validate_context(context: &str) -> Result<(), PostGameMessageHandlerError> {
    if context.trim().is_empty() {
        return Err(PostGameMessageHandlerError::EmptyContext);
    }
    if context.chars().count() > MAX_CONTEXT_CHARS {
        return Err(PostGameMessageHandlerError::ContextTooLong(MAX_CONTEXT_CHARS));
    }
    Ok(())
}

/// 宛先の国コード一覧を解析し、国の並び順に並べ替えて返す
fn parse_recipients(symbols: &[String]) -> Result<Vec<Power>, PostGameMessageHandlerError> {
    if symbols.is_empty() {
        return Err(PostGameMessageHandlerError::InvalidRecipients(
            "recipients must not be empty".to_string(),
        ));
    }

    let mut recipients = Vec::with_capacity(symbols.len());
    for symbol in symbols {
        let power = Power::from_symbol(symbol.trim())
            .ok_or_else(|| PostGameMessageHandlerError::InvalidRecipients(format!("unknown power in recipients: {}", symbol)))?;
        if recipients.contains(&power) {
            return Err(PostGameMessageHandlerError::InvalidRecipients(
                "recipients must not contain duplicates".to_string(),
            ));
        }
        recipients.push(power);
    }

    Ok(Power::iter().filter(|power| recipients.contains(power)).collect())
}

///
/// 発言のリアルタイム配信（SSE）リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_games_messages_stream<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    headers: HeaderMap,
    Path(game_uuid_str): Path<String>,
    Query(query): Query<GetGameMessagesStreamQueryParams>,
) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let Ok(game_uuid) = Uuid::parse_str(&game_uuid_str) else {
        return bad_request_response("game_uuid is invalid");
    };

    // 再接続時は Last-Event-ID ヘッダを優先し、無ければ after クエリを使う
    let last_event_id = headers
        .get(LAST_EVENT_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let after = match last_event_id.or(query.after).as_deref().map(Uuid::parse_str) {
        Some(Ok(uuid)) => Some(uuid),
        Some(Err(_)) => return bad_request_response("after (or Last-Event-ID) is invalid"),
        None => None,
    };

    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());

    // 取りこぼしを防ぐため、起点を決める前に保存通知を購読しておく
    let receiver = state.message_repository.subscribe();

    let game_service = Arc::clone(&state.game_service);
    let message_repository = Arc::clone(&state.message_repository);
    let init_result = tokio::task::spawn_blocking(move || {
        let game = game_service.get_game(game_uuid)?;
        let viewer_user_uuid = game_service
            .find_viewer_user_uuid(optional_bearer_token(authorization.as_deref()))
            .map_err(crate::GetGameError::Repository)?;
        // 起点の指定が無い場合は、接続時点より後のメッセージだけを配信する
        let cursor = match after {
            Some(after) => Some(after),
            None => message_repository
                .find_latest_message_uuid(game_uuid)
                .map_err(crate::GetGameError::Repository)?,
        };
        Ok::<_, crate::GetGameError>((game.current_turn(), viewer_user_uuid, cursor))
    })
    .await;

    let (season, viewer_user_uuid, cursor) = match init_result {
        Ok(Ok(initial)) => initial,
        Ok(Err(crate::GetGameError::NotFound)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    code: "not_found",
                    message: "game not found".to_string(),
                }),
            )
                .into_response();
        }
        Ok(Err(crate::GetGameError::Repository(error))) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiErrorResponse {
                    code: "repository_error",
                    message: error.to_string(),
                }),
            )
                .into_response();
        }
        Err(join_err) => {
            eprintln!("get_games_messages_stream task failed: {}", join_err);
            return internal_error_response();
        }
    };

    let stream_state = StreamState {
        game_service: Arc::clone(&state.game_service),
        user_repository: Arc::clone(&state.user_repository),
        message_repository: Arc::clone(&state.message_repository),
        receiver,
        game_uuid,
        viewer_user_uuid,
        cursor,
        season,
        pending: VecDeque::new(),
        // 起点が指定された再接続では、取りこぼした分をすぐに送り直す
        needs_fetch: after.is_some(),
    };

    let event_stream = stream::unfold(stream_state, next_stream_event);

    Sse::new(event_stream)
        .keep_alive(KeepAlive::new().interval(KEEP_ALIVE_INTERVAL).text("ping"))
        .into_response()
}

///
/// SSE で次に送るイベントを返す
///
/// 保存通知を待ち、対象卓のメッセージが保存されたら差分を取得してイベントにする。
/// 通知チャネルが閉じられた場合や差分取得に失敗した場合はストリームを終了する
/// （クライアントは Last-Event-ID 付きで再接続することで続きを受け取れる）。
///
async fn next_stream_event<U, G>(mut state: StreamState<U, G>) -> Option<(Result<Event, Infallible>, StreamState<U, G>)>
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
{
    loop {
        if let Some(event) = state.pending.pop_front() {
            return Some((Ok(event), state));
        }

        if !state.needs_fetch {
            match state.receiver.recv().await {
                Ok(game_uuid) if game_uuid == state.game_uuid => {}
                Ok(_) => continue,
                // 通知を取りこぼした可能性があるので差分を取得する
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
        state.needs_fetch = false;

        let game_service = Arc::clone(&state.game_service);
        let user_repository = Arc::clone(&state.user_repository);
        let message_repository = Arc::clone(&state.message_repository);
        let game_uuid = state.game_uuid;
        let viewer_user_uuid = state.viewer_user_uuid;
        let cursor = state.cursor;
        let season = state.season.clone();
        let update = tokio::task::spawn_blocking(move || {
            collect_stream_update(
                &game_service,
                user_repository.as_ref(),
                message_repository.as_ref(),
                game_uuid,
                viewer_user_uuid,
                cursor,
                &season,
            )
        })
        .await;

        let update = match update {
            Ok(Ok(update)) => update,
            Ok(Err(error)) => {
                eprintln!(
                    "failed to collect message stream update (game_uuid={}): {}",
                    state.game_uuid, error
                );
                return None;
            }
            Err(join_err) => {
                eprintln!("message stream task failed (game_uuid={}): {}", state.game_uuid, join_err);
                return None;
            }
        };

        state.cursor = update.cursor;
        if let Some(new_season) = update.new_season {
            match Event::default().event("season").json_data(SeasonEventData {
                season: new_season.clone(),
            }) {
                Ok(event) => state.pending.push_back(event),
                Err(error) => eprintln!("failed to serialize season event: {}", error),
            }
            state.season = new_season;
        }
        for message in update.messages {
            match Event::default()
                .id(message.message_uuid.to_string())
                .event("message")
                .json_data(&message)
            {
                Ok(event) => state.pending.push_back(event),
                Err(error) => eprintln!("failed to serialize message event: {}", error),
            }
        }
    }
}

///
/// SSE 配信の差分を取得する純粋関数（テスト可能）
///
/// 公開範囲は外交履歴取得 API と同じ。卓のステータスや担当国は取得のたびに読み直す。
///
fn collect_stream_update<U, G>(
    service: &GameService<U, G>,
    user_repository: &U,
    message_repository: &SqliteMessageRepository,
    game_uuid: Uuid,
    viewer_user_uuid: Option<Uuid>,
    cursor: Option<Uuid>,
    season: &str,
) -> Result<StreamUpdate, StreamUpdateError>
where
    U: UserRepository,
    G: GameRepository,
{
    let game = service.get_game(game_uuid).map_err(StreamUpdateError::Game)?;
    let records = message_repository
        .find_by_game_after(game_uuid, cursor)
        .map_err(StreamUpdateError::Repository)?;

    let next_cursor = records.last().map(|record| record.message_uuid).or(cursor);

    let current_season = game.current_turn();
    let new_season = (current_season != season).then_some(current_season);

    let viewer_power = viewer_user_uuid.and_then(|viewer| {
        game.players
            .iter()
            .find(|player| player.user_uuid == viewer)
            .and_then(|player| player.power)
    });
    let mut messages = apply_log_access_control(records, &game, viewer_power);
    attach_sender_users(&mut messages, user_repository).map_err(StreamUpdateError::Repository)?;

    Ok(StreamUpdate {
        cursor: next_cursor,
        new_season,
        messages,
    })
}

///
/// SSE 配信の差分取得エラーの列挙体
///
#[derive(Debug)]
enum StreamUpdateError {
    Game(crate::GetGameError),
    Repository(RepositoryError),
}

/// SSE 配信の差分取得エラーの列挙体の実装（fmt::Display トレイト）
impl std::fmt::Display for StreamUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Game(error) => write!(f, "{}", error),
            Self::Repository(error) => write!(f, "{}", error),
        }
    }
}

/// 400 Bad Request（invalid_request）レスポンスを生成する
fn bad_request_response(message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiErrorResponse {
            code: "invalid_request",
            message: message.to_string(),
        }),
    )
        .into_response()
}

/// 500 Internal Server Error レスポンスを生成する
fn internal_error_response() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiErrorResponse {
            code: "internal_error",
            message: "internal server error".to_string(),
        }),
    )
        .into_response()
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use chrono::Utc;

    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::Game;
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::domain::ProgressMode;
    use crate::domain::Province;
    use crate::domain::Regulation;
    use crate::domain::Unit;
    use crate::repositories::GameStatusFilter;
    use crate::repositories::GameSummary;
    use crate::repositories::NewGame;
    use crate::repositories::NewUser;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    // ------------------------------------------------------------------------
    // test doubles
    // ------------------------------------------------------------------------

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        users: Vec<UserRecord>,
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_uuid(&self, user_uuid: Uuid) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.users.iter().find(|user| user.uuid == user_uuid).cloned())
        }

        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.users.iter().find(|user| user.username == username).cloned())
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.users.iter().find(|user| user.access_token == access_token).cloned())
        }

        fn update_last_access_at_by_access_token(
            &self,
            _access_token: &str,
            _last_access_at: DateTime<Utc>,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
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
        games: Vec<Game>,
    }

    impl GameRepository for InMemoryGameRepository {
        fn insert(&self, _new_game: NewGame) -> Result<Game, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(Vec::new())
        }

        fn find_by_uuid(&self, game_uuid: Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(self.games.iter().find(|game| game.uuid == game_uuid).cloned())
        }

        fn find_progress_candidates(&self, _now: chrono::NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(Vec::new())
        }

        fn exists_active_game_for_user(&self, _user_uuid: Uuid) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        fn update(&self, _game: &Game) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn add_player(&self, _game_uuid: Uuid, _user_uuid: Uuid, _requested_power: Option<Power>) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn assign_game_number(&self, _game_uuid: Uuid) -> Result<i32, RepositoryError> {
            Ok(1)
        }

        fn find_paginated_by_status(
            &self,
            _filter: GameStatusFilter,
            _page: u32,
            _per_page: u32,
        ) -> Result<(Vec<GameSummary>, u64), RepositoryError> {
            Ok((Vec::new(), 0))
        }

        fn find_paginated_by_user_uuid(
            &self,
            _user_uuid: Uuid,
            _filter: Option<GameStatusFilter>,
            _page: u32,
            _per_page: u32,
        ) -> Result<(Vec<GameSummary>, u64), RepositoryError> {
            Ok((Vec::new(), 0))
        }
    }

    // ------------------------------------------------------------------------
    // fixtures
    // ------------------------------------------------------------------------

    struct Fixture {
        service: GameService<InMemoryUserRepository, InMemoryGameRepository>,
        user_repository: InMemoryUserRepository,
        message_repository: SqliteMessageRepository,
        game_uuid: Uuid,
        france_user_uuid: Uuid,
        england_user_uuid: Uuid,
    }

    fn user(username: &str, token: &str, is_admin: bool) -> UserRecord {
        UserRecord {
            id: 1,
            uuid: Uuid::now_v7(),
            discord_user_id: "123456789".to_string(),
            username: username.to_string(),
            global_name: Some(format!("{} global", username)),
            avatar_hash: None,
            avatar_url: Some(format!("https://example.com/{}.png", username)),
            access_token: token.to_string(),
            last_access_at: Utc::now(),
            is_admin,
        }
    }

    fn player(user_uuid: Uuid, power: Option<Power>) -> Player {
        Player {
            user_uuid,
            power,
            is_accepting_draw: false,
            progress_consented: false,
            is_owner: false,
            requested_power: None,
        }
    }

    /// France（token-f）と England（token-e）が参加し、France と England だけがユニットを持つ卓を作る
    fn fixture(status: GameStatus) -> Fixture {
        let france = user("france", "token-f", false);
        let england = user("england", "token-e", false);
        let outsider = user("outsider", "token-o", false);
        let admin = user("admin", "token-admin", true);

        let ready = Phase::new_ready();
        let mut phases = vec![ready.clone()];
        match status {
            GameStatus::Preparing | GameStatus::Ready => {}
            GameStatus::InProgress => {
                let mut spring = Phase::new_spring_main(ready.year, ready.index);
                spring
                    .units
                    .push(Unit::new_army(Power::France, Province::from_code("par").expect("par")));
                spring
                    .units
                    .push(Unit::new_army(Power::England, Province::from_code("lon").expect("lon")));
                phases.push(spring);
            }
            _ => {
                let spring = Phase::new_spring_main(ready.year, ready.index);
                let debrief = Phase::new_debrief(spring.year, spring.index);
                phases.push(spring);
                phases.push(debrief);
            }
        }

        let assigned = !matches!(status, GameStatus::Preparing);
        let game = Game {
            uuid: Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation: Regulation::new(
                FaceType::Girls,
                ProgressMode::Scheduled,
                DurationType::Normal,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("date"),
                21,
            )
            .expect("regulation"),
            players: vec![
                player(france.uuid, assigned.then_some(Power::France)),
                player(england.uuid, assigned.then_some(Power::England)),
            ],
            phases,
            status,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };
        let game_uuid = game.uuid;

        let user_repository = InMemoryUserRepository {
            users: vec![france.clone(), england.clone(), outsider, admin],
        };
        let game_repository = InMemoryGameRepository { games: vec![game] };
        let path = std::env::temp_dir().join(format!("moe3-message-handler-test-{}.messages.db", Uuid::now_v7()));

        Fixture {
            service: GameService::new(user_repository.clone(), game_repository),
            user_repository,
            message_repository: SqliteMessageRepository::new(&path.to_string_lossy()),
            game_uuid,
            france_user_uuid: france.uuid,
            england_user_uuid: england.uuid,
        }
    }

    fn body(kind: &str, recipients: Option<Vec<&str>>, context: &str, as_admin: bool) -> PostGameMessageRequestBody {
        PostGameMessageRequestBody {
            kind: kind.to_string(),
            recipients: recipients.map(|values| values.into_iter().map(|value| value.to_string()).collect()),
            context: context.to_string(),
            as_admin,
        }
    }

    fn post(
        fixture: &Fixture,
        token: Option<&str>,
        is_impersonating: bool,
        body: PostGameMessageRequestBody,
    ) -> Result<GameLogMessage, PostGameMessageHandlerError> {
        handle_post_games_messages(
            &fixture.service,
            &fixture.user_repository,
            &fixture.message_repository,
            PostGameMessageRequest {
                authorization: token.map(|token| format!("Bearer {}", token)),
                is_impersonating,
                game_uuid: fixture.game_uuid,
                body,
            },
        )
    }

    fn collect(fixture: &Fixture, viewer: Option<Uuid>, cursor: Option<Uuid>, season: &str) -> StreamUpdate {
        collect_stream_update(
            &fixture.service,
            &fixture.user_repository,
            &fixture.message_repository,
            fixture.game_uuid,
            viewer,
            cursor,
            season,
        )
        .expect("collect should succeed")
    }

    // ------------------------------------------------------------------------
    // handle_post_games_messages
    // ------------------------------------------------------------------------

    #[test]
    fn post_public_in_progress_is_recorded_as_power() {
        let fixture = fixture(GameStatus::InProgress);

        let message = post(&fixture, Some("token-f"), false, body("public", None, "こんにちは", false)).expect("should succeed");

        assert_eq!(message.sender, Some("f".to_string()));
        assert_eq!(message.sender_kind, "power");
        assert_eq!(message.sender_user, None);
        assert_eq!(message.kind, "public");
        assert_eq!(message.turn, "1901s");
        assert_eq!(message.context, Some("こんにちは".to_string()));
        assert_eq!(message.recipients, None);

        let stored = fixture
            .message_repository
            .find_by_game_and_season(fixture.game_uuid, "1901s", None)
            .expect("find should succeed");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].sender_user_uuid, Some(fixture.france_user_uuid));
    }

    #[test]
    fn post_confidential_sorts_recipients_in_power_order() {
        let fixture = fixture(GameStatus::InProgress);

        let message = post(
            &fixture,
            Some("token-f"),
            false,
            body("confidential", Some(vec!["g", "E"]), "密談", false),
        )
        .expect("should succeed");

        assert_eq!(message.kind, "confidential");
        assert_eq!(message.recipients, Some(vec!["e".to_string(), "g".to_string()]));
    }

    #[test]
    fn post_confidential_rejects_invalid_recipients() {
        let fixture = fixture(GameStatus::InProgress);
        let cases = vec![None, Some(vec![]), Some(vec!["f"]), Some(vec!["e", "e"]), Some(vec!["x"])];

        for recipients in cases {
            let error = post(
                &fixture,
                Some("token-f"),
                false,
                body("confidential", recipients.clone(), "密談", false),
            )
            .expect_err("should fail");
            assert_eq!(error.code(), "invalid_recipients", "recipients: {:?}", recipients);
            assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[test]
    fn post_public_rejects_recipients() {
        let fixture = fixture(GameStatus::InProgress);

        let error = post(
            &fixture,
            Some("token-f"),
            false,
            body("public", Some(vec!["e"]), "本文", false),
        )
        .expect_err("should fail");

        assert_eq!(error.code(), "invalid_recipients");
    }

    #[test]
    fn post_rejects_invalid_kind_and_context() {
        let fixture = fixture(GameStatus::InProgress);

        let error = post(&fixture, Some("token-f"), false, body("ghost", None, "本文", false)).expect_err("should fail");
        assert_eq!(error.code(), "invalid_kind");

        let error = post(&fixture, Some("token-f"), false, body("public", None, "  \n ", false)).expect_err("should fail");
        assert_eq!(error.code(), "empty_context");

        let long = "あ".repeat(MAX_CONTEXT_CHARS + 1);
        let error = post(&fixture, Some("token-f"), false, body("public", None, &long, false)).expect_err("should fail");
        assert_eq!(error.code(), "context_too_long");

        let just = "あ".repeat(MAX_CONTEXT_CHARS);
        post(&fixture, Some("token-f"), false, body("public", None, &just, false)).expect("max length should be accepted");
    }

    #[test]
    fn post_requires_authentication() {
        let fixture = fixture(GameStatus::InProgress);

        let error = post(&fixture, None, false, body("public", None, "本文", false)).expect_err("should fail");
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);

        let error = post(&fixture, Some("unknown"), false, body("public", None, "本文", false)).expect_err("should fail");
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn post_returns_not_found_for_missing_game() {
        let mut fixture = fixture(GameStatus::InProgress);
        fixture.game_uuid = Uuid::now_v7();

        let error = post(&fixture, Some("token-f"), false, body("public", None, "本文", false)).expect_err("should fail");

        assert_eq!(error.code(), "not_found");
        assert_eq!(error.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn post_rejects_non_player() {
        let fixture = fixture(GameStatus::InProgress);

        let error = post(&fixture, Some("token-o"), false, body("public", None, "本文", false)).expect_err("should fail");

        assert_eq!(error.code(), "not_a_player");
        assert_eq!(error.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn post_rejects_eliminated_power() {
        let fixture = fixture(GameStatus::InProgress);
        // Germany 担当（ユニットなし）のプレイヤーを追加する
        let germany = user("germany", "token-g", false);
        let mut game = fixture.service.get_game(fixture.game_uuid).expect("game");
        game.players.push(player(germany.uuid, Some(Power::Germany)));
        let mut users = fixture.user_repository.users.clone();
        users.push(germany);
        let user_repository = InMemoryUserRepository { users };
        let fixture = Fixture {
            service: GameService::new(user_repository.clone(), InMemoryGameRepository { games: vec![game] }),
            user_repository,
            ..fixture
        };

        let error = post(&fixture, Some("token-g"), false, body("public", None, "本文", false)).expect_err("should fail");

        assert_eq!(error.code(), "power_eliminated");
    }

    #[test]
    fn post_rejects_closed_and_aborted_games() {
        for status in [GameStatus::Closed, GameStatus::Aborted] {
            let fixture = fixture(status);

            let error = post(&fixture, Some("token-f"), false, body("public", None, "本文", false)).expect_err("should fail");
            assert_eq!(error.code(), "game_not_accepting_messages");

            let error = post(&fixture, Some("token-admin"), false, body("public", None, "本文", true)).expect_err("should fail");
            assert_eq!(error.code(), "game_not_accepting_messages");
        }
    }

    #[test]
    fn post_before_start_is_recorded_as_user_and_public_only() {
        for status in [GameStatus::Preparing, GameStatus::Ready] {
            let fixture = fixture(status);

            let message =
                post(&fixture, Some("token-f"), false, body("public", None, "よろしく", false)).expect("should succeed");
            assert_eq!(message.turn, "ready");
            assert_eq!(message.sender, None);
            assert_eq!(message.sender_kind, "user");
            assert_eq!(
                message.sender_user,
                Some(crate::api::GameLogSenderUser {
                    username: "france".to_string(),
                    global_name: Some("france global".to_string()),
                    avatar_url: Some("https://example.com/france.png".to_string()),
                })
            );

            let error = post(&fixture, Some("token-f"), false, body("personal", None, "独り言", false)).expect_err("should fail");
            assert_eq!(error.code(), "invalid_kind");

            let error = post(&fixture, Some("token-o"), false, body("public", None, "本文", false)).expect_err("should fail");
            assert_eq!(error.code(), "not_a_player");
        }
    }

    #[test]
    fn post_during_debrief_is_recorded_as_user_and_public_only() {
        for status in [GameStatus::Solo, GameStatus::Draw] {
            let fixture = fixture(status);

            let message = post(
                &fixture,
                Some("token-e"),
                false,
                body("public", None, "お疲れさまでした", false),
            )
            .expect("should succeed");
            assert_eq!(message.turn, "debrief");
            assert_eq!(message.sender_kind, "user");
            assert_eq!(message.sender_user.map(|user| user.username), Some("england".to_string()));

            let error = post(
                &fixture,
                Some("token-e"),
                false,
                body("confidential", Some(vec!["f"]), "密談", false),
            )
            .expect_err("should fail");
            assert_eq!(error.code(), "invalid_kind");

            let error = post(&fixture, Some("token-o"), false, body("public", None, "本文", false)).expect_err("should fail");
            assert_eq!(error.code(), "not_a_player");
        }
    }

    #[test]
    fn post_as_admin_is_recorded_as_admin_public() {
        let fixture = fixture(GameStatus::InProgress);

        let message =
            post(&fixture, Some("token-admin"), false, body("public", None, "管理人です", true)).expect("should succeed");

        assert_eq!(message.sender, None);
        assert_eq!(message.sender_kind, "admin");
        assert_eq!(message.sender_user, None);
        assert_eq!(message.kind, "public");
    }

    #[test]
    fn post_as_admin_by_non_player_is_allowed_before_start_and_during_debrief() {
        for status in [GameStatus::Preparing, GameStatus::Ready, GameStatus::Solo, GameStatus::Draw] {
            let fixture = fixture(status);

            let message =
                post(&fixture, Some("token-admin"), false, body("public", None, "管理人です", true)).expect("should succeed");
            assert_eq!(message.sender_kind, "admin");
            assert_eq!(message.kind, "public");

            let error =
                post(&fixture, Some("token-admin"), false, body("personal", None, "本文", true)).expect_err("should fail");
            assert_eq!(error.code(), "invalid_kind");
        }
    }

    #[test]
    fn post_as_admin_rejects_invalid_conditions() {
        let fixture = fixture(GameStatus::InProgress);

        let error = post(&fixture, Some("token-f"), false, body("public", None, "本文", true)).expect_err("should fail");
        assert_eq!(error.code(), "admin_post_not_allowed");
        assert_eq!(error.status(), StatusCode::FORBIDDEN);

        let error = post(&fixture, Some("token-admin"), true, body("public", None, "本文", true)).expect_err("should fail");
        assert_eq!(error.code(), "admin_post_not_allowed");

        let error = post(&fixture, Some("token-admin"), false, body("personal", None, "本文", true)).expect_err("should fail");
        assert_eq!(error.code(), "invalid_kind");
        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn post_without_as_admin_by_non_player_admin_is_rejected() {
        let fixture = fixture(GameStatus::InProgress);

        let error = post(&fixture, Some("token-admin"), false, body("public", None, "本文", false)).expect_err("should fail");

        assert_eq!(error.code(), "not_a_player");
    }

    #[test]
    fn post_notifies_subscribers() {
        let fixture = fixture(GameStatus::InProgress);
        let mut receiver = fixture.message_repository.subscribe();

        post(&fixture, Some("token-f"), false, body("public", None, "本文", false)).expect("should succeed");

        assert_eq!(receiver.try_recv().expect("should be notified"), fixture.game_uuid);
    }

    // ------------------------------------------------------------------------
    // collect_stream_update
    // ------------------------------------------------------------------------

    #[test]
    fn stream_update_applies_access_control_per_viewer() {
        let fixture = fixture(GameStatus::InProgress);
        let cursor = fixture
            .message_repository
            .find_latest_message_uuid(fixture.game_uuid)
            .expect("latest should succeed");
        post(&fixture, Some("token-f"), false, body("public", None, "公開", false)).expect("public");
        post(&fixture, Some("token-f"), false, body("personal", None, "独り言", false)).expect("personal");
        let last = post(
            &fixture,
            Some("token-f"),
            false,
            body("confidential", Some(vec!["e"]), "密談", false),
        )
        .expect("confidential");

        // 送信者本人は全文
        let update = collect(&fixture, Some(fixture.france_user_uuid), cursor, "1901s");
        let kinds: Vec<_> = update.messages.iter().map(|m| m.kind.as_str()).collect();
        assert_eq!(kinds, vec!["public", "personal", "confidential"]);
        assert_eq!(update.cursor, Some(last.message_uuid));
        assert_eq!(update.new_season, None);

        // 宛先は独り言が見えず、密談は全文
        let update = collect(&fixture, Some(fixture.england_user_uuid), cursor, "1901s");
        let kinds: Vec<_> = update.messages.iter().map(|m| m.kind.as_str()).collect();
        assert_eq!(kinds, vec!["public", "confidential"]);

        // 第三者は密談が masked
        let update = collect(&fixture, None, cursor, "1901s");
        let kinds: Vec<_> = update.messages.iter().map(|m| m.kind.as_str()).collect();
        assert_eq!(kinds, vec!["public", "masked"]);
        assert_eq!(update.cursor, Some(last.message_uuid));
    }

    #[test]
    fn stream_update_reports_season_change() {
        let fixture = fixture(GameStatus::InProgress);

        let update = collect(&fixture, None, None, "ready");

        assert_eq!(update.new_season, Some("1901s".to_string()));
        assert!(update.messages.is_empty());
        assert_eq!(update.cursor, None);
    }

    #[test]
    fn stream_update_after_cursor_returns_only_newer_messages() {
        let fixture = fixture(GameStatus::InProgress);
        let first = post(&fixture, Some("token-f"), false, body("public", None, "1", false)).expect("first");
        let second = post(&fixture, Some("token-e"), false, body("public", None, "2", false)).expect("second");

        let update = collect(&fixture, None, Some(first.message_uuid), "1901s");

        assert_eq!(update.messages.len(), 1);
        assert_eq!(update.messages[0].message_uuid, second.message_uuid);

        let update = collect(&fixture, None, Some(second.message_uuid), "1901s");
        assert!(update.messages.is_empty());
        assert_eq!(update.cursor, Some(second.message_uuid));
    }

    fn stream_state(
        fixture: &Fixture,
        viewer: Option<Uuid>,
        cursor: Option<Uuid>,
        needs_fetch: bool,
    ) -> StreamState<InMemoryUserRepository, InMemoryGameRepository> {
        StreamState {
            game_service: Arc::new(GameService::new(
                fixture.user_repository.clone(),
                InMemoryGameRepository {
                    games: vec![fixture.service.get_game(fixture.game_uuid).expect("game")],
                },
            )),
            user_repository: Arc::new(fixture.user_repository.clone()),
            message_repository: Arc::new(fixture.message_repository.clone()),
            receiver: fixture.message_repository.subscribe(),
            game_uuid: fixture.game_uuid,
            viewer_user_uuid: viewer,
            cursor,
            season: "1901s".to_string(),
            pending: VecDeque::new(),
            needs_fetch,
        }
    }

    #[tokio::test]
    async fn stream_yields_message_event_after_notification() {
        let fixture = fixture(GameStatus::InProgress);
        let state = stream_state(&fixture, Some(fixture.england_user_uuid), None, false);

        // 別卓の通知は無視され、対象卓の保存通知で差分が配信される
        fixture
            .message_repository
            .append_draw_message(Uuid::now_v7())
            .expect("other game message");
        let posted = post(
            &fixture,
            Some("token-f"),
            false,
            body("confidential", Some(vec!["e"]), "密談", false),
        )
        .expect("post");

        let (event, state) = tokio::time::timeout(Duration::from_secs(5), next_stream_event(state))
            .await
            .expect("should not time out")
            .expect("stream should yield");
        let event = format!("{:?}", event.expect("infallible"));
        assert!(event.contains(&posted.message_uuid.to_string()), "event: {}", event);
        assert!(event.contains("message"), "event: {}", event);
        assert_eq!(state.cursor, Some(posted.message_uuid));
        assert!(state.pending.is_empty());
    }

    #[tokio::test]
    async fn stream_resends_missed_messages_on_reconnect() {
        let fixture = fixture(GameStatus::InProgress);
        let first = post(&fixture, Some("token-f"), false, body("public", None, "1", false)).expect("first");
        let second = post(&fixture, Some("token-f"), false, body("public", None, "2", false)).expect("second");
        let state = stream_state(&fixture, None, Some(first.message_uuid), true);

        let (event, state) = tokio::time::timeout(Duration::from_secs(5), next_stream_event(state))
            .await
            .expect("should not time out")
            .expect("stream should yield");

        let event = format!("{:?}", event.expect("infallible"));
        assert!(event.contains(&second.message_uuid.to_string()), "event: {}", event);
        assert_eq!(state.cursor, Some(second.message_uuid));
    }

    #[test]
    fn season_event_data_serializes_as_expected() {
        let json = serde_json::to_string(&SeasonEventData {
            season: "1901f".to_string(),
        })
        .expect("serialize");

        assert_eq!(json, r#"{"season":"1901f"}"#);
    }

    #[test]
    fn game_log_message_json_does_not_include_internal_fields() {
        let fixture = fixture(GameStatus::Ready);
        let message = post(&fixture, Some("token-f"), false, body("public", None, "本文", false)).expect("should succeed");

        let json = serde_json::to_value(&message).expect("serialize");

        assert!(json.get("sender_user_uuid").is_none());
        assert_eq!(json["sender_kind"], "user");
        assert_eq!(json["sender_user"]["username"], "france");
        assert!(json["sender_user"].get("discord_user_id").is_none());
    }
}
