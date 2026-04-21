// ============================================================================
// imports
// ============================================================================

use chrono::Timelike;
use chrono::Utc;

use super::GameProgressionError;
use super::GameRepository;
use super::GameStatus;
use super::Phase;
use super::PhaseContext;
use crate::domain::PhaseKind;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓進行サービスの構造体
///
pub(crate) struct GameProgressionService<G>
where
    G: GameRepository,
{
    game_repository: G,
}

/// 卓進行サービスの構造体の実装
impl<G> GameProgressionService<G>
where
    G: GameRepository,
{
    pub(crate) fn new(game_repository: G) -> Self {
        Self { game_repository }
    }

    /// Closed 以外の全 Game に対してフェイズ進行を試みる。
    /// next_update が現在時刻より過去の場合のみ進行処理を実行する。
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

        let Some(previous_next_update) = game.next_update else {
            return Ok(());
        };

        if game.status == GameStatus::Closed || previous_next_update > now {
            return Ok(());
        }

        let latest_phase = game.phases.pop().expect("game should have at least one phase");

        let mut context = PhaseContext::new();
        latest_phase.close(&mut context);

        let is_finished = context.is_finished();
        let new_next_update = if is_finished {
            None
        } else {
            let next_phase = context.phases().back().expect("in-progress game should have next phase");
            Some(Self::calculate_next_update(
                previous_next_update,
                next_phase,
                game.regulation.duration_type,
            ))
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
        game.next_update = new_next_update;

        self.game_repository.update(&game).map_err(GameProgressionError::Repository)?;

        Ok(())
    }

    fn calculate_next_update(
        previous_next_update: chrono::NaiveDateTime,
        next_phase: &Phase,
        duration_type: crate::domain::DurationType,
    ) -> chrono::NaiveDateTime {
        let duration_minutes = if Self::is_sub_phase(next_phase) {
            duration_type.sub_phase_minutes()
        } else {
            duration_type.main_phase_minutes()
        };

        let raw_next_update = previous_next_update + chrono::Duration::minutes(i64::from(duration_minutes));
        Self::ceil_to_5_minutes(raw_next_update)
    }

    fn is_sub_phase(phase: &Phase) -> bool {
        matches!(
            phase.kind,
            PhaseKind::SpringRetreat(_) | PhaseKind::FallRetreat(_) | PhaseKind::Adjustment(_)
        )
    }

    fn ceil_to_5_minutes(dt: chrono::NaiveDateTime) -> chrono::NaiveDateTime {
        let truncated =
            dt - chrono::Duration::seconds(i64::from(dt.second())) - chrono::Duration::nanoseconds(i64::from(dt.nanosecond()));

        // 秒以下がある時刻は分境界を過ぎているため、1分進めてから5分単位へ切り上げる。
        let minute_aligned = if dt.second() == 0 && dt.nanosecond() == 0 {
            truncated
        } else {
            truncated + chrono::Duration::minutes(1)
        };

        let remainder = minute_aligned.minute() % 5;
        if remainder == 0 {
            minute_aligned
        } else {
            minute_aligned + chrono::Duration::minutes(i64::from(5 - remainder))
        }
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
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
    use crate::repositories::RepositoryError;

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
                .filter(|game| game.next_update.is_some_and(|next_update| next_update <= now))
                .map(|game| game.uuid)
                .collect())
        }

        fn update(&self, game: &Game) -> Result<(), RepositoryError> {
            self.updated_games.borrow_mut().push(game.clone());
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

    fn sample_game(next_update: Option<chrono::NaiveDateTime>) -> Game {
        Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            regulation: sample_regulation(),
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
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
            next_update,
        }
    }

    #[test]
    fn progress_games_skips_when_next_update_is_none() {
        let repository = InMemoryGameRepository::new(vec![sample_game(None)]);
        let service = GameProgressionService::new(repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_skips_when_next_update_is_in_future() {
        let future = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(5);
        let repository = InMemoryGameRepository::new(vec![sample_game(Some(future))]);
        let service = GameProgressionService::new(repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn progress_games_updates_when_next_update_is_in_past() {
        let past_date = (chrono::Utc::now().naive_utc() - chrono::Duration::days(1)).date();
        let past = past_date.and_hms_opt(14, 32, 0).expect("valid datetime");
        let repository = InMemoryGameRepository::new(vec![sample_game(Some(past))]);
        let service = GameProgressionService::new(repository.clone());

        service.progress_games().expect("progress should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(updated.status, GameStatus::InProgress);
        assert_eq!(
            updated.next_update,
            Some(past_date.and_hms_opt(15, 5, 0).expect("valid datetime"))
        );
    }

    #[test]
    fn ceil_to_5_minutes_rounds_up_when_seconds_exist() {
        let dt = chrono::NaiveDate::from_ymd_opt(2026, 4, 22)
            .expect("valid date")
            .and_hms_opt(14, 30, 1)
            .expect("valid datetime");

        let rounded = GameProgressionService::<InMemoryGameRepository>::ceil_to_5_minutes(dt);

        assert_eq!(
            rounded,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 22)
                .expect("valid date")
                .and_hms_opt(14, 35, 0)
                .expect("valid datetime")
        );
    }
}
