use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct LoginResponse {
    pub access_token: String,
    pub user: LoginUserResponse,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct LoginUserResponse {
    pub discord_user_id: String,
    pub display_name: String,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ApiErrorResponse {
    pub code: &'static str,
    pub message: String,
}
