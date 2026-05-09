// ============================================================================
// imports
// ============================================================================

use chrono::FixedOffset;
use chrono::TimeZone;
use chrono::Timelike;
use uuid::Uuid;

use super::CreateGameError;
use super::Game;
use super::GameProgressionService;
use super::GameRepository;
use super::GameStatus;
use super::GameStatusFilter;
use super::GameSummary;
use super::JoinGameError;
use super::ListGamesError;
use super::NewGame;
use super::Phase;
use super::Player;
use super::Power;
use super::Province;
use super::Regulation;
use super::RepositoryError;
use super::SetDrawProposalError;
use super::SetNextUpdateAtError;
use super::SetProgressConsensusError;
use super::SetProgressModeError;
use super::SetTerritoryError;
use super::SetUnitError;
use super::Unit;
use super::UserRepository;
use crate::Territory;
use strum::IntoEnumIterator;

// ============================================================================
// definitions
// ============================================================================

///
/// 新卓作成コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CreateGameCommand {
    pub access_token: String,
    pub regulation: Regulation,
    pub requested_power: Option<Power>,
    pub keyword: Option<String>,
}

///
/// 新卓作成処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CreateGameResult {
    pub game: Game,
    pub owner_user_uuid: Uuid,
    pub requested_power: Option<Power>,
}

///
/// 卓参加コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JoinGameCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub requested_power: Option<Power>,
    pub keyword: Option<String>,
}

///
/// 卓参加処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JoinGameResult {
    pub game: Game,
    pub user_uuid: Uuid,
    pub requested_power: Option<Power>,
}

///
/// 和平終了フラグ設定コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetDrawProposalCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub draw_proposal: bool,
}

///
/// 和平終了フラグ設定処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetDrawProposalResult {
    pub game: Game,
    pub changed: bool,
}

///
/// ユニット配置制御コマンドのユニット指定
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UnitSpec {
    pub power_symbol: String,
    pub kind_str: String,
}

///
/// ユニット配置制御コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetUnitCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub location: String,
    pub unit: Option<UnitSpec>,
    pub season: String,
}

///
/// ユニット配置制御処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetUnitResult {
    pub game: Game,
    pub old_unit: Option<Unit>,
    pub new_unit: Option<Unit>,
    pub changed: bool,
}

///
/// 占領情報編集コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetTerritoryCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub code: String,
    pub power: Option<String>,
    pub season: String,
}

///
/// 占領情報編集処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetTerritoryResult {
    pub game: Game,
    pub province: Province,
    pub old_power: Option<Power>,
    pub new_power: Option<Power>,
    pub changed: bool,
}

///
/// 進行モード変更コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetProgressModeCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub season: String,
}

///
/// 進行モード変更処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetProgressModeResult {
    pub game: Game,
    pub changed: bool,
}

///
/// 即時進行合意設定コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetProgressConsensusCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub agreed: bool,
}

///
/// 即時進行合意設定処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetProgressConsensusResult {
    pub game: Game,
    pub agreed: bool,
    pub actor_power: Power,
    pub changed: bool,
    pub reached_consensus_in_main_phase: bool,
}

///
/// 次回更新時刻変更コマンドの構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetNextUpdateAtCommand {
    pub access_token: String,
    pub game_uuid: Uuid,
    pub next_update_at: String,
    pub season: String,
}

///
/// 次回更新時刻変更処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetNextUpdateAtResult {
    pub game: Game,
    pub next_update_at_jst: String,
    pub changed: bool,
}

///
/// 卓一覧取得処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ListGamesResult {
    pub games: Vec<GameSummary>,
    pub total: u64,
}

///
/// 卓サービスの構造体
///
pub(crate) struct GameService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    user_repository: U,
    game_repository: G,
}

