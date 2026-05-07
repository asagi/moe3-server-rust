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
/// トークンリセットリクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthResetTokenRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
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
    InvalidKeyword,
}

///
/// 卓参加リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JoinGameRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
    InvalidRequestedPower,
    InvalidKeyword,
}

///
/// 和平終了フラグ設定リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetDrawProposalRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
}

///
/// ユニット配置制御リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetUnitRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
    InvalidSeason,
}

///
/// 占領情報編集リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetTerritoryRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
    InvalidSeason,
}

///
/// 進行モード変更リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetProgressModeRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
    InvalidSeason,
}

///
/// 即時進行合意設定リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetProgressConsensusRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
}

///
/// 次回更新時刻変更リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SetNextUpdateAtRequestValidationError {
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidGameUuid,
    InvalidSeason,
}

///
/// 卓一覧取得リクエストのバリデーションエラーの列挙体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GetGamesRequestValidationError {
    ConflictingParams,
    InvalidStatus,
    MissingAuthorization,
    InvalidAuthorizationScheme,
    MissingAccessToken,
    InvalidPage,
    InvalidPerPage,
}
