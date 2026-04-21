// ============================================================================
// imports
// ============================================================================

use super::ApiErrorResponse;
use super::AuthError;
use super::AuthRequestValidationError;
use super::CreateGameError;
use super::CreateGameRequestValidationError;
use super::DiscordClientError;

// ============================================================================
// definitions
// ============================================================================

///
/// ログインリクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum AuthHandlerError {
    InvalidRequest(AuthRequestValidationError),
    Service(AuthError),
}

/// ログインリクエストハンドラのエラーの列挙体の実装
impl AuthHandlerError {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(AuthError::InvalidRequest(_)) => "invalid_request",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unauthorized)) => "unauthorized",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unavailable(_))) => "discord_unavailable",
            Self::Service(AuthError::Repository(_)) => "repository_error",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(AuthRequestValidationError::MissingDiscordAccessToken) => {
                "discord_access_token is required".to_string()
            }
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 卓作成リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum CreateGameHandlerError {
    InvalidRequest(CreateGameRequestValidationError),
    Service(CreateGameError),
}

/// 卓作成リクエストハンドラのエラーの列挙体の実装
impl CreateGameHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(CreateGameError::InvalidRequest(_)) => "invalid_request",
            Self::Service(CreateGameError::Unauthorized) => "unauthorized",
            Self::Service(CreateGameError::Repository(_)) => "repository_error",
            Self::Service(CreateGameError::Internal(_)) => "internal_error",
        }
    }

    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(CreateGameRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::MissingAccessToken) => "access token is required".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidFaceType) => "face_type is invalid".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidProgressMode) => "progress_mode is invalid".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidDurationType) => "duration_type is invalid".to_string(),
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidFirstPeriodHour) => {
                "first_period_hour must be between 0 and 23".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidStartDate) => {
                "start_date is invalid (expected YYYY-MM-DD)".to_string()
            }
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidRequestedPower) => {
                "requested_power is invalid".to_string()
            }
            Self::Service(error) => error.to_string(),
        }
    }
}
