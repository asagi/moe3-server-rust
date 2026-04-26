// ============================================================================
// imports
// ============================================================================

use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use super::RepositoryError;
use super::UserId;

// ============================================================================
// definitions
// ============================================================================

///
/// Discord プロフィールの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscordProfile {
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

///
/// ユーザーレコードの構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserRecord {
    pub id: UserId,
    pub uuid: Uuid,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
    pub last_access_at: DateTime<Utc>,
}

///
/// 新規ユーザー生成用の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewUser {
    pub uuid: Uuid,
    pub discord_user_id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,
}

///
/// ユーザープロフィール更新用の構造体
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserProfileUpdate {
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub avatar_url: Option<String>,
}

/// ユーザープロフィール更新用の構造体への変換の実装（From トレイト）
impl From<&DiscordProfile> for UserProfileUpdate {
    fn from(profile: &DiscordProfile) -> Self {
        Self {
            username: profile.username.clone(),
            global_name: profile.global_name.clone(),
            avatar_hash: profile.avatar_hash.clone(),
            avatar_url: profile.avatar_url.clone(),
        }
    }
}

///
/// ユーザリポジトリのトレイト
///
pub(crate) trait UserRepository {
    /// ID でユーザをロードする
    fn find_by_uuid(&self, user_uuid: Uuid) -> Result<Option<UserRecord>, RepositoryError>;

    /// Discord ユーザ ID でユーザをロードする
    fn find_by_discord_user_id(&self, discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError>;

    /// アクセストークンでユーザをロードする
    fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError>;

    /// アクセストークンでユーザの最終アクセス日時を更新する
    fn update_last_access_at_by_access_token(
        &self,
        access_token: &str,
        last_access_at: DateTime<Utc>,
    ) -> Result<bool, RepositoryError>;

    /// 新規ユーザを挿入する
    fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError>;

    /// ユーザのプロフィールを更新する
    fn update_profile(&self, id: UserId, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError>;
}
