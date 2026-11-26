// ============================================================================
// imports
// ============================================================================

use axum::extract::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::AppState;
use super::DiscordIdentityProvider;
use super::GameRepository;
use super::GameService;
use super::UserRepository;
use super::game_handler::optional_bearer_token;
use crate::Order;
use crate::OrderKind;
use crate::Province;
use crate::Unit;
use crate::services::GetOrdersCommand;
use crate::services::OrderSpec;
use crate::services::OrdersError;
use crate::services::OrdersView;
use crate::services::SubmitOrdersCommand;

// ============================================================================
// definitions
// ============================================================================

///
/// 命令確定リクエストボディの構造体
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PutOrdersRequestBody {
    pub season: String,
    pub phase_kind: String,
    #[serde(default)]
    pub orders: Vec<OrderBody>,
}

///
/// 命令確定リクエストの命令の構造体
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OrderBody {
    pub unit_power: String,
    pub unit_kind: String,
    pub location: String,
    pub order_kind: String,
    pub dest: Option<String>,
    pub target_location: Option<String>,
    pub target_dest: Option<String>,
    pub via_convoy: Option<bool>,
}

///
/// 命令取得・確定レスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct OrdersResponse {
    pub season: String,
    pub phase_kind: String,
    pub power: String,
    pub needs_action: bool,
    pub orders: Vec<OrderEntryResponse>,
    pub options: Vec<UnitOptionsResponse>,
    pub adjustment: Option<AdjustmentResponse>,
}

///
/// 命令の構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct OrderEntryResponse {
    pub unit_power: String,
    pub unit_kind: String,
    pub location: String,
    pub order_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_dest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via_convoy: Option<bool>,
}

///
/// ユニットごとの命令候補の構造体
///
/// メインフェイズでは `move_dests`・`convoy_dests`・`support_targets`・`convoy_targets` を、
/// 撤退フェイズでは `retreat_dests` を返す（フェイズに関係しないフィールドは省略する）。
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct UnitOptionsResponse {
    pub unit_power: String,
    pub unit_kind: String,
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_dests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convoy_dests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_targets: Option<Vec<SupportTargetResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convoy_targets: Option<Vec<ConvoyTargetResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retreat_dests: Option<Vec<String>>,
}

///
/// 支援候補の構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct SupportTargetResponse {
    pub target_location: String,
    /// 支援先（`null` は維持の支援）
    pub target_dests: Vec<Option<String>>,
}

///
/// 輸送候補の構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct ConvoyTargetResponse {
    pub target_location: String,
    pub target_dests: Vec<String>,
}

///
/// 調整フェイズの候補の構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct AdjustmentResponse {
    pub build_count: usize,
    pub disband_count: usize,
    pub buildable: Vec<BuildableResponse>,
}

///
/// 増設できる地域の構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct BuildableResponse {
    pub location: String,
    pub unit_kinds: Vec<String>,
}

///
/// 命令 API のエラーレスポンスの構造体
///
#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct OrdersErrorResponse {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

// ============================================================================
// functions
// ============================================================================

///
/// 命令取得リクエスト Axum ハンドラ関数
///
pub(crate) async fn get_games_orders<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let Ok(game_uuid) = Uuid::parse_str(&game_uuid_str) else {
        return invalid_game_uuid_response();
    };
    let access_token = extract_access_token(&headers);

    let game_service = std::sync::Arc::clone(&state.game_service);
    let result =
        tokio::task::spawn_blocking(move || handle_get_orders(&game_service, GetOrdersCommand { access_token, game_uuid })).await;

    into_response(result)
}

