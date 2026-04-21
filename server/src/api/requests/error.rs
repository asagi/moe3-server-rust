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
    InvalidProgressMode,
    InvalidDurationType,
    InvalidStartDate,
    InvalidRequestedPower,
    InvalidFirstPeriodHour,
}
