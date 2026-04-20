#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// external crates
use serde::Serialize;
use uuid::Uuid;

// ============================================================================
// definitions
// ============================================================================

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuthLoginResponse {
    pub access_token: String,
    pub user: AuthLoginUserResponse,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuthLoginUserResponse {
    pub uuid: Uuid,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ApiErrorResponse {
    pub code: &'static str,
    pub message: String,
}
