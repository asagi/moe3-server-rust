#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use serde::Deserialize;

// ============================================================================
// definitions
// ============================================================================

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct AuthLoginRequest {
    pub discord_access_token: String,
}

impl AuthLoginRequest {
    pub(crate) fn validate(&self) -> Result<(), AuthRequestValidationError> {
        if self.discord_access_token.trim().is_empty() {
            return Err(AuthRequestValidationError::MissingDiscordAccessToken);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthRequestValidationError {
    MissingDiscordAccessToken,
}
