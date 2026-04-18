use serde::Serialize;

use crate::services::LoginUser;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuthLoginResponse {
    pub access_token: String,
    pub user: LoginUser,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ApiErrorResponse {
    pub code: &'static str,
    pub message: String,
}
