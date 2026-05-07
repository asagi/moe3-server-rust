// ============================================================================
// imports
// ============================================================================

use serde::Deserialize;
use uuid::Uuid;

use super::CreateGameRequestValidationError;
use super::GetGamesRequestValidationError;
use super::JoinGameRequestValidationError;
use super::SetDrawProposalRequestValidationError;
use super::SetNextUpdateAtRequestValidationError;
use super::SetProgressConsensusRequestValidationError;
use super::SetProgressModeRequestValidationError;
use super::SetTerritoryRequestValidationError;
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
    pub season: String,
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
    pub season: String,
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

        if !is_valid_season(&self.season) {
            return Err(SetUnitRequestValidationError::InvalidSeason);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetTerritoryRequestBody {
    pub power: String,
    pub season: String,
}

///
/// 占領情報編集リクエストの構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SetTerritoryRequest {
    pub authorization: String,
    pub game_uuid: Uuid,
    pub code: String,
    pub power: Option<String>,
    pub season: String,
}

/// 占領情報編集リクエストの構造体の実装
impl SetTerritoryRequest {
    pub(crate) fn validate(&self) -> Result<(), SetTerritoryRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(SetTerritoryRequestValidationError::MissingAuthorization);
        }

        let token = match auth.get(..7) {
            Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
            _ => return Err(SetTerritoryRequestValidationError::InvalidAuthorizationScheme),
        };
        if token.is_empty() {
            return Err(SetTerritoryRequestValidationError::MissingAccessToken);
        }

        if !is_valid_season(&self.season) {
            return Err(SetTerritoryRequestValidationError::InvalidSeason);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeleteUnitQueryParams {
    pub season: String,
}

///
/// 占領情報削除リクエストのクエリパラメータ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct DeleteTerritoryQueryParams {
    pub season: String,
}

/// season 文字列が有効な形式かどうかを検証する（例: "1901s", "1901F"）
fn is_valid_season(season: &str) -> bool {
    let bytes = season.as_bytes();
    if bytes.len() != 5 {
        return false;
    }
    bytes[..4].iter().all(|b| b.is_ascii_digit()) && matches!(bytes[4], b's' | b'S' | b'f' | b'F')
}

/// season 文字列が次回更新時刻変更 API で有効な形式かどうかを検証する（"ready" または "1901s" 形式）
fn is_valid_next_update_season(season: &str) -> bool {
    season == "ready" || is_valid_season(season)
}

///
/// 進行モード変更リクエストボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct SetProgressModeRequestBody {
    pub season: String,
}

///
/// 進行モード変更リクエストの構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SetProgressModeRequest {
    pub authorization: String,
    pub game_uuid: Uuid,
    pub season: String,
}

/// 進行モード変更リクエストの構造体の実装
impl SetProgressModeRequest {
    pub(crate) fn validate(&self) -> Result<(), SetProgressModeRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(SetProgressModeRequestValidationError::MissingAuthorization);
        }

        let token = match auth.get(..7) {
            Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
            _ => return Err(SetProgressModeRequestValidationError::InvalidAuthorizationScheme),
        };
        if token.is_empty() {
            return Err(SetProgressModeRequestValidationError::MissingAccessToken);
        }

        if !is_valid_season(&self.season) {
            return Err(SetProgressModeRequestValidationError::InvalidSeason);
        }

        Ok(())
    }
}

///
/// 即時進行合意設定リクエストボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct SetProgressConsensusRequestBody {
    pub agreed: bool,
}

///
/// 即時進行合意設定リクエストの構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SetProgressConsensusRequest {
    pub authorization: String,
    pub game_uuid: Uuid,
    pub agreed: bool,
}

/// 即時進行合意設定リクエストの構造体の実装
impl SetProgressConsensusRequest {
    pub(crate) fn validate(&self) -> Result<(), SetProgressConsensusRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(SetProgressConsensusRequestValidationError::MissingAuthorization);
        }

        let token = match auth.get(..7) {
            Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
            _ => return Err(SetProgressConsensusRequestValidationError::InvalidAuthorizationScheme),
        };
        if token.is_empty() {
            return Err(SetProgressConsensusRequestValidationError::MissingAccessToken);
        }

        Ok(())
    }
}

///
/// 次回更新時刻変更リクエストボディ構造体
///
#[derive(Debug, Deserialize)]
pub(crate) struct SetNextUpdateAtRequestBody {
    pub next_update_at: String,
    pub season: String,
}

///
/// 次回更新時刻変更リクエストの構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SetNextUpdateAtRequest {
    pub authorization: String,
    pub game_uuid: Uuid,
    pub next_update_at: String,
    pub season: String,
}

/// 次回更新時刻変更リクエストの構造体の実装
impl SetNextUpdateAtRequest {
    pub(crate) fn validate(&self) -> Result<(), SetNextUpdateAtRequestValidationError> {
        let auth = self.authorization.trim();
        if auth.is_empty() {
            return Err(SetNextUpdateAtRequestValidationError::MissingAuthorization);
        }

        let token = match auth.get(..7) {
            Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
            _ => return Err(SetNextUpdateAtRequestValidationError::InvalidAuthorizationScheme),
        };
        if token.is_empty() {
            return Err(SetNextUpdateAtRequestValidationError::MissingAccessToken);
        }

        if !is_valid_next_update_season(&self.season) {
            return Err(SetNextUpdateAtRequestValidationError::InvalidSeason);
        }

        Ok(())
    }
}

///
/// 卓一覧取得クエリパラメータ構造体（Axum Query 抽出用）
///
#[derive(Debug, Deserialize)]
pub(crate) struct GetGamesQueryParams {
    pub status: Option<String>,
    pub user: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

///
/// 卓一覧取得リクエストの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GetGamesRequest {
    pub authorization: Option<String>,
    pub status: Option<String>,
    pub user: Option<String>,
    pub page: u32,
    pub per_page: u32,
}

/// 卓一覧取得リクエストの構造体の実装
impl GetGamesRequest {
    pub(crate) fn validate(&self) -> Result<(), GetGamesRequestValidationError> {
        if self.status.is_some() && self.user.is_some() {
            return Err(GetGamesRequestValidationError::ConflictingParams);
        }

        if let Some(status) = &self.status {
            match status.as_str() {
                "active" | "closed" | "aborted" => {}
                _ => return Err(GetGamesRequestValidationError::InvalidStatus),
            }
        }

        if let Some(user) = &self.user {
            if user.trim().is_empty() {
                return Err(GetGamesRequestValidationError::InvalidUser);
            }
            let auth = self.authorization.as_deref().unwrap_or("").trim_start();
            if auth.trim().is_empty() {
                return Err(GetGamesRequestValidationError::MissingAuthorization);
            }
            let token = match auth.get(..7) {
                Some(prefix) if prefix.eq_ignore_ascii_case("Bearer ") => auth.get(7..).unwrap_or("").trim(),
                _ => return Err(GetGamesRequestValidationError::InvalidAuthorizationScheme),
            };
            if token.is_empty() {
                return Err(GetGamesRequestValidationError::MissingAccessToken);
            }
        }

        if self.page < 1 {
            return Err(GetGamesRequestValidationError::InvalidPage);
        }

        if self.per_page < 1 || self.per_page > 100 {
            return Err(GetGamesRequestValidationError::InvalidPerPage);
        }

        Ok(())
    }

    pub(crate) fn access_token(&self) -> &str {
        self.authorization
            .as_deref()
            .unwrap_or("")
            .trim()
            .split_once(' ')
            .map(|(_, token)| token.trim())
            .unwrap_or("")
    }
}
