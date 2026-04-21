// ============================================================================
// imports
// ============================================================================

use std::error::Error;
use std::fmt;

// ============================================================================
// definitions
// ============================================================================

///
/// リポジトリエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RepositoryError {
    NotFound,
    Conflict,
    Unavailable(String),
}

/// リポジトリエラーの列挙体の実装（Error トレイト）
impl Error for RepositoryError {}

/// リポジトリエラーの列挙体の実装（fmt::Display トレイト）
impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "record not found"),
            Self::Conflict => write!(f, "record conflict"),
            Self::Unavailable(message) => write!(f, "repository unavailable: {}", message),
        }
    }
}
