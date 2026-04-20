// ============================================================================
// imports
// ============================================================================

// standard library
use std::error::Error;
use std::fmt;

// external crates
use chrono::Utc;

// structs
use super::Game;
use super::PhaseContext;

// enums
use super::GameStatus;
use super::RepositoryError;

// traits
use super::GameRepository;

// ============================================================================
// definitions
// ============================================================================

#[allow(dead_code)]
pub(crate) struct GameAdvancementService<G>
where
    G: GameRepository,
{
    game_repository: G,
}

#[allow(dead_code)]
impl<G> GameAdvancementService<G>
where
    G: GameRepository,
{
    pub(crate) fn new(game_repository: G) -> Self {
        Self { game_repository }
    }

    /// Closed 以外の全 Game に対してフェイズ進行を試みる。
    /// next_update が現在時刻より過去の場合のみ進行処理を実行する。
    pub(crate) fn advance_games(&self) -> Result<(), GameAdvancementError> {
        let now = Utc::now().naive_utc();

        let games = self
            .game_repository
            .find_all_active()
            .map_err(GameAdvancementError::Repository)?;

        for mut game in games {
            let Some(next_update) = game.next_update else {
                continue;
            };

            if next_update > now {
                continue;
            }

            game.next_update = None; // FIXME: 暫定

            self.advance_game(game)?;
        }

        Ok(())
    }

    fn advance_game(&self, mut game: Game) -> Result<(), GameAdvancementError> {
        let latest_phase = game.phases.pop().expect("game should have at least one phase");

        let mut context = PhaseContext::new();
        latest_phase.close(&mut context);

        // PhaseContext の結果を Game に反映
        for phase in context.phases().iter() {
            game.phases.push(phase.clone());
        }
        game.is_draw = context.is_draw();
        game.is_solo = context.is_solo();
        game.status = if context.is_finished() {
            GameStatus::Finished
        } else {
            GameStatus::InProgress
        };

        // TODO: 次のフェイズの更新予定時刻を設定する

        self.game_repository.update(&game).map_err(GameAdvancementError::Repository)?;

        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum GameAdvancementError {
    Repository(RepositoryError),
}

impl fmt::Display for GameAdvancementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

impl Error for GameAdvancementError {}

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
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::domain::Power;
    use crate::domain::ProgressMode;
    use crate::domain::Regulation;
    use crate::repositories::NewGame;

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
    fn advance_games_skips_when_next_update_is_none() {
        let repository = InMemoryGameRepository::new(vec![sample_game(None)]);
        let service = GameAdvancementService::new(repository.clone());

        service.advance_games().expect("advance should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn advance_games_skips_when_next_update_is_in_future() {
        let future = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(5);
        let repository = InMemoryGameRepository::new(vec![sample_game(Some(future))]);
        let service = GameAdvancementService::new(repository.clone());

        service.advance_games().expect("advance should succeed");

        assert_eq!(repository.updated_len(), 0);
    }

    #[test]
    fn advance_games_updates_when_next_update_is_in_past() {
        let past = chrono::Utc::now().naive_utc() - chrono::Duration::minutes(5);
        let repository = InMemoryGameRepository::new(vec![sample_game(Some(past))]);
        let service = GameAdvancementService::new(repository.clone());

        service.advance_games().expect("advance should succeed");

        assert_eq!(repository.updated_len(), 1);
        let updated = repository.updated_first().expect("updated game should exist");
        assert_eq!(updated.status, GameStatus::InProgress);
        assert!(updated.next_update.is_none());
    }
}
