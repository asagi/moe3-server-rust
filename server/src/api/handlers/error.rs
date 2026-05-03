// ============================================================================
// imports
// ============================================================================

use super::ApiErrorResponse;
use super::AuthError;
use super::AuthRequestValidationError;
use super::CreateGameError;
use super::CreateGameRequestValidationError;
use super::DiscordClientError;
use super::JoinGameError;
use super::JoinGameRequestValidationError;
use super::SetDrawProposalError;
use super::SetDrawProposalRequestValidationError;

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
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(AuthError::InvalidRequest(_)) => "invalid_request",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unauthorized)) => "unauthorized",
            Self::Service(AuthError::DiscordClient(DiscordClientError::Unavailable(_))) => "discord_unavailable",
            Self::Service(AuthError::Repository(_)) => "repository_error",
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
    ///
    /// エラーコードを取得する
    ///
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(CreateGameError::InvalidRequest(_)) => "invalid_request",
            Self::Service(CreateGameError::Unauthorized) => "unauthorized",
            Self::Service(CreateGameError::Forbidden(_)) => "forbidden",
            Self::Service(CreateGameError::Repository(_)) => "repository_error",
            Self::Service(CreateGameError::Internal(_)) => "internal_error",
        }
    }

    ///
    /// 卓作成リクエストハンドラのエラーを API エラーレスポンスに変換する
    ///
    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    /// 卓作成リクエストハンドラのエラーに対応するエラーメッセージを生成する
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
            Self::InvalidRequest(CreateGameRequestValidationError::InvalidKeyword) => {
                "keyword must contain only alphanumeric characters".to_string()
            }
            Self::Service(CreateGameError::Forbidden(message)) => message.clone(),
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 卓参加リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum JoinGameHandlerError {
    InvalidRequest(JoinGameRequestValidationError),
    Service(JoinGameError),
}

/// 卓参加リクエストハンドラのエラーの列挙体の実装
impl JoinGameHandlerError {
    ///
    /// 卓参加リクエストハンドラのエラーに対応するエラーコードを生成する
    ///
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(JoinGameError::InvalidRequest(_)) => "invalid_request",
            Self::Service(JoinGameError::Unauthorized) => "unauthorized",
            Self::Service(JoinGameError::NotFound) => "not_found",
            Self::Service(JoinGameError::Forbidden(_)) => "forbidden",
            Self::Service(JoinGameError::Repository(_)) => "repository_error",
        }
    }

    ///
    /// 卓参加リクエストハンドラのエラーを API エラーレスポンスに変換する
    ///
    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    /// 卓参加リクエストハンドラのエラーに対応するエラーメッセージを生成する
    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(JoinGameRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(JoinGameRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(JoinGameRequestValidationError::MissingAccessToken) => "access token is required".to_string(),
            Self::InvalidRequest(JoinGameRequestValidationError::InvalidGameUuid) => "game_uuid is invalid".to_string(),
            Self::InvalidRequest(JoinGameRequestValidationError::InvalidRequestedPower) => {
                "requested_power is invalid".to_string()
            }
            Self::InvalidRequest(JoinGameRequestValidationError::InvalidKeyword) => {
                "keyword must contain only alphanumeric characters".to_string()
            }
            Self::Service(JoinGameError::Forbidden(message)) => message.clone(),
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 和平終了フラグ設定リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum SetDrawProposalHandlerError {
    InvalidRequest(SetDrawProposalRequestValidationError),
    Service(SetDrawProposalError),
}

/// 和平終了フラグ設定リクエストハンドラのエラーの列挙体の実装
impl SetDrawProposalHandlerError {
    ///
    /// エラーコードを取得する
    ///
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(SetDrawProposalError::Unauthorized) => "unauthorized",
            Self::Service(SetDrawProposalError::NotFound) => "not_found",
            Self::Service(SetDrawProposalError::Forbidden(_)) => "forbidden",
            Self::Service(SetDrawProposalError::Repository(_)) => "repository_error",
        }
    }

    ///
    /// 和平終了フラグ設定リクエストハンドラのエラーを API エラーレスポンスに変換する
    ///
    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    /// 和平終了フラグ設定リクエストハンドラのエラーに対応するエラーメッセージを生成する
    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(SetDrawProposalRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(SetDrawProposalRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(SetDrawProposalRequestValidationError::MissingAccessToken) => {
                "access token is required".to_string()
            }
            Self::InvalidRequest(SetDrawProposalRequestValidationError::InvalidGameUuid) => "game_uuid is invalid".to_string(),
            Self::Service(SetDrawProposalError::Forbidden(message)) => message.clone(),
            Self::Service(error) => error.to_string(),
        }
    }
}
