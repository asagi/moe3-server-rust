// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;

use super::AuthRequestValidationError;
use super::AuthResetTokenRequestValidationError;

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
/// トークンリセットリクエストの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthResetTokenRequest {
    pub authorization: String,
}

/// トークンリセットリクエストの構造体の実装
impl AuthResetTokenRequest {
    pub(crate) fn validate(&self) -> Result<(), AuthResetTokenRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(AuthResetTokenRequestValidationError::MissingAuthorization);
        }

        let mut parts = auth.splitn(2, ' ');
        let scheme = parts.next().unwrap_or("");
        if !scheme.eq_ignore_ascii_case("Bearer") {
            return Err(AuthResetTokenRequestValidationError::InvalidAuthorizationScheme);
        }

        let token = parts.next().unwrap_or("").trim();
        if token.is_empty() {
            return Err(AuthResetTokenRequestValidationError::MissingAccessToken);
        }

        Ok(())
    }

    pub(crate) fn access_token(&self) -> &str {
        self.authorization.trim()
            .split_once(' ')
            .map(|(_, token)| token.trim())
            .unwrap_or("")
    }
}
