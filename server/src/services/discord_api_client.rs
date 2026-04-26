// ============================================================================
// imports
// ============================================================================

use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde::Deserialize;

use super::DiscordClientError;
use super::DiscordIdentityProvider;
use super::DiscordProfile;

// ============================================================================
// definitions
// ============================================================================

/// コネクションタイムアウト（秒）
const DISCORD_CONNECT_TIMEOUT_SECS: u64 = 3;

/// リクエストタイムアウト（秒）
const DISCORD_REQUEST_TIMEOUT_SECS: u64 = 10;

///
/// Discord API クライアントの構造体
///
pub(crate) struct DiscordApiClient {
    base_url: String,
    http_client: Client,
}

/// Discord API クライアントの構造体の実装
impl DiscordApiClient {
    ///
    /// new 関数
    ///
    pub(crate) fn new() -> Self {
        let http_client = Client::builder()
            .connect_timeout(Duration::from_secs(DISCORD_CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(DISCORD_REQUEST_TIMEOUT_SECS))
            .build()
            .expect("failed to build discord reqwest blocking client");

        Self {
            base_url: "https://discord.com/api/v10".to_string(),
            http_client,
        }
    }

    /// アバター URL を生成する
    fn build_avatar_url(user_id: &str, avatar_hash: Option<&str>) -> Option<String> {
        avatar_hash.map(|hash| format!("https://cdn.discordapp.com/avatars/{}/{}.png", user_id, hash))
    }
}

/// Discord API クライアントの構造体の実装（DiscordIdentityProvider トレイトの実装）
impl DiscordIdentityProvider for DiscordApiClient {
    /// Discord プロフィールを取得する
    fn fetch_profile(&self, discord_access_token: &str) -> Result<DiscordProfile, DiscordClientError> {
        let endpoint = format!("{}/users/@me", self.base_url);
        let response = self
            .http_client
            .get(endpoint)
            .bearer_auth(discord_access_token)
            .send()
            .map_err(|error| DiscordClientError::Unavailable(format!("request discord /users/@me: {}", error)))?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(DiscordClientError::Unauthorized);
        }

        if !response.status().is_success() {
            return Err(DiscordClientError::Unavailable(format!(
                "discord returned non-success status: {}",
                response.status()
            )));
        }

        let user: DiscordMeResponse = response
            .json()
            .map_err(|error| DiscordClientError::Unavailable(format!("decode discord user json: {}", error)))?;

        let avatar_url = Self::build_avatar_url(&user.id, user.avatar.as_deref());

        Ok(DiscordProfile {
            discord_user_id: user.id,
            username: user.username,
            global_name: user.global_name,
            avatar_hash: user.avatar,
            avatar_url,
        })
    }
}

/// Discord アカウント情報取得API のレスポンスの構造体
#[derive(Debug, Deserialize)]
struct DiscordMeResponse {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_client_with_default_base_url() {
        let client = DiscordApiClient::new();
        assert_eq!(client.base_url, "https://discord.com/api/v10");
    }

    #[test]
    fn build_avatar_url_returns_none_without_hash() {
        let url = DiscordApiClient::build_avatar_url("1001", None);
        assert_eq!(url, None);
    }

    #[test]
    fn build_avatar_url_builds_expected_url() {
        let url = DiscordApiClient::build_avatar_url("1001", Some("abc"));
        assert_eq!(url.as_deref(), Some("https://cdn.discordapp.com/avatars/1001/abc.png"));
    }
}
