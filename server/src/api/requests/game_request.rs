// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;
use uuid::Uuid;

use super::CreateGameRequestValidationError;
use super::JoinGameRequestValidationError;

// ============================================================================
// definitions
// ============================================================================

///
/// 卓作成リクエストパラメータボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct CreateGameRequestBody {
    pub face_type: i32,
    pub duration_type: i32,
    pub start_date: String,
    pub first_period_hour: u8,
    pub requested_power: Option<String>,
    pub keyword: Option<String>,
}

///
/// 卓作成リクエストの構造体
///
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct CreateGameRequest {
    pub authorization: String,
    pub face_type: i32,
    pub duration_type: i32,
    pub start_date: String,
    pub first_period_hour: u8,
    pub requested_power: Option<String>,
    pub keyword: Option<String>,
}

/// 卓作成リクエストの構造体の実装
impl CreateGameRequest {
    pub(crate) fn validate(&self) -> Result<(), CreateGameRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(CreateGameRequestValidationError::MissingAuthorization);
        }

        if !auth.starts_with("Bearer ") {
            return Err(CreateGameRequestValidationError::InvalidAuthorizationScheme);
        }

        let token = auth.trim_start_matches("Bearer ").trim();
        if token.is_empty() {
            return Err(CreateGameRequestValidationError::MissingAccessToken);
        }

        Ok(())
    }
}

///
/// 卓参加リクエストパラメータボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct JoinGameRequestBody {
    pub requested_power: Option<String>,
    pub keyword: Option<String>,
}

///
/// 卓参加リクエストの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JoinGameRequest {
    pub authorization: String,
    pub game_uuid: Uuid,
    pub requested_power: Option<String>,
    pub keyword: Option<String>,
}

/// 卓参加リクエストの構造体の実装
impl JoinGameRequest {
    pub(crate) fn validate(&self) -> Result<(), JoinGameRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(JoinGameRequestValidationError::MissingAuthorization);
        }

        if !auth.starts_with("Bearer ") {
            return Err(JoinGameRequestValidationError::InvalidAuthorizationScheme);
        }

        let token = auth.trim_start_matches("Bearer ").trim();
        if token.is_empty() {
            return Err(JoinGameRequestValidationError::MissingAccessToken);
        }

        Ok(())
    }
}
