// ============================================================================
// imports
// ============================================================================

use super::GameProgressionError;
use super::GameRepository;
use super::GameStatus;
use super::PhaseContext;
use super::UserRepository;
use chrono::Utc;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓進行サービスの構造体
///
pub(crate) struct GameProgressionService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    user_repository: U,
    game_repository: G,
}

/// 卓進行サービスの構造体の実装
impl<U, G> GameProgressionService<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    pub(crate) fn new(user_repository: U, game_repository: G) -> Self {
        Self {
            user_repository,
            game_repository,
        }
    }

    /// Closed 以外の全 Game に対してフェイズ進行を試みる。
    /// next_update_at が現在時刻より過去の場合のみ進行処理を実行する。
    pub(crate) fn progress_games(&self) -> Result<(), GameProgressionError> {
        let now = Utc::now().naive_utc();

        let candidate_game_uuids = self
            .game_repository
            .find_progress_candidates(now)
            .map_err(GameProgressionError::Repository)?;

        for game_uuid in candidate_game_uuids {
            self.progress_game(game_uuid, now)?;
        }

        Ok(())
    }

    fn progress_game(&self, game_uuid: uuid::Uuid, now: chrono::NaiveDateTime) -> Result<(), GameProgressionError> {
        let Some(mut game) = self
            .game_repository
            .find_by_uuid(game_uuid)
            .map_err(GameProgressionError::Repository)?
        else {
            return Ok(());
        };

        let Some(previous_next_update) = game.next_update_at else {
            return Ok(());
        };

        if game.status == GameStatus::Closed || previous_next_update > now {
            return Ok(());
        }

        let latest_phase = game.phases.pop().expect("game should have at least one phase");

        let mut context = PhaseContext::new();
        if Self::is_owner_accepting_draw(&game) {
            context.set_draw();
        }
        self.remove_idle_powers(&game, &mut context, now)?;
        latest_phase.close(&mut context);

        let is_finished = context.is_finished();
        let new_next_update = if is_finished {
            None
        } else {
            let next_phase = context.phases().back().expect("in-progress game should have next phase");
            Some(game.calculate_next_update(previous_next_update, next_phase))
        };

        // PhaseContext の結果を Game に反映
        for phase in context.phases().iter() {
            game.phases.push(phase.clone());
        }
        game.is_draw = context.is_draw();
        game.is_solo = context.is_solo();
        game.status = if is_finished {
            GameStatus::Finished
        } else {
            GameStatus::InProgress
        };
        game.next_update_at = new_next_update;

        self.game_repository.update(&game).map_err(GameProgressionError::Repository)?;

        Ok(())
    }

    fn is_owner_accepting_draw(game: &super::Game) -> bool {
        game.players
            .iter()
            .find(|player| player.is_owner)
            .is_some_and(|player| player.is_accepting_draw)
    }

    fn remove_idle_powers(
        &self,
        game: &super::Game,
        context: &mut PhaseContext,
        now: chrono::NaiveDateTime,
    ) -> Result<(), GameProgressionError> {
        let idle_limit = chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes()));
        let threshold = now - idle_limit;

        for player in game.players.iter().filter(|player| player.power.is_some()) {
            let power = player.power.expect("filtered as Some");

            let user = self
                .user_repository
                .find_by_uuid(player.user_uuid)
                .map_err(GameProgressionError::Repository)?;

            let is_idle_or_missing = match user {
                Some(user) => user.last_access_at.naive_utc() <= threshold,
                None => true,
            };

            if is_idle_or_missing {
                context.remove_power(&power);
            }
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

    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::Game;
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::domain::Power;
    use crate::domain::ProgressMode;
    use crate::domain::Regulation;
    use crate::repositories::NewGame;
    use crate::repositories::NewUser;
    use crate::repositories::RepositoryError;
    use crate::repositories::UserId;
    use crate::repositories::UserProfileUpdate;
    use crate::repositories::UserRecord;

    #[derive(Debug, Clone)]
    struct InMemoryGameRepository {
        active_games: Rc<RefCell<Vec<Game>>>,
        updated_games: Rc<RefCell<Vec<Game>>>,
    }

    impl InMemoryGameRepository {
        fn new(active_games: Vec<Game>) -> Self {
            Self {
                active_games: Rc::new(RefCell::new(active_games)),
                updated_games: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn updated_len(&self) -> usize {
            self.updated_games.borrow().len()
        }

        fn updated_first(&self) -> Option<Game> {
            self.updated_games.borrow().first().cloned()
        }
    }

    impl GameRepository for InMemoryGameRepository {
        fn insert(&self, _new_game: NewGame) -> Result<Game, RepositoryError> {
            Err(RepositoryError::Unavailable("not used".to_string()))
        }

        fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
            Ok(self.active_games.borrow().clone())
        }

        fn find_by_uuid(&self, game_uuid: uuid::Uuid) -> Result<Option<Game>, RepositoryError> {
            Ok(self.active_games.borrow().iter().find(|game| game.uuid == game_uuid).cloned())
        }

        fn find_progress_candidates(&self, now: chrono::NaiveDateTime) -> Result<Vec<uuid::Uuid>, RepositoryError> {
            Ok(self
                .active_games
                .borrow()
                .iter()
                .filter(|game| game.status != GameStatus::Closed)
                .filter(|game| game.next_update_at.is_some_and(|next_update| next_update <= now))
                .map(|game| game.uuid)
                .collect())
        }

        fn update(&self, game: &Game) -> Result<(), RepositoryError> {
            self.updated_games.borrow_mut().push(game.clone());
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    struct InMemoryUserRepository {
        rows_by_uuid: Rc<RefCell<HashMap<uuid::Uuid, UserRecord>>>,
    }

    impl InMemoryUserRepository {
        fn new(rows: Vec<UserRecord>) -> Self {
            let map = rows.into_iter().map(|row| (row.uuid, row)).collect();
            Self {
                rows_by_uuid: Rc::new(RefCell::new(map)),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn find_by_uuid(&self, user_uuid: uuid::Uuid) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self.rows_by_uuid.borrow().get(&user_uuid).cloned())
        }

        fn find_by_discord_user_id(&self, _discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
            Ok(self
                .rows_by_uuid
                .borrow()
                .values()
                .find(|row| row.access_token == access_token)
                .cloned())
        }

        fn update_last_access_at_by_access_token(
            &self,
            access_token: &str,
            last_access_at: chrono::DateTime<chrono::Utc>,
        ) -> Result<bool, RepositoryError> {
            let mut rows = self.rows_by_uuid.borrow_mut();
            let Some(row) = rows.values_mut().find(|row| row.access_token == access_token) else {
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

    fn sample_game(next_update: Option<chrono::NaiveDateTime>) -> Game {
        sample_game_with_regulation(next_update, sample_regulation())
    }

    fn sample_game_with_regulation(next_update: Option<chrono::NaiveDateTime>, regulation: Regulation) -> Game {
        let owner_uuid = uuid::Uuid::now_v7();
        Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            regulation,
            players: vec![Player {
                user_uuid: owner_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_canceled: false,
            is_draw: false,
            is_solo: false,
            next_update_at: next_update,
        }
    }

    fn user_for(game: &Game, power: Power, last_access_at: chrono::DateTime<chrono::Utc>) -> UserRecord {
        let user_uuid = game
            .players
            .iter()
            .find(|player| player.power == Some(power))
            .map(|player| player.user_uuid)
            .expect("player for power should exist");

        UserRecord {
            id: 1,
            uuid: user_uuid,
            discord_user_id: format!("discord-{}", power.symbol()),
            username: format!("{}-user", power.symbol()),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
            access_token: format!("token-{}", power.symbol()),
            last_access_at,
        }
    }

    #[test]
    fn progress_games_skips_when_next_update_is_none() {
        let game = sample_game(None);
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_skips_when_next_update_is_in_future() {
        let future = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(5);
        let game = sample_game(Some(future));
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_updates_when_next_update_is_in_past() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");
        let game = sample_game(Some(past));
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(updated.status, GameStatus::InProgress);
        assert_eq!(
            updated.next_update_at,
            Some(past_date.and_hms_opt(15, 5, 0).expect("valid datetime"))
        );
    }

    #[test]
    fn progress_games_sets_next_main_phase_to_next_day_first_period_hour_for_scheduled_normal() {
        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Normal,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            21,
        )
        .expect("valid regulation");
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");
        let game = sample_game_with_regulation(Some(past), regulation);
        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(
            updated.next_update_at,
            Some(
                past_date
                    .succ_opt()
                    .expect("next day should exist")
                    .and_hms_opt(12, 0, 0)
                    .expect("valid datetime")
            )
        );
    }

    #[test]
    fn progress_games_skips_retreat_wait_for_idle_power() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");

        let mut game = sample_game(Some(past));
        let france_user_uuid = uuid::Uuid::now_v7();
        let germany_user_uuid = uuid::Uuid::now_v7();
        game.players = vec![
            Player {
                user_uuid: france_user_uuid,
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            },
            Player {
                user_uuid: germany_user_uuid,
                power: Some(Power::Germany),
                is_accepting_draw: false,
                is_owner: false,
                requested_power: Some(Power::Germany),
            },
        ];
        let mut spring_main = Phase::new_spring_main(1900, 0);
        spring_main.units = vec![
            crate::domain::Unit::new_army(
                Power::Germany,
                crate::domain::Province::from_code("ber").expect("valid province"),
            )
            .set_dislodged_from(Some(crate::domain::Province::from_code("kie").expect("valid province"))),
        ];
        spring_main.territories = vec![crate::domain::Territory::new(Power::Germany, "ber")];
        game.phases = vec![spring_main];
        game.status = GameStatus::InProgress;

        let users = InMemoryUserRepository::new(vec![
            UserRecord {
                id: 1,
                uuid: france_user_uuid,
                discord_user_id: "discord-f".to_string(),
                username: "f-user".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-f".to_string(),
                last_access_at: chrono::Utc::now(),
            },
            UserRecord {
                id: 2,
                uuid: germany_user_uuid,
                discord_user_id: "discord-g".to_string(),
                username: "g-user".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-g".to_string(),
                last_access_at: chrono::Utc::now()
                    - chrono::Duration::minutes(i64::from(game.regulation.duration_type.idle_limit_minutes() + 1)),
            },
        ]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        let updated = repository.updated_first().expect("updated game should exist");
        assert!(matches!(
            updated.phases.last().expect("phase should exist").kind,
            crate::domain::PhaseKind::FallMain(_)
        ));
    }

    #[test]
    fn progress_games_finishes_with_draw_when_owner_accepts_draw() {
        let past = chrono::Utc::now().naive_utc() - chrono::Duration::minutes(1);

        let mut game = sample_game(Some(past));
        game.phases = vec![Phase::new_spring_main(1900, 0)];
        game.status = GameStatus::InProgress;
        game.players[0].is_accepting_draw = true;

        let users = InMemoryUserRepository::new(vec![user_for(&game, Power::France, chrono::Utc::now())]);
        let repository = InMemoryGameRepository::new(vec![game]);
        let service = GameProgressionService::new(users, repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert!(updated.is_draw);
        assert!(!updated.is_solo);
        assert_eq!(updated.status, GameStatus::Finished);
        assert!(updated.next_update_at.is_none());
        assert!(matches!(
            updated.phases.last().expect("phase should exist").kind,
            crate::domain::PhaseKind::Debrief(_)
        ));
    }
}