///
/// 命令確定リクエスト Axum ハンドラ関数
///
pub(crate) async fn put_games_orders<U, G, D>(
    State(state): State<AppState<U, G, D>>,
    Path(game_uuid_str): Path<String>,
    headers: HeaderMap,
    Json(body): Json<PutOrdersRequestBody>,
) -> Response
where
    U: UserRepository + Send + Sync + 'static,
    G: GameRepository + Send + Sync + 'static,
    D: DiscordIdentityProvider + Send + Sync + 'static,
{
    let Ok(game_uuid) = Uuid::parse_str(&game_uuid_str) else {
        return invalid_game_uuid_response();
    };
    let command = SubmitOrdersCommand {
        access_token: extract_access_token(&headers),
        game_uuid,
        season: body.season,
        phase_kind: body.phase_kind,
        orders: body
            .orders
            .into_iter()
            .map(|o| OrderSpec {
                unit_power: o.unit_power,
                unit_kind: o.unit_kind,
                location: o.location,
                order_kind: o.order_kind,
                dest: o.dest,
                target_location: o.target_location,
                target_dest: o.target_dest,
                via_convoy: o.via_convoy,
            })
            .collect(),
    };

    // 定時進行とぶつからないよう、卓更新ロックを取ってから確定する
    let _game_update_guard = state.game_update_lock.lock().await;
    let game_service = std::sync::Arc::clone(&state.game_service);
    let result = tokio::task::spawn_blocking(move || handle_put_orders(&game_service, command)).await;
    drop(_game_update_guard);

    into_response(result)
}

/// 命令取得の純粋関数（テスト可能）
pub(crate) fn handle_get_orders<U, G>(
    service: &GameService<U, G>,
    command: GetOrdersCommand,
) -> Result<OrdersResponse, OrdersError>
where
    U: UserRepository,
    G: GameRepository,
{
    service.get_orders(command).map(to_orders_response)
}

/// 命令確定の純粋関数（テスト可能）
pub(crate) fn handle_put_orders<U, G>(
    service: &GameService<U, G>,
    command: SubmitOrdersCommand,
) -> Result<OrdersResponse, OrdersError>
where
    U: UserRepository,
    G: GameRepository,
{
    service.submit_orders(command).map(to_orders_response)
}

/// Authorization ヘッダからアクセストークンを取り出す（無い場合は空文字）
fn extract_access_token(headers: &HeaderMap) -> String {
    optional_bearer_token(headers.get("authorization").and_then(|v| v.to_str().ok())).to_string()
}

/// game_uuid 不正時のレスポンスを生成する
fn invalid_game_uuid_response() -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(OrdersErrorResponse {
            code: "invalid_request",
            message: "game_uuid is invalid".to_string(),
            location: None,
        }),
    )
        .into_response()
}

/// 処理結果を HTTP レスポンスに変換する
fn into_response(result: Result<Result<OrdersResponse, OrdersError>, tokio::task::JoinError>) -> Response {
    match result {
        Ok(Ok(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(Err(error)) => {
            let (status, body) = to_error_response(error);
            (status, Json(body)).into_response()
        }
        Err(join_err) => {
            eprintln!("orders task failed: {}", join_err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OrdersErrorResponse {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                    location: None,
                }),
            )
                .into_response()
        }
    }
}

/// サービスのエラーを HTTP ステータスとエラーレスポンスに変換する
fn to_error_response(error: OrdersError) -> (StatusCode, OrdersErrorResponse) {
    let message = match &error {
        OrdersError::Forbidden(message) => message.clone(),
        OrdersError::InvalidOrder { message, .. } => message.clone(),
        other => other.to_string(),
    };
    let (status, code, location) = match error {
        OrdersError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", None),
        OrdersError::NotFound => (StatusCode::NOT_FOUND, "not_found", None),
        OrdersError::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden", None),
        OrdersError::PhaseConflict => (StatusCode::CONFLICT, "phase_conflict", None),
        OrdersError::InvalidOrder { location, .. } => (StatusCode::BAD_REQUEST, "invalid_order", location),
        OrdersError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "repository_error", None),
    };
    (status, OrdersErrorResponse { code, message, location })
}

