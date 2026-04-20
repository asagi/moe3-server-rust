#![allow(dead_code)] // TODO: 後で削除する
// ============================================================================
// imports
// ============================================================================

// standard library
use std::fmt;

// structs
use super::GameAdvancementError;
use super::GameAdvancementService;

// traits
use super::GameRepository;

// ============================================================================
// definitions
// ============================================================================

/// 全リクエストの直前に実行するグローバルプリハンドラー。
/// ハンドラ呼び出し前に [`GlobalPreHandler::run`] を実行することで、
/// ゲームのフェイズ進行などの定期処理が自動的に適用される。
pub(crate) struct GlobalPreHandler<G>
where
    G: GameRepository,
{
    advancement_service: GameAdvancementService<G>,
}

impl<G> GlobalPreHandler<G>
where
    G: GameRepository,
{
    pub(crate) fn new(game_repository: G) -> Self {
        Self {
            advancement_service: GameAdvancementService::new(game_repository),
        }
    }

    /// 全ハンドラの直前に呼び出す共通処理。
    /// - Closed 以外の Game を取得し、next_update が過去なら最新フェイズを close する。
    pub(crate) fn run(&self) -> Result<(), PreHandlerError> {
        self.advancement_service
            .advance_games()
            .map_err(PreHandlerError::GameAdvancement)
    }
}

#[derive(Debug)]
pub(crate) enum PreHandlerError {
    GameAdvancement(GameAdvancementError),
}

impl fmt::Display for PreHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameAdvancement(error) => write!(f, "game advancement failed: {}", error),
        }
    }
}
