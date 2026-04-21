// ============================================================================
// imports
// ============================================================================

use std::fmt;

use super::GameProgressionError;
use super::GameProgressionService;
use super::GameRepository;

// ============================================================================
// definitions
// ============================================================================

///
/// グローバルプリハンドラの構造体
///
/// リクエストハンドラ呼び出し前に [`GlobalPreHandler::run`] を実行することで、
/// 卓の更新チェックや進行処理などの定期処理が自動的に実行される。
#[allow(dead_code)]
pub(crate) struct GlobalPreHandler<G>
where
    G: GameRepository,
{
    progression_service: GameProgressionService<G>,
}

/// グローバルプリハンドラの構造体の実装
#[allow(dead_code)]
impl<G> GlobalPreHandler<G>
where
    G: GameRepository,
{
    pub(crate) fn new(game_repository: G) -> Self {
        Self {
            progression_service: GameProgressionService::new(game_repository),
        }
    }

    /// 全ハンドラの直前に呼び出す共通処理。
    /// - Closed 以外の Game を取得し、next_update が過去なら最新フェイズを close する。
    pub(crate) fn run(&self) -> Result<(), PreHandlerError> {
        self.progression_service
            .progress_games()
            .map_err(PreHandlerError::GameProgression)
    }
}

///
/// グローバルプリハンドラのエラーの列挙体
///
#[derive(Debug)]
#[allow(dead_code)]
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
