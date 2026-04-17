use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct LoginRequest {
    pub discord_access_token: String,
}

impl LoginRequest {
    pub(crate) fn validate(&self) -> Result<(), RequestValidationError> {
        if self.discord_access_token.trim().is_empty() {
            return Err(RequestValidationError::MissingDiscordAccessToken);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RequestValidationError {
    MissingDiscordAccessToken,
}
