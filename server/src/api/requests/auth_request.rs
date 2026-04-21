// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;

// ============================================================================
// definitions
// ============================================================================

///
/// ログインリクエストの構造体
///
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthLoginRequest {
    pub discord_access_token: String,
}

/// ログインリクエストの構造体の実装
impl AuthLoginRequest {
    pub(crate) fn validate(&self) -> Result<(), AuthRequestValidationError> {
        if self.discord_access_token.trim().is_empty() {
            return Err(AuthRequestValidationError::MissingDiscordAccessToken);
        }

        Ok(())
    }
}

///
/// ログインリクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthRequestValidationError {
    MissingDiscordAccessToken,
}
