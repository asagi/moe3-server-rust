// ============================================================================
// definitions
// ============================================================================

///
/// ログインリクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthRequestValidationError {
    MissingDiscordAccessToken,
}

///
/// 卓作成リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateGameRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidFaceType,
    InvalidDurationType,
    InvalidStartDate,
    InvalidRequestedPower,
    InvalidFirstPeriodHour,
}

///
/// 卓参加リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JoinGameRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidRequestedPower,
}
