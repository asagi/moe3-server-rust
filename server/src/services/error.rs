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
/// Dicord クライアントエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiscordClientError {
    Unauthorized,
    Unavailable(String),
}

/// Discord クライアントエラーの列挙体の実装（Error トレイト）
impl Error for DiscordClientError {}

/// Discorad クライアントエラーの表示の実装（fmt::Display トレイト）
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
            Self::Repository(error) => write!(f, "repository error: {}", error),
            Self::Internal(message) => write!(f, "internal error: {}", message),
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
