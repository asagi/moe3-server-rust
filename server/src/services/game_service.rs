// ============================================================================
// imports
// ============================================================================

use chrono::FixedOffset;
use chrono::TimeZone;
use uuid::Uuid;

use super::CreateGameError;
use super::Game;
use super::GameProgressionService;
use super::GameRepository;
use super::GameStatus;
use super::JoinGameError;
use super::NewGame;
use super::Phase;
use super::Player;
use super::Power;
use super::Province;
use super::Regulation;
use super::RepositoryError;
use super::SetDrawProposalError;
use super::SetUnitError;
use super::Unit;
use super::UserRepository;
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
}

///
/// ユニット配置制御処理結果の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SetUnitResult {
    pub game: Game,
    pub old_unit: Option<Unit>,
    pub new_unit: Option<Unit>,
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

        let location = Province::from_code(&command.location)
            .ok_or_else(|| SetUnitError::InvalidRequest(format!("invalid location: {}", command.location)))?;

        // 新ユニットの構築（指定がある場合）
        let new_unit = if let Some(ref spec) = command.unit {
            let power = Power::from_symbol(&spec.power_symbol)
                .ok_or_else(|| SetUnitError::InvalidRequest(format!("invalid power: {}", spec.power_symbol)))?;

            let unit = match spec.kind_str.to_lowercase().as_str() {
                "a" | "army" => {
                    // 陸軍は海岸バリアントコードを許可しない
                    if location.code_with_coast() != location.code() {
                        return Err(SetUnitError::InvalidRequest(
                            "army cannot be placed on a coast variant location".to_string(),
                        ));
                    }
                    Unit::new_army(power, location)
                }
                "f" | "fleet" => {
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
        })
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
                    && game.status != GameStatus::Finished
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
    fn create_game_allows_when_participating_game_is_finished() {
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

        let finished_game = Game {
            uuid: Uuid::now_v7(),
            game_number: Some(1),
            keyword: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Finished,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let game_repository = InMemoryGameRepository::new(vec![finished_game]);
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
}
