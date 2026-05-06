// ============================================================================
// imports
// ============================================================================

use std::error::Error;
use std::fmt;

use super::RepositoryError;

// ============================================================================
// definitions
// ============================================================================

///
/// Discord クライアントエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiscordClientError {
    Unauthorized,
    Unavailable(String),
}

/// Discord クライアントエラーの列挙体の実装（Error トレイト）
impl Error for DiscordClientError {}

/// Discord クライアントエラーの表示の実装（fmt::Display トレイト）
impl fmt::Display for DiscordClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "discord token is unauthorized"),
            Self::Unavailable(message) => write!(f, "discord api unavailable: {}", message),
        }
    }
}

///
/// 認証エラーの列挙体
///
#[derive(Debug, Clone)]
pub(crate) enum AuthError {
    InvalidRequest(String),
    DiscordClient(DiscordClientError),
    Repository(RepositoryError),
}

/// 認証エラーの列挙体の実装（Error トレイト）
impl Error for AuthError {}

/// 認証エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::DiscordClient(error) => write!(f, "discord client error: {}", error),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 新卓作成エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateGameError {
    InvalidRequest(String),
    Unauthorized,
    Forbidden(String),
    Repository(RepositoryError),
    Internal(String),
}

/// 新卓作成エラーの列挙体の実装（Error トレイト）
impl Error for CreateGameError {}

/// 新卓作成エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for CreateGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::Repository(error) => write!(f, "repository error: {}", error),
            Self::Internal(message) => write!(f, "internal error: {}", message),
        }
    }
}

///
/// 卓参加エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JoinGameError {
    InvalidRequest(String),
    Unauthorized,
    NotFound,
    Forbidden(String),
    Repository(RepositoryError),
}

/// 卓参加エラーの列挙体の実装（Error トレイト）
impl Error for JoinGameError {}

/// 卓参加エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for JoinGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 和平終了フラグ設定エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetDrawProposalError {
    Unauthorized,
    NotFound,
    Forbidden(String),
    Repository(RepositoryError),
}

/// 和平終了フラグ設定エラーの列挙体の実装（Error トレイト）
impl Error for SetDrawProposalError {}

/// 和平終了フラグ設定エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for SetDrawProposalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// ユニット配置制御エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetUnitError {
    Unauthorized,
    NotFound,
    Forbidden(String),
    InvalidRequest(String),
    PhaseConflict,
    Repository(RepositoryError),
}

/// ユニット配置制御エラーの列挙体の実装（Error トレイト）
impl Error for SetUnitError {}

/// ユニット配置制御エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for SetUnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::PhaseConflict => write!(f, "phase has changed since this request was issued"),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 卓進行エラーの列挙体
///
#[derive(Debug)]
pub(crate) enum GameProgressionError {
    Repository(RepositoryError),
}

/// 卓進行エラーの列挙体の実装（Error トレイト）
impl Error for GameProgressionError {}

/// 卓進行エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for GameProgressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 占領情報編集エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetTerritoryError {
    Unauthorized,
    NotFound,
    WaterProvince,
    Forbidden(String),
    InvalidRequest(String),
    PhaseConflict,
    Repository(RepositoryError),
}

/// 占領情報編集エラーの列挙体の実装（Error トレイト）
impl Error for SetTerritoryError {}

/// 占領情報編集エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for SetTerritoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::WaterProvince => write!(f, "cannot set territory ownership for a water province"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::PhaseConflict => write!(f, "phase has changed since this request was issued"),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 進行モード変更エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetProgressModeError {
    Unauthorized,
    NotFound,
    Forbidden(String),
    PhaseConflict,
    Repository(RepositoryError),
}

/// 進行モード変更エラーの列挙体の実装（Error トレイト）
impl Error for SetProgressModeError {}

/// 進行モード変更エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for SetProgressModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::PhaseConflict => write!(f, "phase has changed since this request was issued"),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 即時進行合意設定エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetProgressConsensusError {
    Unauthorized,
    NotFound,
    Forbidden(String),
    Repository(RepositoryError),
}

/// 即時進行合意設定エラーの列挙体の実装（Error トレイト）
impl Error for SetProgressConsensusError {}

/// 即時進行合意設定エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for SetProgressConsensusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}

///
/// 次回更新時刻変更エラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetNextUpdateAtError {
    Unauthorized,
    NotFound,
    Forbidden(String),
    InvalidRequest(String),
    PhaseConflict,
    Repository(RepositoryError),
}

/// 次回更新時刻変更エラーの列挙体の実装（Error トレイト）
impl Error for SetNextUpdateAtError {}

/// 次回更新時刻変更エラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for SetNextUpdateAtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "game not found"),
            Self::Forbidden(message) => write!(f, "forbidden: {}", message),
            Self::InvalidRequest(message) => write!(f, "invalid request: {}", message),
            Self::PhaseConflict => write!(f, "phase has changed since this request was issued"),
            Self::Repository(error) => write!(f, "repository error: {}", error),
        }
    }
}
