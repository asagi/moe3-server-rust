// ============================================================================
// imports
// ============================================================================

use chrono::FixedOffset;
use chrono::TimeZone;
use uuid::Uuid;

use super::CreateGameError;
use super::Game;
use super::GameRepository;
use super::GameStatus;
use super::NewGame;
use super::Phase;
use super::Player;
use super::Power;
use super::Regulation;
use super::UserRepository;

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
    const MIN_START_LEAD_MINUTES: i64 = 30;

    pub(crate) fn new(user_repository: U, game_repository: G) -> Self {
        Self {
            user_repository,
            game_repository,
        }
    }

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
            regulation: command.regulation,
            players: vec![owner],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_canceled: false,
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

    fn validate_start_datetime(start_datetime: chrono::NaiveDateTime, now: chrono::NaiveDateTime) -> Result<(), CreateGameError> {
        let min_allowed = now + chrono::Duration::minutes(Self::MIN_START_LEAD_MINUTES);
        if start_datetime <= min_allowed {
            return Err(CreateGameError::InvalidRequest(
                "start_datetime must be more than 30 minutes in the future".to_string(),
            ));
        }

        Ok(())
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
    }

    impl InMemoryGameRepository {
        fn new() -> Self {
            Self {
                created: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl GameRepository for InMemoryGameRepository {
        fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError> {
            self.created.borrow_mut().push(new_game.game.clone());
            Ok(new_game.game)
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(Vec::new())
        }

        fn find_by_uuid(&self, _game_uuid: Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(None)
        }

        fn find_progress_candidates(&self, _now: chrono::NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(Vec::new())
        }

        fn update(&self, _game: &Game) -> Result<(), RepositoryError> {
            Ok(())
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
        let game_repository = InMemoryGameRepository::new();

        let service = GameService::new(user_repository, game_repository);

        let result = service
            .create_game(CreateGameCommand {
                access_token: "token-1".to_string(),
                regulation: sample_regulation(),
                requested_power: Some(Power::France),
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
        let game_repository = InMemoryGameRepository::new();
        let service = GameService::new(user_repository, game_repository);

        let result = service
            .create_game(CreateGameCommand {
                access_token: "token-1".to_string(),
                regulation: sample_regulation(),
                requested_power: None,
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
        let game_repository = InMemoryGameRepository::new();
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
            })
            .expect_err("create game should fail");

        assert!(matches!(error, CreateGameError::InvalidRequest(_)));
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
}