/// 卓サービスの構造体の実装
impl<U, G> GameService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    /// 新卓作成から開始までの最低猶予時間（分）
    const MIN_START_LEAD_MINUTES: i64 = 30;

    ///
    /// new 関数
    ///
    pub(crate) fn new(user_repository: U, game_repository: G) -> Self {
        Self {
            user_repository,
            game_repository,
        }
    }

    ///
    /// 新卓を作成する
    ///
    pub(crate) fn create_game(&self, command: CreateGameCommand) -> Result<CreateGameResult, CreateGameError> {
        let access_token = command.access_token.trim().to_string();
        if access_token.is_empty() {
            return Err(CreateGameError::InvalidRequest("access_token is empty".to_string()));
        }

        let user = self
            .user_repository
            .find_by_access_token(&access_token)
            .map_err(CreateGameError::Repository)?
            .ok_or(CreateGameError::Unauthorized)?;

        // NOTE: check-then-insert の競合について
        // 同一ユーザーが並行してリクエストを送った場合、このチェックと insert の間に
        // 別のゲームが作成されうる（TOCTOU）。現状はシングルサーバー構成かつ
        // tokio::task::spawn_blocking で直列化されるため実運用上のリスクは低い。
        // 将来的にスケールアウトが必要になった際は DB 制約またはトランザクション内での
        // 存在確認に変更すること。
        if self
            .game_repository
            .exists_active_game_for_user(user.uuid)
            .map_err(CreateGameError::Repository)?
        {
            return Err(CreateGameError::Forbidden(
                "user is already participating in another active game".to_string(),
            ));
        }

        let owner = Player {
            user_uuid: user.uuid,
            power: None,
            is_accepting_draw: false,
            progress_consented: false,
            is_owner: true,
            requested_power: command.requested_power,
        };

        let next_update_jst = command
            .regulation
            .start_date
            .and_hms_opt(command.regulation.first_period_hour as u32, 0, 0)
            .ok_or(CreateGameError::InvalidRequest(
                "first_period_hour is out of range".to_string(),
            ))?;

        let jst = FixedOffset::east_opt(9 * 60 * 60)
            .ok_or(CreateGameError::Internal("failed to build JST timezone offset".to_string()))?;
        // API request start_date/first_period_hour are JST business-time inputs.
        let next_update = jst
            .from_local_datetime(&next_update_jst)
            .single()
            .ok_or(CreateGameError::Internal(
                "failed to convert next_update from JST local time".to_string(),
            ))?
            .with_timezone(&chrono::Utc)
            .naive_utc();

        Self::validate_start_datetime(next_update, chrono::Utc::now().naive_utc())?;

        let game = Game {
            uuid: Uuid::now_v7(),
            game_number: None,
            keyword: command.keyword,
            regulation: command.regulation,
            players: vec![owner],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: Some(next_update),
        };

        let created = self
            .game_repository
            .insert(NewGame { game })
            .map_err(CreateGameError::Repository)?;

        let (owner_user_uuid, requested_power) = created
            .players
            .iter()
            .find(|player| player.is_owner)
            .map(|owner| (owner.user_uuid, owner.requested_power))
            .ok_or(CreateGameError::Internal(
                "created game is missing an owner player".to_string(),
            ))?;

        Ok(CreateGameResult {
            game: created,
            owner_user_uuid,
            requested_power,
        })
    }

    /// 開始日時の妥当性を検証する
    fn validate_start_datetime(start_datetime: chrono::NaiveDateTime, now: chrono::NaiveDateTime) -> Result<(), CreateGameError> {
        let min_allowed = now + chrono::Duration::minutes(Self::MIN_START_LEAD_MINUTES);
        if start_datetime <= min_allowed {
            return Err(CreateGameError::InvalidRequest(format!(
                "initial next_update_at must be more than {} minutes in the future",
                Self::MIN_START_LEAD_MINUTES,
            )));
        }

        Ok(())
    }

    ///
    /// ユーザーをプレイヤーとして卓に追加する
    ///
    pub(crate) fn join_game(&self, command: JoinGameCommand) -> Result<JoinGameResult, JoinGameError> {
        let access_token = command.access_token.trim().to_string();
        if access_token.is_empty() {
            return Err(JoinGameError::InvalidRequest("access_token is empty".to_string()));
        }

        let user = self
            .user_repository
            .find_by_access_token(&access_token)
            .map_err(JoinGameError::Repository)?
            .ok_or(JoinGameError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(JoinGameError::Repository)?
            .ok_or(JoinGameError::NotFound)?;

        // 既にこの卓に参加済みなら冪等な成功を返す（requested_powerも一致していればOK）
        if let Some(existing) = game.players.iter().find(|p| p.user_uuid == user.uuid) {
            // requested_powerが異なる場合はエラー
            if existing.requested_power != command.requested_power {
                return Err(JoinGameError::InvalidRequest(
                    "user already joined with different requested_power".to_string(),
                ));
            }
            return Ok(JoinGameResult {
                game,
                user_uuid: user.uuid,
                requested_power: command.requested_power,
            });
        }

        // 他の卓に参加中かチェック（同卓参加済みは上で除外済み）
        if self
            .game_repository
            .exists_active_game_for_user(user.uuid)
            .map_err(JoinGameError::Repository)?
        {
            return Err(JoinGameError::Forbidden(
                "user is already participating in another active game".to_string(),
            ));
        }

        // keyword 不一致の場合はエラー（卓にkeywordがない場合にリクエストにkeywordがあっても弾く）
        if command.keyword.as_deref().unwrap_or("") != game.keyword.as_deref().unwrap_or("") {
            return Err(JoinGameError::Forbidden("keyword does not match".to_string()));
        }

        if game.status != GameStatus::Preparing {
            return Err(JoinGameError::Forbidden("game is not accepting new players".to_string()));
        }

        // 定員超過チェック
        if game.players.len() >= Power::iter().count() {
            return Err(JoinGameError::Forbidden("game is full (max players reached)".to_string()));
        }

        // DBに新規プレイヤーを追加
        self.game_repository
            .add_player(game.uuid, user.uuid, command.requested_power)
            .map_err(|e| match e {
                RepositoryError::Conflict => JoinGameError::Forbidden("game is full (max players reached)".to_string()),
                other => JoinGameError::Repository(other),
            })?;

        // 最新状態を再取得する
        let mut updated_game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(JoinGameError::Repository)?
            .ok_or(JoinGameError::NotFound)?;

        // 7 人揃ったら担当国割り当て・ステータス変更・卓番号採番を行う
        if updated_game.players.len() == Power::iter().count() {
            GameProgressionService::<U, G>::assign_powers(&mut updated_game.players);
            updated_game.status = GameStatus::Ready;
            // assign_game_number は採番と DB 更新をアトミックに行う
            let game_number = self
                .game_repository
                .assign_game_number(updated_game.uuid)
                .map_err(JoinGameError::Repository)?;
            updated_game.game_number = Some(game_number);
        }

        self.game_repository
            .update(&updated_game)
            .map_err(JoinGameError::Repository)?;

        Ok(JoinGameResult {
            game: updated_game,
            user_uuid: user.uuid,
            requested_power: command.requested_power,
        })
    }

    ///
    /// 卓主権限で和平終了フラグを設定する
    ///
    pub(crate) fn set_draw_proposal(
        &self,
        command: SetDrawProposalCommand,
    ) -> Result<SetDrawProposalResult, SetDrawProposalError> {
        let access_token = command.access_token.trim().to_string();
        if access_token.is_empty() {
            return Err(SetDrawProposalError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(&access_token)
            .map_err(SetDrawProposalError::Repository)?
            .ok_or(SetDrawProposalError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(SetDrawProposalError::Repository)?
            .ok_or(SetDrawProposalError::NotFound)?;

        let owner =
            game.players
                .iter()
                .find(|p| p.is_owner && p.user_uuid == user.uuid)
                .ok_or(SetDrawProposalError::Forbidden(
                    "user is not the owner of this game".to_string(),
                ))?;

        // 最新フェイズがメインフェイズ以外の場合はエラー
        let latest_phase = game
            .phases
            .last()
            .ok_or(SetDrawProposalError::Forbidden("game has no phases".to_string()))?;
        let is_main_phase = matches!(
            latest_phase.kind,
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        );
        if !is_main_phase {
            return Err(SetDrawProposalError::Forbidden(
                "draw proposal can only be set during a main phase".to_string(),
            ));
        }

        // 状態が既に同じなら変更なしで OK を返す
        if owner.is_accepting_draw == command.draw_proposal {
            return Ok(SetDrawProposalResult { game, changed: false });
        }

        let mut updated_game = game;
        if let Some(owner_player) = updated_game.players.iter_mut().find(|p| p.is_owner) {
            owner_player.is_accepting_draw = command.draw_proposal;
        }

        self.game_repository
            .update(&updated_game)
            .map_err(SetDrawProposalError::Repository)?;

        Ok(SetDrawProposalResult {
            game: updated_game,
            changed: true,
        })
    }

    ///
    /// 卓主権限で次回更新時刻を変更する
    ///
    pub(crate) fn set_next_update_at(
        &self,
        command: SetNextUpdateAtCommand,
    ) -> Result<SetNextUpdateAtResult, SetNextUpdateAtError> {
        if command.access_token.is_empty() {
            return Err(SetNextUpdateAtError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(&command.access_token)
            .map_err(SetNextUpdateAtError::Repository)?
            .ok_or(SetNextUpdateAtError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(SetNextUpdateAtError::Repository)?
            .ok_or(SetNextUpdateAtError::NotFound)?;

        game.players
            .iter()
            .find(|p| p.is_owner && p.user_uuid == user.uuid)
            .ok_or_else(|| SetNextUpdateAtError::Forbidden("user is not the owner of this game".to_string()))?;

        let latest_phase = game
            .phases
            .last()
            .ok_or_else(|| SetNextUpdateAtError::Forbidden("game has no phases".to_string()))?;

        let is_allowed_phase = matches!(
            latest_phase.kind,
            crate::domain::PhaseKind::Ready(_) | crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        );
        if !is_allowed_phase {
            return Err(SetNextUpdateAtError::Forbidden(
                "next_update_at can only be changed during a ready or main phase".to_string(),
            ));
        }

        let command_season = command.season.to_ascii_lowercase();
        if game.current_turn() != command_season {
            return Err(SetNextUpdateAtError::PhaseConflict);
        }

        let current_next_update = game
            .next_update_at
            .ok_or_else(|| SetNextUpdateAtError::Forbidden("next_update_at is not set for this game".to_string()))?;

        let naive_jst = chrono::NaiveDateTime::parse_from_str(&command.next_update_at, "%Y-%m-%d %H:%M").map_err(|_| {
            SetNextUpdateAtError::InvalidRequest("next_update_at must be in 'YYYY-MM-DD HH:MM' format".to_string())
        })?;

        if naive_jst.minute() % 5 != 0 {
            return Err(SetNextUpdateAtError::InvalidRequest(
                "minutes of next_update_at must be a multiple of 5".to_string(),
            ));
        }

        let jst = chrono::FixedOffset::east_opt(9 * 3600)
            .ok_or_else(|| SetNextUpdateAtError::InvalidRequest("internal: failed to build JST offset".to_string()))?;

        let new_next_update = jst
            .from_local_datetime(&naive_jst)
            .single()
            .ok_or_else(|| SetNextUpdateAtError::InvalidRequest("ambiguous or invalid local time".to_string()))?
            .with_timezone(&chrono::Utc)
            .naive_utc();

        if new_next_update < current_next_update {
            return Err(SetNextUpdateAtError::InvalidRequest(
                "new next_update_at must not be earlier than the current next_update_at".to_string(),
            ));
        }

        if new_next_update == current_next_update {
            return Ok(SetNextUpdateAtResult {
                game,
                next_update_at_jst: command.next_update_at,
                changed: false,
            });
        }

        let mut updated_game = game;
        updated_game.next_update_at = Some(new_next_update);

        self.game_repository
            .update(&updated_game)
            .map_err(SetNextUpdateAtError::Repository)?;

        Ok(SetNextUpdateAtResult {
            game: updated_game,
            next_update_at_jst: command.next_update_at,
            changed: true,
        })
    }

    ///
    /// ユニットを配置・削除する
    ///
    pub(crate) fn set_unit(&self, command: SetUnitCommand) -> Result<SetUnitResult, SetUnitError> {
        if command.access_token.is_empty() {
            return Err(SetUnitError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(&command.access_token)
            .map_err(SetUnitError::Repository)?
            .ok_or(SetUnitError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(SetUnitError::Repository)?
            .ok_or(SetUnitError::NotFound)?;

        game.players
            .iter()
            .find(|p| p.is_owner && p.user_uuid == user.uuid)
            .ok_or_else(|| SetUnitError::Forbidden("user is not the owner of this game".to_string()))?;

        let latest_phase = game
            .phases
            .last()
            .ok_or_else(|| SetUnitError::Forbidden("game has no phases".to_string()))?;
        let is_main_phase = matches!(
            latest_phase.kind,
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        );
        if !is_main_phase {
            return Err(SetUnitError::Forbidden(
                "units can only be set during a main phase".to_string(),
            ));
        }

        if game.current_turn() != command.season {
            return Err(SetUnitError::PhaseConflict);
        }

        let location = Province::from_code(&command.location)
            .ok_or_else(|| SetUnitError::InvalidRequest(format!("invalid location: {}", command.location)))?;

        // 新ユニットの構築（指定がある場合）
        let new_unit = if let Some(ref spec) = command.unit {
            let power = Power::from_symbol(&spec.power_symbol)
                .ok_or_else(|| SetUnitError::InvalidRequest(format!("invalid power: {}", spec.power_symbol)))?;

            let unit = match spec.kind_str.to_lowercase().as_str() {
                "a" | "army" => {
                    // 陸軍は海域に配置できない
                    if location.is_water() {
                        return Err(SetUnitError::InvalidRequest(
                            "army cannot be placed in a sea province".to_string(),
                        ));
                    }
                    // 陸軍は海岸バリアントコードを許可しない
                    if location.code_with_coast() != location.code() {
                        return Err(SetUnitError::InvalidRequest(
                            "army cannot be placed on a coast variant location".to_string(),
                        ));
                    }
                    Unit::new_army(power, location)
                }
                "f" | "fleet" => {
                    // 海軍は内陸に配置できない
                    if location.kind() == "Inland" {
                        return Err(SetUnitError::InvalidRequest(
                            "fleet cannot be placed in an inland province".to_string(),
                        ));
                    }
                    // 海軍は双海岸地域（例: spa）に直接配置できない
                    if location.code_with_coast() == location.code() && location.has_coast_variants() {
                        return Err(SetUnitError::InvalidRequest(
                            "fleet must specify a coast variant for this location".to_string(),
                        ));
                    }
                    Unit::new_fleet(power, location)
                }
                _ => {
                    return Err(SetUnitError::InvalidRequest(format!("invalid unit kind: {}", spec.kind_str)));
                }
            };
            Some(unit)
        } else {
            None
        };

        let mut updated_game = game;
        let phase = updated_game.phases.last_mut().expect("phase exists");

        // 既存ユニット検索（ベースコードで一致）
        let old_unit = phase.units.iter().find(|u| u.location.code() == location.code()).copied();

        let changed = old_unit != new_unit;
        if !changed {
            return Ok(SetUnitResult {
                game: updated_game,
                old_unit,
                new_unit,
                changed: false,
            });
        }

        // 既存ユニットを削除（ベースコード一致するものすべて）
        if old_unit.is_some() {
            phase.units.retain(|u| u.location.code() != location.code());

            // 削除ユニットに関する命令もすべて削除
            phase.orders.retain(|o| {
                // 直接命令（削除ユニット自身が主体）
                if o.unit.location.code() == location.code() {
                    return false;
                }
                // 仮定命令（支援・輸送の対象が削除ユニット）
                match o.kind {
                    crate::domain::OrderKind::Support(s) => s.target_unit.location.code() != location.code(),
                    crate::domain::OrderKind::Convoy(c) => c.target_unit.location.code() != location.code(),
                    _ => true,
                }
            });
        }

        // 新ユニットを配置し、Hold 命令を生成
        if let Some(unit) = new_unit {
            phase.units.push(unit);
            phase.orders.push(crate::domain::Order::new_hold(unit.power, unit));
        }

        self.game_repository.update(&updated_game).map_err(SetUnitError::Repository)?;

        Ok(SetUnitResult {
            game: updated_game,
            old_unit,
            new_unit,
            changed: true,
        })
    }

    ///
    /// 卓主権限で占領情報を設定・削除する
    ///
    pub(crate) fn set_territory(&self, command: SetTerritoryCommand) -> Result<SetTerritoryResult, SetTerritoryError> {
        if command.access_token.is_empty() {
            return Err(SetTerritoryError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(&command.access_token)
            .map_err(SetTerritoryError::Repository)?
            .ok_or(SetTerritoryError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(SetTerritoryError::Repository)?
            .ok_or(SetTerritoryError::NotFound)?;

        game.players
            .iter()
            .find(|p| p.is_owner && p.user_uuid == user.uuid)
            .ok_or_else(|| SetTerritoryError::Forbidden("user is not the owner of this game".to_string()))?;

        let latest_phase = game
            .phases
            .last()
            .ok_or_else(|| SetTerritoryError::Forbidden("game has no phases".to_string()))?;
        let is_main_phase = matches!(
            latest_phase.kind,
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        );
        if !is_main_phase {
            return Err(SetTerritoryError::Forbidden(
                "territories can only be set during a main phase".to_string(),
            ));
        }

        if game.current_turn() != command.season {
            return Err(SetTerritoryError::PhaseConflict);
        }

        let province = Province::from_code(&command.code)
            .ok_or_else(|| SetTerritoryError::InvalidRequest(format!("invalid code: {}", command.code)))?;

        if province.is_water() {
            return Err(SetTerritoryError::WaterProvince);
        }

        let base_code = province.code();

        let new_power = if let Some(ref symbol) = command.power {
            let p = Power::from_symbol(symbol)
                .ok_or_else(|| SetTerritoryError::InvalidRequest(format!("invalid power: {}", symbol)))?;
            Some(p)
        } else {
            None
        };

        let mut updated_game = game;
        let phase = updated_game.phases.last_mut().expect("phase exists");

        let old_power = phase.territories.iter().find(|t| t.code() == base_code).map(|t| t.power);

        let changed = old_power != new_power;
        if !changed {
            return Ok(SetTerritoryResult {
                game: updated_game,
                province,
                old_power,
                new_power,
                changed: false,
            });
        }

        phase.territories.retain(|t| t.code() != base_code);

        if let Some(power) = new_power {
            phase.territories.push(Territory::new(power, base_code));
        }

        self.game_repository
            .update(&updated_game)
            .map_err(SetTerritoryError::Repository)?;

        Ok(SetTerritoryResult {
            game: updated_game,
            province,
            old_power,
            new_power,
            changed: true,
        })
    }

    ///
    /// 卓主権限で進行モードを合意進行に変更する
    ///
    pub(crate) fn set_progress_mode(
        &self,
        command: SetProgressModeCommand,
    ) -> Result<SetProgressModeResult, SetProgressModeError> {
        if command.access_token.is_empty() {
            return Err(SetProgressModeError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(&command.access_token)
            .map_err(SetProgressModeError::Repository)?
            .ok_or(SetProgressModeError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(SetProgressModeError::Repository)?
            .ok_or(SetProgressModeError::NotFound)?;

        game.players
            .iter()
            .find(|p| p.is_owner && p.user_uuid == user.uuid)
            .ok_or_else(|| SetProgressModeError::Forbidden("user is not the owner of this game".to_string()))?;

        let latest_phase = game
            .phases
            .last()
            .ok_or_else(|| SetProgressModeError::Forbidden("game has no phases".to_string()))?;
        let is_main_phase = matches!(
            latest_phase.kind,
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        );
        if !is_main_phase {
            return Err(SetProgressModeError::Forbidden(
                "progress mode can only be changed during a main phase".to_string(),
            ));
        }

        if game.current_turn() != command.season {
            return Err(SetProgressModeError::PhaseConflict);
        }

        // 既に Consensus なら変更なしで OK を返す
        if game.regulation.progress_mode == crate::domain::ProgressMode::Consensus {
            return Ok(SetProgressModeResult { game, changed: false });
        }

        let mut updated_game = game;
        updated_game.regulation.progress_mode = crate::domain::ProgressMode::Consensus;

        self.game_repository
            .update(&updated_game)
            .map_err(SetProgressModeError::Repository)?;

        Ok(SetProgressModeResult {
            game: updated_game,
            changed: true,
        })
    }

    ///
    /// プレイヤー権限で即時進行への合意フラグを設定する
    ///
    pub(crate) fn set_progress_consensus(
        &self,
        command: SetProgressConsensusCommand,
    ) -> Result<SetProgressConsensusResult, SetProgressConsensusError> {
        let access_token = command.access_token.trim().to_string();
        if access_token.is_empty() {
            return Err(SetProgressConsensusError::Unauthorized);
        }

        let user = self
            .user_repository
            .find_by_access_token(&access_token)
            .map_err(SetProgressConsensusError::Repository)?
            .ok_or(SetProgressConsensusError::Unauthorized)?;

        let game = self
            .game_repository
            .find_by_uuid(command.game_uuid)
            .map_err(SetProgressConsensusError::Repository)?
            .ok_or(SetProgressConsensusError::NotFound)?;

        if game.regulation.progress_mode != crate::domain::ProgressMode::Consensus {
            return Err(SetProgressConsensusError::Forbidden(
                "progress consensus is available only when progress mode is consensus".to_string(),
            ));
        }

        let actor = game
            .players
            .iter()
            .find(|p| p.user_uuid == user.uuid)
            .ok_or_else(|| SetProgressConsensusError::Forbidden("user is not a player of this game".to_string()))?;

        let actor_power = actor
            .power
            .ok_or_else(|| SetProgressConsensusError::Forbidden("player has no assigned power yet".to_string()))?;

        let latest_phase = game
            .phases
            .last()
            .ok_or_else(|| SetProgressConsensusError::Forbidden("game has no phases".to_string()))?;
        let is_main_phase = matches!(
            latest_phase.kind,
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_)
        );
        let active_powers = Self::consensus_required_powers(latest_phase);

        if !active_powers.contains(&actor_power) {
            return Err(SetProgressConsensusError::Forbidden(
                "player is not an active power in the current phase".to_string(),
            ));
        }

        let mut updated_game = game;

        let mut changed = false;
        if let Some(actor_player) = updated_game.players.iter_mut().find(|p| p.user_uuid == user.uuid)
            && actor_player.progress_consented != command.agreed
        {
            actor_player.progress_consented = command.agreed;
            changed = true;
        }

        let mut reached_consensus_in_main_phase = false;
        if changed && command.agreed {
            let all_active_consented = updated_game
                .players
                .iter()
                .filter_map(|p| p.power)
                .filter(|power| active_powers.contains(power))
                .all(|power| {
                    updated_game
                        .players
                        .iter()
                        .any(|p| p.power == Some(power) && p.progress_consented)
                });

            if !active_powers.is_empty() && all_active_consented {
                let idle_threshold = chrono::Utc::now().naive_utc()
                    - chrono::Duration::minutes(i64::from(updated_game.regulation.duration_type.idle_limit_minutes()));
                updated_game.next_update_at = Some(chrono::Utc::now().naive_utc());
                for player in &mut updated_game.players {
                    if player.power.is_none() {
                        continue;
                    }
                    let is_idle = match self
                        .user_repository
                        .find_by_uuid(player.user_uuid)
                        .map_err(SetProgressConsensusError::Repository)?
                    {
                        Some(user_record) => user_record.last_access_at.naive_utc() <= idle_threshold,
                        None => true,
                    };
                    if !is_idle {
                        player.progress_consented = false;
                    }
                }

                reached_consensus_in_main_phase = is_main_phase;
            }
        }

        if !changed && !reached_consensus_in_main_phase {
            return Ok(SetProgressConsensusResult {
                game: updated_game,
                agreed: command.agreed,
                actor_power,
                changed: false,
                reached_consensus_in_main_phase: false,
            });
        }

        self.game_repository
            .update(&updated_game)
            .map_err(SetProgressConsensusError::Repository)?;

        Ok(SetProgressConsensusResult {
            game: updated_game,
            agreed: command.agreed,
            actor_power,
            changed,
            reached_consensus_in_main_phase,
        })
    }

    fn consensus_required_powers(latest_phase: &Phase) -> std::collections::HashSet<Power> {
        match latest_phase.kind {
            crate::domain::PhaseKind::SpringMain(_) | crate::domain::PhaseKind::FallMain(_) => {
                latest_phase.territories.iter().map(|t| t.power).collect()
            }
            crate::domain::PhaseKind::SpringRetreat(_) | crate::domain::PhaseKind::FallRetreat(_) => latest_phase
                .orders
                .iter()
                .filter(|o| o.dislodged_from.is_some())
                .map(|o| o.power)
                .collect(),
            crate::domain::PhaseKind::Adjustment(_) => Power::iter()
                .filter(|power| {
                    let supply_center_count = latest_phase.territories.iter().filter(|t| t.power == *power).count();
                    let unit_count = latest_phase.units.iter().filter(|u| u.power == *power).count();
                    supply_center_count != unit_count
                })
                .collect(),
            crate::domain::PhaseKind::Ready(_) | crate::domain::PhaseKind::Debrief(_) => std::collections::HashSet::new(),
        }
    }

    ///
    /// ステータスフィルタで卓一覧をページネーションして返す
    ///
    pub(crate) fn list_games_by_status(
        &self,
        filter: GameStatusFilter,
        page: u32,
        per_page: u32,
    ) -> Result<ListGamesResult, ListGamesError> {
        let (games, total) = self
            .game_repository
            .find_paginated_by_status(filter, page, per_page)
            .map_err(ListGamesError::Repository)?;
        Ok(ListGamesResult { games, total })
    }

    ///
    /// 指定ユーザーが参加している卓一覧をページネーションして返す
    ///
    pub(crate) fn list_games_by_user(
        &self,
        access_token: &str,
        target_discord_user_id: &str,
        page: u32,
        per_page: u32,
    ) -> Result<ListGamesResult, ListGamesError> {
        let user = self
            .user_repository
            .find_by_access_token(access_token)
            .map_err(ListGamesError::Repository)?
            .ok_or(ListGamesError::Unauthorized)?;

        if user.discord_user_id != target_discord_user_id.trim() {
            return Err(ListGamesError::Forbidden("you can only query your own games".to_string()));
        }

        let (games, total) = self
            .game_repository
            .find_paginated_by_user_uuid(user.uuid, page, per_page)
            .map_err(ListGamesError::Repository)?;

        Ok(ListGamesResult { games, total })
    }
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
    use chrono::Datelike;
    use chrono::Timelike;
    use chrono::Utc;

    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::ProgressMode;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        rows: Rc<RefCell<HashMap<String, UserRecord>>>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows
                .into_iter()
                .map(|row| (row.access_token.clone(), row))
                .collect::<HashMap<_, _>>();
            Self {
                rows: Rc::new(RefCell::new(map)),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_uuid(&self, user_uuid: Uuid) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows.borrow().values().find(|row| row.uuid == user_uuid).cloned())
        }

        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows.borrow().get(access_token).cloned())
        }

        fn update_last_access_at_by_access_token(
            &self,
            access_token: &str,
            last_access_at: DateTime<Utc>,
        ) -> Result<bool, RepositoryError> {
            let mut rows = self.rows.borrow_mut();
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
        created: Rc<RefCell<Vec<Game>>>,
        active_games: Rc<RefCell<Vec<Game>>>,
        updated: Rc<RefCell<Vec<Game>>>,
    }

    impl InMemoryGameRepository {
        fn new(active_games: Vec<Game>) -> Self {
            Self {
                created: Rc::new(RefCell::new(Vec::new())),
                active_games: Rc::new(RefCell::new(active_games)),
                updated: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn created_len(&self) -> usize {
            self.created.borrow().len()
        }

        fn updated_first(&self) -> Option<Game> {
            self.updated.borrow().first().cloned()
        }
    }

    impl GameRepository for InMemoryGameRepository {
        fn add_player(&self, game_uuid: Uuid, user_uuid: Uuid, requested_power: Option<Power>) -> Result<(), RepositoryError> {
            let mut games = self.active_games.borrow_mut();
            if let Some(game) = games.iter_mut().find(|g| g.uuid == game_uuid) {
                if !game.players.iter().any(|p| p.user_uuid == user_uuid) {
                    game.players.push(Player {
                        user_uuid,
                        power: None,
                        is_accepting_draw: false,
                        progress_consented: false,
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
            self.created.borrow_mut().push(new_game.game.clone());
            Ok(new_game.game)
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(self.active_games.borrow().clone())
        }

        fn find_by_uuid(&self, game_uuid: Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(self.active_games.borrow().iter().find(|g| g.uuid == game_uuid).cloned())
        }

        fn find_progress_candidates(&self, _now: chrono::NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(Vec::new())
        }

        fn exists_active_game_for_user(&self, user_uuid: Uuid) -> Result<bool, RepositoryError> {
            let exists = self.active_games.borrow().iter().any(|game| {
                game.players.iter().any(|player| player.user_uuid == user_uuid)
                    && game.status != GameStatus::Solo
                    && game.status != GameStatus::Draw
                    && game.status != GameStatus::Closed
                    && game.status != GameStatus::Aborted
            });
            Ok(exists)
        }

        fn update(&self, game: &Game) -> Result<(), RepositoryError> {
            self.updated.borrow_mut().push(game.clone());
            // active_games も更新して find_by_uuid が最新状態を返せるようにする
            let mut games = self.active_games.borrow_mut();
            if let Some(existing) = games.iter_mut().find(|g| g.uuid == game.uuid) {
                *existing = game.clone();
            }
            Ok(())
        }

        fn assign_game_number(&self, game_uuid: Uuid) -> Result<i32, RepositoryError> {
            // 既に採番済みか確認
            if let Some(n) = self
                .active_games
                .borrow()
                .iter()
                .find(|g| g.uuid == game_uuid)
                .and_then(|g| g.game_number)
            {
                return Ok(n);
            }
            // 現在の最大値を取得
            let max = self
                .active_games
                .borrow()
                .iter()
                .filter_map(|g| g.game_number)
                .max()
                .unwrap_or(0);
            let next = max + 1;
            let mut games = self.active_games.borrow_mut();
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
                .active_games
                .borrow()
                .iter()
                .filter(|g| match filter {
                    GameStatusFilter::Active => !matches!(g.status, GameStatus::Closed | GameStatus::Aborted),
                    GameStatusFilter::Closed => matches!(g.status, GameStatus::Closed),
                    GameStatusFilter::Aborted => matches!(g.status, GameStatus::Aborted),
                })
                .map(game_to_summary)
                .collect();
            let total = all.len() as u64;
            let start = (page as usize).saturating_sub(1) * per_page as usize;
            let items = all.into_iter().skip(start).take(per_page as usize).collect();
            Ok((items, total))
        }

        fn find_paginated_by_user_uuid(
            &self,
            user_uuid: Uuid,
            page: u32,
            per_page: u32,
        ) -> Result<(Vec<GameSummary>, u64), RepositoryError> {
            let all: Vec<GameSummary> = self
                .active_games
                .borrow()
                .iter()
                .filter(|g| g.players.iter().any(|p| p.user_uuid == user_uuid))
                .map(game_to_summary)
                .collect();
            let total = all.len() as u64;
            let start = (page as usize).saturating_sub(1) * per_page as usize;
            let items = all.into_iter().skip(start).take(per_page as usize).collect();
            Ok((items, total))
        }
    }

    fn game_to_summary(game: &Game) -> GameSummary {
        GameSummary {
            uuid: game.uuid,
            game_number: game.game_number,
            status: game.status,
            next_update_at: game.next_update_at,
            regulation: game.regulation,
            player_count: game.players.len() as u64,
            season_label: game.current_season_label(),
        }
    }

    fn sample_regulation() -> Regulation {
        let jst = FixedOffset::east_opt(9 * 60 * 60).expect("valid JST offset");
        let now_jst = Utc::now().with_timezone(&jst);
        let start_jst = now_jst + chrono::Duration::hours(2);

        Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            start_jst.date_naive(),
            start_jst.hour() as u8,
        )
        .expect("valid regulation")
    }

    #[test]
    fn create_game_creates_owner_player_and_ready_phase() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game_repository = InMemoryGameRepository::new(vec![]);

        let service = GameService::new(user_repository, game_repository);

        let result = service
            .create_game(CreateGameCommand {
                access_token: "token-1".to_string(),
                regulation: sample_regulation(),
                requested_power: Some(Power::France),
                keyword: None,
            })
            .expect("create game should succeed");

        assert_eq!(result.owner_user_uuid, user_uuid);
        assert_eq!(result.requested_power, Some(Power::France));
        assert_eq!(result.game.players.len(), 1);
        let owner = &result.game.players[0];
        assert_eq!(owner.user_uuid, user_uuid);
        assert!(owner.is_owner);
        assert_eq!(owner.requested_power, Some(Power::France));
        assert_eq!(result.game.phases.len(), 1);
    }

    #[test]
    fn create_game_converts_initial_next_update_from_jst_to_utc() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let result = service
            .create_game(CreateGameCommand {
                access_token: "token-1".to_string(),
                regulation: sample_regulation(),
                requested_power: None,
                keyword: None,
            })
            .expect("create game should succeed");

        let jst = FixedOffset::east_opt(9 * 60 * 60).expect("valid JST offset");
        let expected = jst
            .with_ymd_and_hms(
                result.game.regulation.start_date.year(),
                result.game.regulation.start_date.month(),
                result.game.regulation.start_date.day(),
                result.game.regulation.first_period_hour as u32,
                0,
                0,
            )
            .single()
            .expect("valid JST datetime")
            .with_timezone(&Utc)
            .naive_utc();
        assert_eq!(result.game.next_update_at, Some(expected));
    }

    #[test]
    fn create_game_rejects_past_start_datetime() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);
        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let past_regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid date"),
            0,
        )
        .expect("valid regulation");

        let error = service
            .create_game(CreateGameCommand {
                access_token: "token-1".to_string(),
                regulation: past_regulation,
                requested_power: None,
                keyword: None,
            })
            .expect_err("create game should fail");

        assert!(matches!(error, CreateGameError::InvalidRequest(_)));
    }

    #[test]
    fn create_game_rejects_when_user_already_participates_in_active_game() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let participating_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(1),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: false,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let game_repository = InMemoryGameRepository::new(vec![participating_game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let error = service
            .create_game(CreateGameCommand {
                access_token: "token-1".to_string(),
                regulation: sample_regulation(),
                requested_power: None,
                keyword: None,
            })
            .expect_err("create game should fail");

        assert!(matches!(error, CreateGameError::Forbidden(_)));
        assert_eq!(game_repository.created_len(), 0);
    }

    #[test]
    fn create_game_allows_when_participating_game_is_solo_or_draw() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let solo_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(1),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: false,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Solo,
            is_draw: false,
            is_solo: true,
            next_update_at: None,
        };

        let game_repository = InMemoryGameRepository::new(vec![solo_game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service.create_game(CreateGameCommand {
            access_token: "token-1".to_string(),
            regulation: sample_regulation(),
            requested_power: None,
            keyword: None,
        });

        assert!(result.is_ok());
        assert_eq!(game_repository.created_len(), 1);
    }

    #[test]
    fn create_game_allows_when_participating_game_is_draw() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let draw_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(1),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid,
                power: Some(Power::France),
                is_accepting_draw: true,
                progress_consented: false,
                is_owner: false,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Draw,
            is_draw: true,
            is_solo: false,
            next_update_at: None,
        };

        let game_repository = InMemoryGameRepository::new(vec![draw_game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service.create_game(CreateGameCommand {
            access_token: "token-1".to_string(),
            regulation: sample_regulation(),
            requested_power: None,
            keyword: None,
        });

        assert!(result.is_ok());
        assert_eq!(game_repository.created_len(), 1);
    }

    #[test]
    fn create_game_allows_when_participating_game_is_aborted() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let canceled_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(1),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: false,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Aborted,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let game_repository = InMemoryGameRepository::new(vec![canceled_game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service.create_game(CreateGameCommand {
            access_token: "token-1".to_string(),
            regulation: sample_regulation(),
            requested_power: None,
            keyword: None,
        });

        assert!(result.is_ok());
        assert_eq!(game_repository.created_len(), 1);
    }

    #[test]
    fn validate_start_datetime_rejects_when_within_30_minutes() {
        let now = chrono::NaiveDate::from_ymd_opt(2026, 4, 24)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid datetime");
        let start = now + chrono::Duration::minutes(30);

        let result = GameService::<InMemoryUserRepository, InMemoryGameRepository>::validate_start_datetime(start, now);
        assert!(matches!(result, Err(CreateGameError::InvalidRequest(_))));
    }

    #[test]
    fn validate_start_datetime_accepts_when_more_than_30_minutes() {
        let now = chrono::NaiveDate::from_ymd_opt(2026, 4, 24)
            .expect("valid date")
            .and_hms_opt(0, 0, 0)
            .expect("valid datetime");
        let start = now + chrono::Duration::minutes(31);

        let result = GameService::<InMemoryUserRepository, InMemoryGameRepository>::validate_start_datetime(start, now);
        assert!(result.is_ok());
    }

    fn sample_preparing_game(player_user_uuid: Uuid) -> Game {
        Game {
            uuid: Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid: player_user_uuid,
                power: None,
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: true,
                requested_power: None,
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        }
    }

    #[test]
    fn join_game_registers_player() {
        let owner_uuid = Uuid::now_v7();
        let joiner_uuid = Uuid::now_v7();
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

        let game = sample_preparing_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .join_game(JoinGameCommand {
                access_token: "token-2".to_string(),
                game_uuid,
                requested_power: Some(Power::France),
                keyword: None,
            })
            .expect("join game should succeed");

        assert_eq!(result.user_uuid, joiner_uuid);
        assert_eq!(result.requested_power, Some(Power::France));
        assert_eq!(result.game.players.len(), 2);

        let joined_player = result
            .game
            .players
            .iter()
            .find(|p| p.user_uuid == joiner_uuid)
            .expect("joiner should be in players");
        assert!(!joined_player.is_owner);
        assert_eq!(joined_player.requested_power, Some(Power::France));
        assert!(joined_player.power.is_none());

        let updated = game_repository.updated_first().expect("game should have been updated");
        assert_eq!(updated.players.len(), 2);
    }

    #[test]
    fn join_game_rejects_unauthorized() {
        let user_repository = InMemoryUserRepository::new(vec![]);
        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "invalid-token".to_string(),
                game_uuid: Uuid::now_v7(),
                requested_power: None,
                keyword: None,
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::Unauthorized));
    }

    #[test]
    fn join_game_rejects_when_user_already_participates_in_active_game() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let active_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(1),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: false,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };
        let target_game = sample_preparing_game(Uuid::now_v7());
        let target_uuid = target_game.uuid;

        let game_repository = InMemoryGameRepository::new(vec![active_game, target_game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid: target_uuid,
                requested_power: None,
                keyword: None,
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::Forbidden(_)));
    }

    #[test]
    fn join_game_rejects_when_game_not_found() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid: Uuid::now_v7(),
                requested_power: None,
                keyword: None,
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::NotFound));
    }

    #[test]
    fn join_game_rejects_when_game_is_not_preparing() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "asagi".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let in_progress_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(2),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid: Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };
        let game_uuid = in_progress_game.uuid;

        let game_repository = InMemoryGameRepository::new(vec![in_progress_game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid,
                requested_power: None,
                keyword: None,
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::Forbidden(_)));
    }

    /// 6 人分のプレイヤー（卓主 + 5 人）が参加済みのゲームを生成するヘルパー
    fn sample_six_player_game(owner_uuid: Uuid, user_records: &mut Vec<UserRecord>) -> Game {
        use strum::IntoEnumIterator;
        let powers: Vec<Power> = Power::iter().collect();
        let mut game = Game {
            uuid: Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid: owner_uuid,
                power: None,
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: true,
                requested_power: Some(powers[0]),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        for power in powers.iter().skip(1).take(5) {
            let uuid = Uuid::now_v7();
            user_records.push(UserRecord {
                id: user_records.len() as i64 + 2,
                uuid,
                discord_user_id: format!("discord-{}", uuid),
                username: format!("user-{}", uuid),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: format!("token-{}", uuid),
                last_access_at: Utc::now(),
            });
            game.players.push(Player {
                user_uuid: uuid,
                power: None,
                is_accepting_draw: false,
                progress_consented: false,
                is_owner: false,
                requested_power: Some(*power),
            });
        }

        game
    }

    #[test]
    fn join_game_sets_status_to_ready_when_seventh_player_joins() {
        let owner_uuid = Uuid::now_v7();
        let seventh_uuid = Uuid::now_v7();

        let mut extra_users: Vec<UserRecord> = vec![UserRecord {
            id: 1,
            uuid: owner_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }];

        let game = sample_six_player_game(owner_uuid, &mut extra_users);
        let game_uuid = game.uuid;

        // 7 人目のユーザー
        extra_users.push(UserRecord {
            id: 8,
            uuid: seventh_uuid,
            discord_user_id: "discord-7".to_string(),
            username: "seventh".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-7".to_string(),
            last_access_at: Utc::now(),
        });

        let user_repository = InMemoryUserRepository::new(extra_users);
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .join_game(JoinGameCommand {
                access_token: "token-7".to_string(),
                game_uuid,
                requested_power: Some(Power::Turkey),
                keyword: None,
            })
            .expect("join game should succeed");

        assert_eq!(result.game.status, GameStatus::Ready, "ステータスが Ready になるべき");
        assert_eq!(result.game.players.len(), 7, "プレイヤーが 7 人になるべき");
        assert!(result.game.game_number.is_some(), "卓番号が割り当てられるべき");

        // 全員に担当国が割り当てられている
        assert!(
            result.game.players.iter().all(|p| p.power.is_some()),
            "全プレイヤーに担当国が割り当てられるべき"
        );

        // 担当国に重複なし
        use std::collections::HashSet;
        let assigned: HashSet<Power> = result.game.players.iter().filter_map(|p| p.power).collect();
        assert_eq!(assigned.len(), 7, "担当国は重複しないべき");
    }

    #[test]
    fn join_game_does_not_change_status_when_fewer_than_seven_players() {
        let owner_uuid = Uuid::now_v7();
        let joiner_uuid = Uuid::now_v7();
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
                uuid: joiner_uuid,
                discord_user_id: "discord-2".to_string(),
                username: "joiner".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-2".to_string(),
                last_access_at: Utc::now(),
            },
        ]);

        let game = sample_preparing_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let result = service
            .join_game(JoinGameCommand {
                access_token: "token-2".to_string(),
                game_uuid,
                requested_power: None,
                keyword: None,
            })
            .expect("join game should succeed");

        assert_eq!(
            result.game.status,
            GameStatus::Preparing,
            "7 人未満なら Preparing のままであるべき"
        );
        assert!(result.game.game_number.is_none(), "7 人未満では卓番号が割り当てられないべき");
    }

    #[test]
    fn join_game_rejects_when_keyword_does_not_match() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "joiner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let owner_uuid = Uuid::now_v7();
        let mut game = sample_preparing_game(owner_uuid);
        game.keyword = Some("secret".to_string());
        let game_uuid = game.uuid;

        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid,
                requested_power: None,
                keyword: Some("wrong".to_string()),
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::Forbidden(_)));
    }

    #[test]
    fn join_game_rejects_when_keyword_is_missing() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "joiner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let owner_uuid = Uuid::now_v7();
        let mut game = sample_preparing_game(owner_uuid);
        game.keyword = Some("secret".to_string());
        let game_uuid = game.uuid;

        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid,
                requested_power: None,
                keyword: None,
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::Forbidden(_)));
    }

    #[test]
    fn join_game_succeeds_when_keyword_matches() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "joiner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let owner_uuid = Uuid::now_v7();
        let mut game = sample_preparing_game(owner_uuid);
        game.keyword = Some("secret".to_string());
        let game_uuid = game.uuid;

        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid,
                requested_power: None,
                keyword: Some("secret".to_string()),
            })
            .expect("join game should succeed");

        assert_eq!(result.user_uuid, user_uuid);
        assert_eq!(result.game.players.len(), 2);
    }

    #[test]
    fn join_game_rejects_when_game_has_no_keyword_but_request_has_keyword() {
        let user_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "1001".to_string(),
            username: "joiner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-1".to_string(),
            last_access_at: Utc::now(),
        }]);

        let owner_uuid = Uuid::now_v7();
        let game = sample_preparing_game(owner_uuid); // keyword: None
        let game_uuid = game.uuid;

        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .join_game(JoinGameCommand {
                access_token: "token-1".to_string(),
                game_uuid,
                requested_power: None,
                keyword: Some("unexpected".to_string()),
            })
            .expect_err("join game should fail");

        assert!(matches!(error, JoinGameError::Forbidden(_)));
    }

    // ============================================================================
    // set_draw_proposal tests
    // ============================================================================

    /// SpringMain フェイズを持つゲームを返すヘルパー
    fn sample_in_progress_game(owner_uuid: Uuid) -> Game {
        let mut game = sample_preparing_game(owner_uuid);
        game.status = GameStatus::InProgress;
        // Ready フェイズの後に SpringMain フェイズを追加
        let ready = game.phases.last().unwrap().clone();
        let spring_main = Phase::new_spring_main(ready.year, ready.index);
        game.phases.push(spring_main);
        game
    }

    fn owner_user_record(user_uuid: Uuid) -> UserRecord {
        UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: "discord-owner".to_string(),
            username: "owner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: "token-owner".to_string(),
            last_access_at: Utc::now(),
        }
    }

    fn player_user_record(user_uuid: Uuid, token: &str) -> UserRecord {
        UserRecord {
            id: 2,
            uuid: user_uuid,
            discord_user_id: "discord-player".to_string(),
            username: "player".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: token.to_string(),
            last_access_at: Utc::now(),
        }
    }

    fn sample_consensus_game_with_two_active_players(owner_uuid: Uuid, player_uuid: Uuid) -> Game {
        let mut game = sample_in_progress_game(owner_uuid);
        game.regulation.progress_mode = ProgressMode::Consensus;

        game.players[0].power = Some(Power::France);
        game.players.push(Player {
            user_uuid: player_uuid,
            power: Some(Power::England),
            is_accepting_draw: false,
            progress_consented: false,
            is_owner: false,
            requested_power: None,
        });

        let phase = game.phases.last_mut().expect("phase exists");
        phase.territories.clear();
        phase.territories.push(crate::domain::Territory::new(Power::France, "par"));
        phase.territories.push(crate::domain::Territory::new(Power::England, "lon"));

        game
    }

    fn sample_consensus_spring_retreat_game(owner_uuid: Uuid, player_uuid: Uuid, optional_uuid: Uuid) -> Game {
        let mut game = sample_consensus_game_with_two_active_players(owner_uuid, player_uuid);
        game.players.push(Player {
            user_uuid: optional_uuid,
            power: Some(Power::Germany),
            is_accepting_draw: false,
            progress_consented: false,
            is_owner: false,
            requested_power: None,
        });

        let mut dislodged_france_unit = Unit::new_army(Power::France, Province::from_code("par").expect("valid province"));
        dislodged_france_unit.set_dislodged_from(Some(Province::from_code("bur").expect("valid province")));

        let mut retreat_phase = Phase::new_spring_retreat(1901, 0);
        retreat_phase.orders = vec![dislodged_france_unit.disband()];
        retreat_phase.units = vec![dislodged_france_unit];
        retreat_phase.territories = vec![Territory::new(Power::France, "par"), Territory::new(Power::England, "lon")];

        game.phases = vec![retreat_phase];
        game
    }

    fn sample_consensus_adjustment_game(owner_uuid: Uuid, player_uuid: Uuid, optional_uuid: Uuid) -> Game {
        let mut game = sample_consensus_game_with_two_active_players(owner_uuid, player_uuid);
        game.players.push(Player {
            user_uuid: optional_uuid,
            power: Some(Power::Germany),
            is_accepting_draw: false,
            progress_consented: false,
            is_owner: false,
            requested_power: None,
        });

        let mut adjustment_phase = Phase::new_adjustment(1901, 1);
        adjustment_phase.orders = vec![];
        adjustment_phase.units = vec![
            Unit::new_army(Power::France, Province::from_code("par").expect("valid province")),
            Unit::new_army(Power::England, Province::from_code("lon").expect("valid province")),
        ];
        adjustment_phase.territories = vec![
            Territory::new(Power::France, "par"),
            Territory::new(Power::France, "mar"),
            Territory::new(Power::England, "lon"),
        ];

        game.phases = vec![adjustment_phase];
        game
    }

    #[test]
    fn set_draw_proposal_rejects_empty_access_token() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "".to_string(),
                game_uuid,
                draw_proposal: true,
            })
            .expect_err("should reject empty token");

        assert!(matches!(error, SetDrawProposalError::Unauthorized));
    }

    #[test]
    fn set_draw_proposal_rejects_unknown_access_token() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "unknown-token".to_string(),
                game_uuid,
                draw_proposal: true,
            })
            .expect_err("should reject unknown token");

        assert!(matches!(error, SetDrawProposalError::Unauthorized));
    }

    #[test]
    fn set_draw_proposal_rejects_when_game_not_found() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "token-owner".to_string(),
                game_uuid: Uuid::now_v7(),
                draw_proposal: true,
            })
            .expect_err("should reject when game not found");

        assert!(matches!(error, SetDrawProposalError::NotFound));
    }

    #[test]
    fn set_draw_proposal_rejects_non_owner() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
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
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "token-other".to_string(),
                game_uuid,
                draw_proposal: true,
            })
            .expect_err("should reject non-owner");

        assert!(matches!(error, SetDrawProposalError::Forbidden(_)));
    }

    #[test]
    fn set_draw_proposal_rejects_when_not_main_phase() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        // Ready フェイズのみ（メインフェイズでない）
        let game = sample_preparing_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                draw_proposal: true,
            })
            .expect_err("should reject when not main phase");

        assert!(matches!(error, SetDrawProposalError::Forbidden(_)));
    }

    #[test]
    fn set_draw_proposal_returns_unchanged_when_already_same_value() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let mut game = sample_in_progress_game(owner_uuid);
        // オーナーの is_accepting_draw を true に設定済み
        game.players.iter_mut().find(|p| p.is_owner).unwrap().is_accepting_draw = true;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                draw_proposal: true,
            })
            .expect("should succeed");

        assert!(!result.changed, "changed should be false when value is unchanged");
        // update() は呼ばれないはず
        assert!(game_repository.updated_first().is_none(), "update should not be called");
    }

    #[test]
    fn set_draw_proposal_sets_flag_and_returns_changed_true() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_draw_proposal(SetDrawProposalCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                draw_proposal: true,
            })
            .expect("should succeed");

        assert!(result.changed, "changed should be true when flag is updated");
        assert!(
            result.game.players.iter().find(|p| p.is_owner).unwrap().is_accepting_draw,
            "owner is_accepting_draw should be true"
        );
        let updated = game_repository.updated_first().expect("game should be updated");
        assert!(
            updated.players.iter().find(|p| p.is_owner).unwrap().is_accepting_draw,
            "persisted game should have is_accepting_draw=true"
        );
    }

    #[test]
    fn set_unit_rejects_empty_access_token() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: None,
                season: "1901s".to_string(),
            })
            .expect_err("should reject empty token");

        assert!(matches!(error, SetUnitError::Unauthorized));
    }

    #[test]
    fn set_unit_rejects_unknown_access_token() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "unknown-token".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: None,
                season: "1901s".to_string(),
            })
            .expect_err("should reject unknown token");

        assert!(matches!(error, SetUnitError::Unauthorized));
    }

    #[test]
    fn set_unit_rejects_when_game_not_found() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid: Uuid::now_v7(),
                location: "par".to_string(),
                unit: None,
                season: "1901s".to_string(),
            })
            .expect_err("should reject when game not found");

        assert!(matches!(error, SetUnitError::NotFound));
    }

    #[test]
    fn set_unit_rejects_non_owner() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
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
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-other".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: None,
                season: "1901s".to_string(),
            })
            .expect_err("should reject non-owner");

        assert!(matches!(error, SetUnitError::Forbidden(_)));
    }

    #[test]
    fn set_unit_rejects_when_not_main_phase() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_preparing_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: None,
                season: "1901s".to_string(),
            })
            .expect_err("should reject when not main phase");

        assert!(matches!(error, SetUnitError::Forbidden(_)));
    }

    #[test]
    fn set_unit_rejects_invalid_location() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "zzz".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "a".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect_err("should reject invalid location");

        assert!(matches!(error, SetUnitError::InvalidRequest(_)));
    }

    #[test]
    fn set_unit_rejects_army_in_sea_province() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        // "mid" = Mid-Atlantic Ocean (Water)
        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "mid".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "a".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect_err("should reject army in sea province");

        assert!(matches!(error, SetUnitError::InvalidRequest(_)));
    }

    #[test]
    fn set_unit_rejects_fleet_in_inland_province() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        // "par" = Paris (Inland)
        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "f".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect_err("should reject fleet in inland province");

        assert!(matches!(error, SetUnitError::InvalidRequest(_)));
    }

    #[test]
    fn set_unit_rejects_fleet_on_dual_coast_base_code() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        // "spa" = Spain (Coast with north/south coast variants)
        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "spa".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "f".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect_err("should reject fleet on dual-coast base code");

        assert!(matches!(error, SetUnitError::InvalidRequest(_)));
    }

    #[test]
    fn set_unit_places_army_and_creates_hold_order() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        // "par" = Paris (Inland) — army can be placed here
        let result = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "a".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect("should succeed");

        let phase = result.game.phases.last().unwrap();
        let unit = phase
            .units
            .iter()
            .find(|u| u.location.code() == "par")
            .expect("unit should be placed");
        assert_eq!(unit.power, Power::France);
        assert_eq!(unit.symbol(), "A");

        // Hold 命令が生成されているか確認
        let hold_order = phase
            .orders
            .iter()
            .find(|o| o.unit.location.code() == "par" && matches!(o.kind, crate::domain::OrderKind::Hold(_)));
        assert!(hold_order.is_some(), "hold order should be created for placed unit");

        assert!(result.old_unit.is_none());
        assert!(result.new_unit.is_some());
    }

    #[test]
    fn set_unit_returns_unchanged_when_same_unit_is_requested() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let mut game = sample_in_progress_game(owner_uuid);
        let paris = Province::from_code("par").expect("par should be a valid province");
        let existing = Unit::new_army(Power::France, paris);
        let phase = game.phases.last_mut().expect("phase should exist");
        phase.units.push(existing);
        phase.orders.push(crate::domain::Order::new_hold(Power::France, existing));

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "a".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect("should succeed");

        assert!(!result.changed, "changed should be false for no-op update");
        assert!(
            game_repository.updated_first().is_none(),
            "update should not be called for no-op"
        );
    }

    #[test]
    fn set_unit_removes_unit_and_its_orders() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let mut game = sample_in_progress_game(owner_uuid);
        let paris = Province::from_code("par").unwrap();
        let unit = Unit::new_army(Power::France, paris);
        let phase = game.phases.last_mut().unwrap();
        phase.units.push(unit);
        phase.orders.push(crate::domain::Order::new_hold(Power::France, unit));

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: None,
                season: "1901s".to_string(),
            })
            .expect("should succeed");

        let phase = result.game.phases.last().unwrap();
        assert!(
            phase.units.iter().all(|u| u.location.code() != "par"),
            "unit should be removed"
        );
        assert!(
            phase.orders.iter().all(|o| o.unit.location.code() != "par"),
            "orders for removed unit should be deleted"
        );
        assert!(result.old_unit.is_some());
        assert!(result.new_unit.is_none());
    }

    #[test]
    fn set_unit_replaces_existing_unit() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let mut game = sample_in_progress_game(owner_uuid);
        let brest = Province::from_code("bre").unwrap();
        let old_unit = Unit::new_army(Power::France, brest);
        let phase = game.phases.last_mut().unwrap();
        phase.units.push(old_unit);
        phase.orders.push(crate::domain::Order::new_hold(Power::France, old_unit));

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        // bre = Brest (Coast) — fleet can be placed here
        let result = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "bre".to_string(),
                unit: Some(UnitSpec {
                    power_symbol: "f".to_string(),
                    kind_str: "f".to_string(),
                }),
                season: "1901s".to_string(),
            })
            .expect("should succeed");

        let phase = result.game.phases.last().unwrap();
        let new_unit = phase
            .units
            .iter()
            .find(|u| u.location.code() == "bre")
            .expect("unit should exist");
        assert_eq!(new_unit.symbol(), "F", "unit should now be a fleet");

        // 古い命令は削除され、新しい Hold 命令のみ
        let hold_orders: Vec<_> = phase.orders.iter().filter(|o| o.unit.location.code() == "bre").collect();
        assert_eq!(hold_orders.len(), 1, "exactly one hold order for new unit");
        assert!(matches!(hold_orders[0].kind, crate::domain::OrderKind::Hold(_)));

        assert!(result.old_unit.is_some(), "old_unit should be Some");
        assert!(result.new_unit.is_some(), "new_unit should be Some");
    }

    #[test]
    fn set_unit_rejects_wrong_season() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_unit(SetUnitCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                location: "par".to_string(),
                unit: None,
                season: "1901f".to_string(), // wrong season (game is 1901s)
            })
            .expect_err("should reject mismatched season");

        assert!(matches!(error, SetUnitError::PhaseConflict));
    }

    #[test]
    fn set_next_update_at_returns_noop_when_time_is_unchanged() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);

        let mut game = sample_in_progress_game(owner_uuid);
        let jst = chrono::FixedOffset::east_opt(9 * 3600).expect("valid JST offset");
        let target_jst_str = "2099-06-15 14:00";
        let target_naive = chrono::NaiveDateTime::parse_from_str(target_jst_str, "%Y-%m-%d %H:%M").expect("valid datetime");
        let target_utc = jst
            .from_local_datetime(&target_naive)
            .single()
            .expect("non-ambiguous JST")
            .with_timezone(&Utc)
            .naive_utc();
        game.next_update_at = Some(target_utc);

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_next_update_at(SetNextUpdateAtCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                next_update_at: target_jst_str.to_string(),
                season: "1901s".to_string(),
            })
            .expect("no-op should succeed");

        assert!(!result.changed, "changed should be false when time is unchanged");
        assert!(game_repository.updated_first().is_none(), "update should not be called");
    }

    #[test]
    fn set_next_update_at_updates_when_time_is_extended() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);

        let mut game = sample_in_progress_game(owner_uuid);
        let jst = chrono::FixedOffset::east_opt(9 * 3600).expect("valid JST offset");
        let current_jst_str = "2099-06-15 13:00";
        let new_jst_str = "2099-06-15 14:00";

        let current_naive =
            chrono::NaiveDateTime::parse_from_str(current_jst_str, "%Y-%m-%d %H:%M").expect("valid current datetime");
        let current_utc = jst
            .from_local_datetime(&current_naive)
            .single()
            .expect("non-ambiguous JST")
            .with_timezone(&Utc)
            .naive_utc();
        game.next_update_at = Some(current_utc);

        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_next_update_at(SetNextUpdateAtCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                next_update_at: new_jst_str.to_string(),
                season: "1901s".to_string(),
            })
            .expect("update should succeed");

        assert!(result.changed, "changed should be true when time is extended");
        assert_eq!(result.next_update_at_jst, new_jst_str);

        let updated = game_repository.updated_first().expect("update should be called");
        let expected_new_utc = jst
            .from_local_datetime(
                &chrono::NaiveDateTime::parse_from_str(new_jst_str, "%Y-%m-%d %H:%M").expect("valid new datetime"),
            )
            .single()
            .expect("non-ambiguous JST")
            .with_timezone(&Utc)
            .naive_utc();
        assert_eq!(updated.next_update_at, Some(expected_new_utc));
    }

    // ============================================================================
    // set_progress_mode tests
    // ============================================================================

    #[test]
    fn set_progress_mode_rejects_empty_access_token() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            })
            .expect_err("should reject empty token");

        assert!(matches!(error, SetProgressModeError::Unauthorized));
    }

    #[test]
    fn set_progress_mode_rejects_unknown_access_token() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "unknown-token".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            })
            .expect_err("should reject unknown token");

        assert!(matches!(error, SetProgressModeError::Unauthorized));
    }

    #[test]
    fn set_progress_mode_rejects_when_game_not_found() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game_repository = InMemoryGameRepository::new(vec![]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "token-owner".to_string(),
                game_uuid: Uuid::now_v7(),
                season: "1901s".to_string(),
            })
            .expect_err("should reject when game not found");

        assert!(matches!(error, SetProgressModeError::NotFound));
    }

    #[test]
    fn set_progress_mode_rejects_non_owner() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
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
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "token-other".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            })
            .expect_err("should reject non-owner");

        assert!(matches!(error, SetProgressModeError::Forbidden(_)));
    }

    #[test]
    fn set_progress_mode_rejects_when_not_main_phase() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        // Ready フェイズのみ（メインフェイズでない）
        let game = sample_preparing_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            })
            .expect_err("should reject when not main phase");

        assert!(matches!(error, SetProgressModeError::Forbidden(_)));
    }

    #[test]
    fn set_progress_mode_rejects_mismatched_season() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                season: "1901f".to_string(), // wrong season (game is 1901s)
            })
            .expect_err("should reject mismatched season");

        assert!(matches!(error, SetProgressModeError::PhaseConflict));
    }

    #[test]
    fn set_progress_mode_returns_unchanged_when_already_consensus() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let mut game = sample_in_progress_game(owner_uuid);
        game.regulation.progress_mode = ProgressMode::Consensus;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            })
            .expect("should succeed");

        assert!(!result.changed, "changed should be false when already consensus");
        assert!(game_repository.updated_first().is_none(), "update should not be called");
    }

    #[test]
    fn set_progress_mode_changes_to_consensus_and_persists() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let game = sample_in_progress_game(owner_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_mode(SetProgressModeCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                season: "1901s".to_string(),
            })
            .expect("should succeed");

        assert!(result.changed, "changed should be true");
        assert_eq!(result.game.regulation.progress_mode, ProgressMode::Consensus);
        let updated = game_repository.updated_first().expect("game should be updated");
        assert_eq!(updated.regulation.progress_mode, ProgressMode::Consensus);
    }

    // ============================================================================
    // set_progress_consensus tests
    // ============================================================================

    #[test]
    fn set_progress_consensus_rejects_when_mode_is_not_consensus() {
        let owner_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![owner_user_record(owner_uuid)]);
        let mut game = sample_in_progress_game(owner_uuid);
        game.players[0].power = Some(Power::France);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect_err("should reject when progress mode is not consensus");

        assert!(matches!(error, SetProgressConsensusError::Forbidden(_)));
    }

    #[test]
    fn set_progress_consensus_rejects_non_player() {
        let owner_uuid = Uuid::now_v7();
        let outsider_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
            player_user_record(outsider_uuid, "token-outsider"),
        ]);
        let mut game = sample_in_progress_game(owner_uuid);
        game.regulation.progress_mode = ProgressMode::Consensus;
        game.players[0].power = Some(Power::France);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository);

        let error = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-outsider".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect_err("should reject non-player");

        assert!(matches!(error, SetProgressConsensusError::Forbidden(_)));
    }

    #[test]
    fn set_progress_consensus_returns_noop_when_state_is_unchanged() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
            player_user_record(other_uuid, "token-player"),
        ]);
        let mut game = sample_consensus_game_with_two_active_players(owner_uuid, other_uuid);
        game.players[0].progress_consented = true;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect("no-op should succeed");

        assert!(!result.changed);
        assert!(!result.reached_consensus_in_main_phase);
        assert!(game_repository.updated_first().is_none(), "update should not be called");
    }

    #[test]
    fn set_progress_consensus_sets_flag_and_persists_without_full_consensus() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
            player_user_record(other_uuid, "token-player"),
        ]);
        let mut game = sample_consensus_game_with_two_active_players(owner_uuid, other_uuid);
        game.players[1].progress_consented = false;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect("should succeed");

        assert!(result.changed);
        assert!(!result.reached_consensus_in_main_phase);
        assert_eq!(result.actor_power, Power::France);

        let updated = game_repository.updated_first().expect("game should be updated");
        assert!(updated.players[0].progress_consented, "actor consent should be true");
    }

    #[test]
    fn set_progress_consensus_reaches_full_consensus_and_resets_non_idle_flags() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
            player_user_record(other_uuid, "token-player"),
        ]);
        let mut game = sample_consensus_game_with_two_active_players(owner_uuid, other_uuid);
        game.players[0].progress_consented = false;
        game.players[1].progress_consented = true;
        game.next_update_at = None;
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect("should succeed");

        assert!(result.changed);
        assert!(result.reached_consensus_in_main_phase);
        let updated = game_repository.updated_first().expect("game should be updated");
        assert!(updated.next_update_at.is_some(), "next_update_at should be set to now");
        assert!(
            updated.players.iter().all(|p| !p.progress_consented),
            "non-idle players should be reset after consensus reached"
        );
    }

    #[test]
    fn set_progress_consensus_reaches_consensus_in_retreat_with_only_required_players() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let optional_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
            player_user_record(other_uuid, "token-player"),
            player_user_record(optional_uuid, "token-optional"),
        ]);
        let game = sample_consensus_spring_retreat_game(owner_uuid, other_uuid, optional_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect("should succeed");

        assert!(result.changed);
        assert!(!result.reached_consensus_in_main_phase);
        let updated = game_repository.updated_first().expect("game should be updated");
        assert!(updated.next_update_at.is_some(), "next_update_at should be set to now");
    }

    #[test]
    fn set_progress_consensus_reaches_consensus_in_adjustment_with_only_required_players() {
        let owner_uuid = Uuid::now_v7();
        let other_uuid = Uuid::now_v7();
        let optional_uuid = Uuid::now_v7();
        let user_repository = InMemoryUserRepository::new(vec![
            owner_user_record(owner_uuid),
            player_user_record(other_uuid, "token-player"),
            player_user_record(optional_uuid, "token-optional"),
        ]);
        let game = sample_consensus_adjustment_game(owner_uuid, other_uuid, optional_uuid);
        let game_uuid = game.uuid;
        let game_repository = InMemoryGameRepository::new(vec![game]);
        let service = GameService::new(user_repository, game_repository.clone());

        let result = service
            .set_progress_consensus(SetProgressConsensusCommand {
                access_token: "token-owner".to_string(),
                game_uuid,
                agreed: true,
            })
            .expect("should succeed");

        assert!(result.changed);
        assert!(!result.reached_consensus_in_main_phase);
        let updated = game_repository.updated_first().expect("game should be updated");
        assert!(updated.next_update_at.is_some(), "next_update_at should be set to now");
    }
}
