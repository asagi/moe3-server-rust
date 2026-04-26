// ============================================================================
// imports
// ============================================================================

use std::fmt;

use super::GameProgressionError;
use super::GameProgressionService;
use super::GameRepository;
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
    pub(crate) fn new(user_repository: U, game_repository: G) -> Self {
        Self {
            progression_service: GameProgressionService::new(user_repository, game_repository),
        }
    }

    ///
    /// 全ハンドラの直前に呼び出す共通処理。
    ///
    /// - アクティブな全卓について卓主の無政府化を確認し、無政府化していれば is_accepting_draw を true に設定する。
    /// - Closed 以外の Game を取得し、next_update_at が過去なら最新フェイズを close する。
    pub(crate) fn run(&self) -> Result<(), PreHandlerError> {
        self.progression_service
            .mark_idle_owners_accepting_draw()
            .map_err(PreHandlerError::GameProgression)?;
        self.progression_service
            .progress_games()
            .map_err(PreHandlerError::GameProgression)
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
