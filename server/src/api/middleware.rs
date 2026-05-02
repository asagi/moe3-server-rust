// ============================================================================
// imports
// ============================================================================

use std::fmt;

use chrono::Utc;

use super::GameProgressionError;
use super::GameProgressionService;
use super::GameRepository;
use super::SqliteMessageRepository;
use super::UserRepository;

// ============================================================================
// definitions
// ============================================================================

///
/// グローバルプリハンドラの構造体
///
/// リクエストハンドラ呼び出し前に [`GlobalPreHandler::run`] を実行することで、
/// 卓の更新チェックや進行処理などの定期処理が自動的に実行される。
pub(crate) struct GlobalPreHandler<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    progression_service: GameProgressionService<U, G>,
    message_repository: SqliteMessageRepository,
}

/// グローバルプリハンドラの構造体の実装
impl<U, G> GlobalPreHandler<U, G>
where
    U: UserRepository,
    G: GameRepository,
{
    ///
    /// new 関数
    ///
    pub(crate) fn new(user_repository: U, game_repository: G, message_repository: SqliteMessageRepository) -> Self {
        Self {
            progression_service: GameProgressionService::new(user_repository, game_repository),
            message_repository,
        }
    }

    ///
    /// 全ハンドラの直前に呼び出す共通処理。
    ///
    /// - アクティブな全卓について卓主の無政府化を確認し、無政府化していれば is_accepting_draw を true に設定する。
    /// - Closed 以外の Game を取得し、next_update_at が過去なら最新フェイズを close する。
    pub(crate) fn run(&self) -> Result<(), PreHandlerError> {
        let newly_idle_owner_games = self
            .progression_service
            .mark_idle_owners_accepting_draw()
            .map_err(PreHandlerError::GameProgression)?;

        for (game_uuid, turn) in newly_idle_owner_games {
            if let Err(error) = self.message_repository.append_owner_absent_message(game_uuid, &turn) {
                eprintln!(
                    "failed to persist OwnerAbsent message for idle owner (game_uuid={}, turn={}): {}",
                    game_uuid, turn, error
                );
            }
        }

        let (aborted_game_uuids, started_seasons, solo_games, draw_games, closed_games) = self
            .progression_service
            .progress_games()
            .map_err(PreHandlerError::GameProgression)?;

        for game_uuid in aborted_game_uuids {
            if let Err(error) = self.message_repository.append_aborted_message(game_uuid) {
                eprintln!("failed to persist Aborted message (game_uuid={}): {}", game_uuid, error);
            }
        }

        for (game_uuid, turn, season) in started_seasons {
            if let Err(error) = self.message_repository.append_start_season_message(game_uuid, &turn, &season) {
                eprintln!(
                    "failed to persist StartSeason message (game_uuid={}, turn={}): {}",
                    game_uuid, turn, error
                );
                continue;
            }

            let owner_idle = self
                .progression_service
                .is_owner_idle_for_game(game_uuid, Utc::now().naive_utc())
                .map_err(PreHandlerError::GameProgression)?;

            if owner_idle && let Err(error) = self.message_repository.append_owner_absent_message(game_uuid, &turn) {
                eprintln!(
                    "failed to persist OwnerAbsent message after StartSeason (game_uuid={}, turn={}): {}",
                    game_uuid, turn, error
                );
            }
        }

        for (game_uuid, power) in solo_games {
            if let Err(error) = self.message_repository.append_solo_message(game_uuid, power) {
                eprintln!("failed to persist Solo message (game_uuid={}): {}", game_uuid, error);
            }
        }

        for game_uuid in draw_games {
            if let Err(error) = self.message_repository.append_draw_message(game_uuid) {
                eprintln!("failed to persist Draw message (game_uuid={}): {}", game_uuid, error);
            }
        }

        for game_uuid in closed_games {
            if let Err(error) = self.message_repository.append_closed_message(game_uuid) {
                eprintln!("failed to persist Closed message (game_uuid={}): {}", game_uuid, error);
            }
        }

        Ok(())
    }
}

///
/// グローバルプリハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum PreHandlerError {
    GameProgression(GameProgressionError),
}

/// グローバルプリハンドラのエラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for PreHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameProgression(error) => write!(f, "game progression failed: {}", error),
        }
    }
}
