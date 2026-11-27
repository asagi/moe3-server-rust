// ============================================================================
// imports
// ============================================================================

use strum::IntoEnumIterator;
use uuid::Uuid;

use super::Game;
use super::GameRepository;
use super::GameService;
use super::GameStatus;
use super::OrdersError;
use super::Phase;
use super::Power;
use super::Province;
use super::RepositoryError;
use super::Unit;
use super::UserRepository;
use crate::domain::AdjustmentOptions;
use crate::domain::MainUnitOptions;
use crate::domain::Order;
use crate::domain::OrderInput;
use crate::domain::OrderInputKind;
use crate::domain::PhaseKind;

// ============================================================================
// definitions
// ============================================================================

///
/// 命令取得コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GetOrdersCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
}

///
/// 命令確定コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SubmitOrdersCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub season: String,
    pub phase_kind: String,
    pub orders: Vec<OrderSpec>,
}

///
/// 命令確定コマンドの命令指定の構造体（文字列のまま受け取り、サービスで解釈する）
///
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct OrderSpec {
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
/// 命令の取得・確定結果（担当国から見た命令と候補）の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OrdersView {
    pub season: String,
    pub phase_kind: &'static str,
    pub power: Power,
    pub needs_action: bool,
    /// 担当国が出している命令（自国ユニットへの命令と仮定命令）
    pub orders: Vec<Order>,
    /// メインフェイズの候補（盤面の全ユニット）
    pub main_options: Vec<MainUnitOptions>,
    /// 撤退フェイズの候補（自国の撃退されたユニットと撤退先）
    pub retreat_options: Vec<(Unit, Vec<Province>)>,
    /// 調整フェイズの候補
    pub adjustment: Option<AdjustmentOptions>,
}

///
/// 国ごとの状況の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PowerStatus {
    pub power: Power,
    pub supply_centers: usize,
    pub armies: usize,
    pub fleets: usize,
    pub needs_action: bool,
    /// 即時進行に合意しているか（過去のフェイズでは `None`）
    pub progress_consented: Option<bool>,
    /// 無政府状態か（過去のフェイズでは `None`）
    pub is_idle: Option<bool>,
    /// 担当者の最終アクセスからの経過分数（切り捨て。過去のフェイズ・担当者がいない国では `None`）
    pub absent_minutes: Option<i64>,
    pub is_eliminated: bool,
}

