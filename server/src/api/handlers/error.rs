// ============================================================================
// imports
// ============================================================================

use super::ApiErrorResponse;
use super::AuthError;
use super::AuthRequestValidationError;
use super::AuthResetTokenRequestValidationError;
use super::CreateGameError;
use super::CreateGameRequestValidationError;
use super::DiscordClientError;
use super::JoinGameError;
use super::JoinGameRequestValidationError;
use super::SetDrawProposalError;
use super::SetDrawProposalRequestValidationError;
use super::SetNextUpdateAtError;
use super::SetNextUpdateAtRequestValidationError;
use super::SetProgressConsensusError;
use super::SetProgressConsensusRequestValidationError;
use super::SetProgressModeError;
use super::SetProgressModeRequestValidationError;
use super::SetTerritoryError;
use super::SetTerritoryRequestValidationError;
use super::SetUnitError;
use super::SetUnitRequestValidationError;

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
            Self::Service(AuthError::Unauthorized) => "unauthorized",
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
/// トークンリセットリクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum ResetTokenHandlerError {
    InvalidRequest(AuthResetTokenRequestValidationError),
    Service(AuthError),
}

/// トークンリセットリクエストハンドラのエラーの列挙体の実装
impl ResetTokenHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(AuthError::Unauthorized) => "unauthorized",
            Self::Service(AuthError::Repository(_)) => "repository_error",
            _ => "internal_error",
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
            Self::InvalidRequest(AuthResetTokenRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(AuthResetTokenRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(AuthResetTokenRequestValidationError::MissingAccessToken) => {
                "access token is required".to_string()
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

///
/// ユニット配置制御リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum SetUnitHandlerError {
    InvalidRequest(SetUnitRequestValidationError),
    Service(SetUnitError),
}

/// ユニット配置制御リクエストハンドラのエラーの列挙体の実装
impl SetUnitHandlerError {
    ///
    /// エラーコードを取得する
    ///
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(SetUnitError::Unauthorized) => "unauthorized",
            Self::Service(SetUnitError::NotFound) => "not_found",
            Self::Service(SetUnitError::Forbidden(_)) => "forbidden",
            Self::Service(SetUnitError::InvalidRequest(_)) => "invalid_request",
            Self::Service(SetUnitError::PhaseConflict) => "phase_conflict",
            Self::Service(SetUnitError::Repository(_)) => "repository_error",
        }
    }

    ///
    /// ユニット配置制御リクエストハンドラのエラーを API エラーレスポンスに変換する
    ///
    pub(crate) fn to_api_error_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code(),
            message: self.message(),
        }
    }

    /// ユニット配置制御リクエストハンドラのエラーに対応するエラーメッセージを生成する
    fn message(&self) -> String {
        match self {
            Self::InvalidRequest(SetUnitRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(SetUnitRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(SetUnitRequestValidationError::MissingAccessToken) => "access token is required".to_string(),
            Self::InvalidRequest(SetUnitRequestValidationError::InvalidGameUuid) => "game_uuid is invalid".to_string(),
            Self::InvalidRequest(SetUnitRequestValidationError::InvalidSeason) => {
                "season must be in the format like '1901s' or '1901f'".to_string()
            }
            Self::Service(SetUnitError::Forbidden(message)) | Self::Service(SetUnitError::InvalidRequest(message)) => {
                message.clone()
            }
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 占領情報編集リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum SetTerritoryHandlerError {
    InvalidRequest(SetTerritoryRequestValidationError),
    Service(SetTerritoryError),
}

/// 占領情報編集リクエストハンドラのエラーの列挙体の実装
impl SetTerritoryHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(SetTerritoryError::Unauthorized) => "unauthorized",
            Self::Service(SetTerritoryError::NotFound) => "not_found",
            Self::Service(SetTerritoryError::WaterProvince) => "not_found",
            Self::Service(SetTerritoryError::Forbidden(_)) => "forbidden",
            Self::Service(SetTerritoryError::InvalidRequest(_)) => "invalid_request",
            Self::Service(SetTerritoryError::PhaseConflict) => "phase_conflict",
            Self::Service(SetTerritoryError::Repository(_)) => "repository_error",
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
            Self::InvalidRequest(SetTerritoryRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(SetTerritoryRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(SetTerritoryRequestValidationError::MissingAccessToken) => {
                "access token is required".to_string()
            }
            Self::InvalidRequest(SetTerritoryRequestValidationError::InvalidGameUuid) => "game_uuid is invalid".to_string(),
            Self::InvalidRequest(SetTerritoryRequestValidationError::InvalidSeason) => {
                "season must be in the format like '1901s' or '1901f'".to_string()
            }
            Self::Service(SetTerritoryError::Forbidden(message)) | Self::Service(SetTerritoryError::InvalidRequest(message)) => {
                message.clone()
            }
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 進行モード変更リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum SetProgressModeHandlerError {
    InvalidRequest(SetProgressModeRequestValidationError),
    Service(SetProgressModeError),
}

/// 進行モード変更リクエストハンドラのエラーの列挙体の実装
impl SetProgressModeHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(SetProgressModeError::Unauthorized) => "unauthorized",
            Self::Service(SetProgressModeError::NotFound) => "not_found",
            Self::Service(SetProgressModeError::Forbidden(_)) => "forbidden",
            Self::Service(SetProgressModeError::PhaseConflict) => "phase_conflict",
            Self::Service(SetProgressModeError::Repository(_)) => "repository_error",
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
            Self::InvalidRequest(SetProgressModeRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(SetProgressModeRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(SetProgressModeRequestValidationError::MissingAccessToken) => {
                "access token is required".to_string()
            }
            Self::InvalidRequest(SetProgressModeRequestValidationError::InvalidGameUuid) => "game_uuid is invalid".to_string(),
            Self::InvalidRequest(SetProgressModeRequestValidationError::InvalidSeason) => {
                "season must be in the format like '1901s' or '1901f'".to_string()
            }
            Self::Service(SetProgressModeError::Forbidden(message)) => message.clone(),
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 即時進行合意設定リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum SetProgressConsensusHandlerError {
    InvalidRequest(SetProgressConsensusRequestValidationError),
    Service(SetProgressConsensusError),
}

/// 即時進行合意設定リクエストハンドラのエラーの列挙体の実装
impl SetProgressConsensusHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(SetProgressConsensusError::Unauthorized) => "unauthorized",
            Self::Service(SetProgressConsensusError::NotFound) => "not_found",
            Self::Service(SetProgressConsensusError::Forbidden(_)) => "forbidden",
            Self::Service(SetProgressConsensusError::Repository(_)) => "repository_error",
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
            Self::InvalidRequest(SetProgressConsensusRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(SetProgressConsensusRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(SetProgressConsensusRequestValidationError::MissingAccessToken) => {
                "access token is required".to_string()
            }
            Self::InvalidRequest(SetProgressConsensusRequestValidationError::InvalidGameUuid) => {
                "game_uuid is invalid".to_string()
            }
            Self::Service(SetProgressConsensusError::Forbidden(message)) => message.clone(),
            Self::Service(error) => error.to_string(),
        }
    }
}

///
/// 次回更新時刻変更リクエストハンドラのエラーの列挙体
///
#[derive(Debug)]
pub(crate) enum SetNextUpdateAtHandlerError {
    InvalidRequest(SetNextUpdateAtRequestValidationError),
    Service(SetNextUpdateAtError),
}

/// 次回更新時刻変更リクエストハンドラのエラーの列挙体の実装
impl SetNextUpdateAtHandlerError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::Service(SetNextUpdateAtError::Unauthorized) => "unauthorized",
            Self::Service(SetNextUpdateAtError::NotFound) => "not_found",
            Self::Service(SetNextUpdateAtError::Forbidden(_)) => "forbidden",
            Self::Service(SetNextUpdateAtError::InvalidRequest(_)) => "invalid_request",
            Self::Service(SetNextUpdateAtError::PhaseConflict) => "phase_conflict",
            Self::Service(SetNextUpdateAtError::Repository(_)) => "repository_error",
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
            Self::InvalidRequest(SetNextUpdateAtRequestValidationError::MissingAuthorization) => {
                "authorization header is required".to_string()
            }
            Self::InvalidRequest(SetNextUpdateAtRequestValidationError::InvalidAuthorizationScheme) => {
                "authorization must start with 'Bearer <token>'".to_string()
            }
            Self::InvalidRequest(SetNextUpdateAtRequestValidationError::MissingAccessToken) => {
                "access token is required".to_string()
            }
            Self::InvalidRequest(SetNextUpdateAtRequestValidationError::InvalidGameUuid) => "game_uuid is invalid".to_string(),
            Self::InvalidRequest(SetNextUpdateAtRequestValidationError::InvalidSeason) => {
                "season must be 'ready' or in the format like '1901s' or '1901f'".to_string()
            }
            Self::Service(SetNextUpdateAtError::Forbidden(message))
            | Self::Service(SetNextUpdateAtError::InvalidRequest(message)) => message.clone(),
            Self::Service(error) => error.to_string(),
        }
    }
}
