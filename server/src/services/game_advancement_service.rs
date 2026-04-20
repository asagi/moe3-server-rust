#![allow(dead_code)] // TODO: 後で削除する
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

pub(crate) struct GameAdvancementService<G>
where
    G: GameRepository,
{
    game_repository: G,
}

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