/// 地域コード（盤面と同じ形）を返す
fn code(province: Province) -> String {
    province.code_with_coast().to_string()
}

/// ユニット種別の記号（小文字）を返す
fn unit_kind(unit: &Unit) -> String {
    unit.kind_code().to_string()
}

/// 命令をレスポンス形式に変換する
fn to_order_entry(order: &Order) -> OrderEntryResponse {
    let (order_kind, dest, target_location, target_dest, via_convoy) = match &order.kind {
        OrderKind::Hold(_) => ("hold", None, None, None, None),
        OrderKind::Move(m) => ("move", Some(code(m.dest)), None, None, Some(m.via_convoy)),
        OrderKind::Support(s) => (
            "support",
            None,
            Some(code(s.target_unit.location)),
            s.target_dest.map(code),
            None,
        ),
        OrderKind::Convoy(c) => (
            "convoy",
            None,
            Some(code(c.target_unit.location)),
            Some(code(c.target_dest)),
            None,
        ),
        OrderKind::Retreat(r) => ("retreat", Some(code(r.dest)), None, None, None),
        OrderKind::Build(_) => ("build", None, None, None, None),
        OrderKind::Disband(_) => ("disband", None, None, None, None),
    };
    OrderEntryResponse {
        unit_power: order.unit.power.symbol().to_string(),
        unit_kind: unit_kind(&order.unit),
        location: code(order.unit.location),
        order_kind: order_kind.to_string(),
        dest,
        target_location,
        target_dest,
        via_convoy,
    }
}

/// 命令と候補をレスポンス形式に変換する
fn to_orders_response(view: OrdersView) -> OrdersResponse {
    let mut options: Vec<UnitOptionsResponse> = view
        .main_options
        .iter()
        .map(|o| UnitOptionsResponse {
            unit_power: o.unit.power.symbol().to_string(),
            unit_kind: unit_kind(&o.unit),
            location: code(o.unit.location),
            move_dests: Some(o.move_dests.iter().copied().map(code).collect()),
            convoy_dests: Some(o.convoy_dests.iter().copied().map(code).collect()),
            support_targets: Some(
                o.support_targets
                    .iter()
                    .map(|s| SupportTargetResponse {
                        target_location: code(s.target.location),
                        target_dests: s.dests.iter().map(|d| d.map(code)).collect(),
                    })
                    .collect(),
            ),
            convoy_targets: Some(
                o.convoy_targets
                    .iter()
                    .map(|c| ConvoyTargetResponse {
                        target_location: code(c.target.location),
                        target_dests: c.dests.iter().copied().map(code).collect(),
                    })
                    .collect(),
            ),
            retreat_dests: None,
        })
        .collect();
    options.extend(view.retreat_options.iter().map(|(unit, dests)| UnitOptionsResponse {
        unit_power: unit.power.symbol().to_string(),
        unit_kind: unit_kind(unit),
        location: code(unit.location),
        move_dests: None,
        convoy_dests: None,
        support_targets: None,
        convoy_targets: None,
        retreat_dests: Some(dests.iter().copied().map(code).collect()),
    }));

    OrdersResponse {
        season: view.season,
        phase_kind: view.phase_kind.to_string(),
        power: view.power.symbol().to_string(),
        needs_action: view.needs_action,
        orders: view.orders.iter().map(to_order_entry).collect(),
        options,
        adjustment: view.adjustment.map(|a| AdjustmentResponse {
            build_count: a.build_count,
            disband_count: a.disband_count,
            buildable: a
                .buildable
                .iter()
                .map(|s| {
                    let mut unit_kinds = Vec::new();
                    if s.army {
                        unit_kinds.push("a".to_string());
                    }
                    if s.fleet {
                        unit_kinds.push("f".to_string());
                    }
                    BuildableResponse {
                        location: code(s.location),
                        unit_kinds,
                    }
                })
                .collect(),
        }),
    }
}
