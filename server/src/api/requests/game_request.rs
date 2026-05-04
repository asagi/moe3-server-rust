// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;
use uuid::Uuid;

use super::CreateGameRequestValidationError;
use super::JoinGameRequestValidationError;
use super::SetDrawProposalRequestValidationError;
use super::SetUnitRequestValidationError;

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

///
/// 和平終了フラグ設定リクエストパラメータボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct SetDrawProposalRequestBody {
    pub draw_proposal: bool,
}

///
/// 和平終了フラグ設定リクエストの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SetDrawProposalRequest {
    pub authorization: String,
    pub game_uuid: uuid::Uuid,
    pub draw_proposal: bool,
}

/// 和平終了フラグ設定リクエストの構造体の実装
impl SetDrawProposalRequest {
    pub(crate) fn validate(&self) -> Result<(), SetDrawProposalRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(SetDrawProposalRequestValidationError::MissingAuthorization);
        }

        let token = match auth.get(..7) {
            Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
            _ => return Err(SetDrawProposalRequestValidationError::InvalidAuthorizationScheme),
        };
        if token.is_empty() {
            return Err(SetDrawProposalRequestValidationError::MissingAccessToken);
        }

        Ok(())
    }
}

///
/// ユニット配置制御リクエストパラメータのユニット指定ボディ構造体
///
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UnitSpecBody {
    pub power: String,
    pub kind: String,
}

///
/// ユニット配置制御リクエストパラメータボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct SetUnitRequestBody {
    pub unit: UnitSpecBody,
}

///
/// ユニット配置制御リクエストの構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SetUnitRequest {
    pub authorization: String,
    pub game_uuid: Uuid,
    pub unit: Option<UnitSpecBody>,
    pub location: String,
}

/// ユニット配置制御リクエストの構造体の実装
impl SetUnitRequest {
    pub(crate) fn validate(&self) -> Result<(), SetUnitRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(SetUnitRequestValidationError::MissingAuthorization);
        }

        let token = match auth.get(..7) {
            Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
            _ => return Err(SetUnitRequestValidationError::InvalidAuthorizationScheme),
        };
        if token.is_empty() {
            return Err(SetUnitRequestValidationError::MissingAccessToken);
        }

        Ok(())
    }
}