/// 卓サービスの構造体の実装（命令・国ごとの状況）
impl<U, G> GameService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    ///
    /// 担当国の命令と候補を取得する
    ///
    pub(crate) fn get_orders(&self, command: GetOrdersCommand) -> Result<OrdersView, OrdersError> {
        let (game, power) = self.load_game_for_orders(&command.access_token, command.game_uuid)?;
        ensure_order_phase(&game)?;
        Ok(build_orders_view(&game, power))
    }

    ///
    /// 担当国の命令を丸ごと置き換えて確定する
    ///
    pub(crate) fn submit_orders(&self, command: SubmitOrdersCommand) -> Result<OrdersView, OrdersError> {
        let (mut game, power) = self.load_game_for_orders(&command.access_token, command.game_uuid)?;

        let latest = game
            .phases
            .last()
            .ok_or_else(|| OrdersError::Forbidden("game has no phases".to_string()))?;
        if !game.current_turn().eq_ignore_ascii_case(command.season.trim()) || phase_kind_of(latest) != command.phase_kind.trim()
        {
            return Err(OrdersError::PhaseConflict);
        }
        ensure_order_phase(&game)?;

        let latest = game.phases.last_mut().expect("phase exists");
        if !latest.powers_requiring_action().contains(&power) {
            return Err(OrdersError::Forbidden(
                "power does not need to act in the current phase".to_string(),
            ));
        }

        let inputs = command.orders.iter().map(parse_order_spec).collect::<Result<Vec<_>, _>>()?;

        latest
            .replace_orders(power, &inputs)
            .map_err(|rejection| OrdersError::InvalidOrder {
                location: rejection.location.map(|p| p.code_with_coast().to_string()),
                message: rejection.message,
            })?;

        self.game_repository.update(&game).map_err(OrdersError::Repository)?;

        Ok(build_orders_view(&game, power))
    }

    ///
    /// 指定したフェイズの国ごとの状況を返す
    ///
    /// `is_latest` が `false`（過去のフェイズ）の場合、即時進行合意・無政府状態・不在時間は `None`、操作の要否は `false` にする。
    ///
    pub(crate) fn power_statuses(
        &self,
        game: &Game,
        phase: &Phase,
        is_latest: bool,
    ) -> Result<Vec<PowerStatus>, RepositoryError> {
        let requiring = phase.powers_requiring_action();
        let now = chrono::Utc::now();
        let threshold = now - chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes()));
        let is_ready_or_debrief = matches!(phase.kind, PhaseKind::Ready(_) | PhaseKind::Debrief(_));

        Power::iter()
            .map(|power| {
                let player = game.players.iter().find(|p| p.power == Some(power));
                let (progress_consented, is_idle, absent_minutes) = if is_latest {
                    let last_access_at = match player {
                        Some(player) => self.user_repository.find_by_uuid(player.user_uuid)?.map(|u| u.last_access_at),
                        None => None,
                    };
                    let is_idle = last_access_at.is_none_or(|at| at <= threshold);
                    // 時計のずれで最終アクセスが未来になっても負にしない
                    let absent_minutes = last_access_at.map(|at| (now - at).num_minutes().max(0));
                    (
                        Some(player.is_some_and(|p| p.progress_consented)),
                        Some(is_idle),
                        absent_minutes,
                    )
                } else {
                    (None, None, None)
                };

                let supply_centers = phase.count_supply_centers(&power);
                let units = phase.units.iter().filter(|u| u.power == power);
                let fleets = units.clone().filter(|u| u.is_fleet()).count();
                let armies = units.count() - fleets;

                Ok(PowerStatus {
                    power,
                    supply_centers,
                    armies,
                    fleets,
                    needs_action: is_latest && requiring.contains(&power),
                    progress_consented,
                    is_idle,
                    absent_minutes,
                    is_eliminated: !is_ready_or_debrief && supply_centers == 0 && armies + fleets == 0,
                })
            })
            .collect()
    }

    /// 認証し、卓とリクエスト者の担当国を取得する
    fn load_game_for_orders(&self, access_token: &str, game_uuid: Uuid) -> Result<(Game, Power), OrdersError> {
        let access_token = access_token.trim();
        if access_token.is_empty() {
            return Err(OrdersError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(access_token)
            .map_err(OrdersError::Repository)?
            .ok_or(OrdersError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(game_uuid)
            .map_err(OrdersError::Repository)?
            .ok_or(OrdersError::NotFound)?;

        let power = game
            .players
            .iter()
            .find(|p| p.user_uuid == user.uuid)
            .and_then(|p| p.power)
            .ok_or_else(|| OrdersError::Forbidden("user is not a player with an assigned power".to_string()))?;

        Ok((game, power))
    }
}

/// フェイズ種別の文字列を返す
pub(crate) fn phase_kind_of(phase: &Phase) -> &'static str {
    match phase.kind {
        PhaseKind::Ready(_) => "ready",
        PhaseKind::SpringMain(_) | PhaseKind::FallMain(_) => "main",
        PhaseKind::SpringRetreat(_) | PhaseKind::FallRetreat(_) => "retreat",
        PhaseKind::Adjustment(_) => "adjustment",
        PhaseKind::Debrief(_) => "debrief",
    }
}

/// 命令を扱える卓・フェイズか（進行中で、最新フェイズがメイン・撤退・調整）を検証する
fn ensure_order_phase(game: &Game) -> Result<(), OrdersError> {
    if game.status != GameStatus::InProgress {
        return Err(OrdersError::Forbidden("game is not in progress".to_string()));
    }
    let latest = game
        .phases
        .last()
        .ok_or_else(|| OrdersError::Forbidden("game has no phases".to_string()))?;
    if !(latest.is_main() || latest.is_retreat() || latest.is_adjustment()) {
        return Err(OrdersError::Forbidden(
            "orders are available only in a main, retreat or adjustment phase".to_string(),
        ));
    }
    Ok(())
}

/// 担当国から見た命令と候補を組み立てる
fn build_orders_view(game: &Game, power: Power) -> OrdersView {
    let phase = game.phases.last().expect("phase exists");

    let orders = phase.orders.iter().filter(|o| o.power == power).copied().collect();
    let main_options = if phase.is_main() { phase.main_options() } else { Vec::new() };
    let retreat_options = if phase.is_retreat() {
        phase
            .units
            .iter()
            .filter(|u| u.power == power && u.dislodged)
            .map(|u| (*u, phase.retreat_dests(u)))
            .collect()
    } else {
        Vec::new()
    };
    let adjustment = phase.is_adjustment().then(|| phase.adjustment_options(&power));

    OrdersView {
        season: game.current_turn(),
        phase_kind: phase_kind_of(phase),
        power,
        needs_action: phase.powers_requiring_action().contains(&power),
        orders,
        main_options,
        retreat_options,
        adjustment,
    }
}

/// 命令指定を命令入力に変換する
fn parse_order_spec(spec: &OrderSpec) -> Result<OrderInput, OrdersError> {
    let location_text = spec.location.trim().to_lowercase();
    let invalid = |message: String| OrdersError::InvalidOrder {
        location: Some(location_text.clone()),
        message,
    };
    let province = |value: &str| -> Result<Province, OrdersError> {
        Province::from_code(&value.trim().to_lowercase()).ok_or_else(|| invalid(format!("invalid province code: {}", value)))
    };
    let required = |value: &Option<String>, name: &str| -> Result<Province, OrdersError> {
        match value {
            Some(value) => province(value),
            None => Err(invalid(format!("{} is required", name))),
        }
    };

    let location = province(&spec.location)?;
    let unit_power =
        Power::from_symbol(spec.unit_power.trim()).ok_or_else(|| invalid(format!("invalid unit_power: {}", spec.unit_power)))?;
    let is_fleet = match spec.unit_kind.trim().to_lowercase().as_str() {
        "a" => false,
        "f" => true,
        _ => return Err(invalid(format!("invalid unit_kind: {}", spec.unit_kind))),
    };

    let kind = match spec.order_kind.trim() {
        "hold" => OrderInputKind::Hold,
        "move" => OrderInputKind::Move {
            dest: required(&spec.dest, "dest")?,
            via_convoy: spec.via_convoy.unwrap_or(false),
        },
        "support" => OrderInputKind::Support {
            target_location: required(&spec.target_location, "target_location")?,
            target_dest: match &spec.target_dest {
                Some(value) => Some(province(value)?),
                None => None,
            },
        },
        "convoy" => OrderInputKind::Convoy {
            target_location: required(&spec.target_location, "target_location")?,
            target_dest: required(&spec.target_dest, "target_dest")?,
        },
        "retreat" => OrderInputKind::Retreat {
            dest: required(&spec.dest, "dest")?,
        },
        "build" => OrderInputKind::Build,
        "disband" => OrderInputKind::Disband,
        other => return Err(invalid(format!("invalid order_kind: {}", other))),
    };

    Ok(OrderInput {
        unit_power,
        is_fleet,
        location,
        kind,
    })
}
