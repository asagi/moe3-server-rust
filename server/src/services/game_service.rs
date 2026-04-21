// ============================================================================
// imports
// ============================================================================

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
    #[allow(dead_code)]
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

        let next_update = command
            .regulation
            .start_date
            .and_hms_opt(command.regulation.first_period_hour as u32, 0, 0)
            .ok_or(CreateGameError::InvalidRequest(
                "first_period_hour is out of range".to_string(),
            ))?;

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
            next_update: Some(next_update),
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
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

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
        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows.borrow().get(access_token).cloned())
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
        Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
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
}
