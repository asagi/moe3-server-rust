// ============================================================================
// imports
// ============================================================================

use std::sync::Arc;
use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::params;

use super::NewUser;
use super::RepositoryError;
use super::UserId;
use super::UserProfileUpdate;
use super::UserRecord;
use super::UserRepository;

// ============================================================================
// definitions
// ============================================================================

///
/// SQLite 用のユーザリポジトリ構造体
///
#[derive(Clone)]
pub(crate) struct SqliteUserRepository {
    connection: Arc<Mutex<Connection>>,
}

/// SQLite 用のユーザリポジトリ構造体の実装
impl SqliteUserRepository {
    ///
    /// new 関数
    ///
    pub(crate) fn new(database_path: &str) -> Result<Self, RepositoryError> {
        let connection =
            Connection::open(database_path).map_err(|error| RepositoryError::Unavailable(format!("open sqlite: {}", error)))?;

        let repository = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        repository.init_schema()?;
        Ok(repository)
    }

    ///
    /// new 関数（テスト用）
    ///
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new_in_memory() -> Result<Self, RepositoryError> {
        let connection = Connection::open_in_memory()
            .map_err(|error| RepositoryError::Unavailable(format!("open sqlite in memory: {}", error)))?;

        let repository = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        repository.init_schema()?;
        Ok(repository)
    }

    /// スキーマを初期化する
    fn init_schema(&self) -> Result<(), RepositoryError> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                discord_user_id TEXT NOT NULL UNIQUE,
                username TEXT NOT NULL,
                global_name TEXT,
                avatar_hash TEXT,
                avatar_url TEXT,
                access_token TEXT NOT NULL UNIQUE,
                last_access_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .execute(sql, [])
            .map_err(|error| RepositoryError::Unavailable(format!("create users table: {}", error)))?;

        Ok(())
    }

    /// SQLite の Row から UserRecord を生成する
    fn row_to_user_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserRecord> {
        let uuid_str: String = row.get("uuid")?;
        let uuid = uuid::Uuid::parse_str(&uuid_str)
            .map_err(|error| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(error)))?;
        let last_access_at_str: String = row.get("last_access_at")?;
        let last_access_at = chrono::DateTime::parse_from_rfc3339(&last_access_at_str)
            .map_err(|error| rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(error)))?
            .with_timezone(&chrono::Utc);

        Ok(UserRecord {
            id: row.get("id")?,
            uuid,
            discord_user_id: row.get("discord_user_id")?,
            username: row.get("username")?,
            global_name: row.get("global_name")?,
            avatar_hash: row.get("avatar_hash")?,
            avatar_url: row.get("avatar_url")?,
            access_token: row.get("access_token")?,
            last_access_at,
        })
    }

    /// ID でユーザをロードする
    fn load_by_id(&self, user_id: UserId) -> Result<UserRecord, RepositoryError> {
        let sql = r#"
            SELECT id, uuid, discord_user_id, username, global_name, avatar_hash, avatar_url, access_token, last_access_at
            FROM users
            WHERE id = ?1
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .query_row(sql, params![user_id], Self::row_to_user_record)
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => RepositoryError::NotFound,
                _ => RepositoryError::Unavailable(format!("load user by id: {}", error)),
            })
    }
}

/// SQLite 用のユーザリポジトリ構造体の実装（UserRepository トレイト）
impl UserRepository for SqliteUserRepository {
    /// ID でユーザをロードする
    fn find_by_uuid(&self, user_uuid: uuid::Uuid) -> Result<Option<UserRecord>, RepositoryError> {
        let sql = r#"
            SELECT id, uuid, discord_user_id, username, global_name, avatar_hash, avatar_url, access_token, last_access_at
            FROM users
            WHERE uuid = ?1
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .query_row(sql, params![user_uuid.to_string()], Self::row_to_user_record)
            .optional()
            .map_err(|error| RepositoryError::Unavailable(format!("find user by uuid: {}", error)))
    }

    /// Discord ユーザ ID でユーザをロードする
    fn find_by_discord_user_id(&self, discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
        let sql = r#"
            SELECT id, uuid, discord_user_id, username, global_name, avatar_hash, avatar_url, access_token, last_access_at
            FROM users
            WHERE discord_user_id = ?1
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .query_row(sql, params![discord_user_id], Self::row_to_user_record)
            .optional()
            .map_err(|error| RepositoryError::Unavailable(format!("find user by discord_user_id: {}", error)))
    }

    /// アクセストークンでユーザをロードする
    fn find_by_access_token(&self, access_token: &str) -> Result<Option<UserRecord>, RepositoryError> {
        let sql = r#"
            SELECT id, uuid, discord_user_id, username, global_name, avatar_hash, avatar_url, access_token, last_access_at
            FROM users
            WHERE access_token = ?1
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .query_row(sql, params![access_token], Self::row_to_user_record)
            .optional()
            .map_err(|error| RepositoryError::Unavailable(format!("find user by access_token: {}", error)))
    }

    /// アクセストークンでユーザの最終アクセス日時を更新する
    fn update_last_access_at_by_access_token(
        &self,
        access_token: &str,
        last_access_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, RepositoryError> {
        let sql = r#"
            UPDATE users
            SET last_access_at = ?1,
                updated_at = ?2
            WHERE access_token = ?3
        "#;
        let now = chrono::Utc::now().to_rfc3339();

        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;
        let affected = connection
            .execute(sql, params![last_access_at.to_rfc3339(), now, access_token])
            .map_err(|error| RepositoryError::Unavailable(format!("update user last_access_at: {}", error)))?;

        Ok(affected > 0)
    }

    /// 新規ユーザを挿入する
    fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError> {
        let now = Utc::now().to_rfc3339();

        let sql = r#"
            INSERT INTO users (
                uuid,
                discord_user_id,
                username,
                global_name,
                avatar_hash,
                avatar_url,
                access_token,
                last_access_at,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#;

        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;
        let result = connection.execute(
            sql,
            params![
                new_user.uuid.to_string(),
                &new_user.discord_user_id,
                &new_user.username,
                &new_user.global_name,
                &new_user.avatar_hash,
                &new_user.avatar_url,
                &new_user.access_token,
                now,
                now,
                now,
            ],
        );

        if let Err(error) = result {
            let message = error.to_string();
            if message.contains("UNIQUE") {
                return Err(RepositoryError::Conflict);
            }
            return Err(RepositoryError::Unavailable(format!("insert user: {}", error)));
        }

        let inserted_id = connection.last_insert_rowid();
        drop(connection);

        self.load_by_id(inserted_id)
    }

    /// ユーザのプロフィールを更新する
    fn update_profile(&self, id: UserId, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
        let now = Utc::now().to_rfc3339();

        let sql = r#"
            UPDATE users
            SET username = ?1,
                global_name = ?2,
                avatar_hash = ?3,
                avatar_url = ?4,
                updated_at = ?5
            WHERE id = ?6
        "#;

        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;
        let affected = connection
            .execute(
                sql,
                params![
                    &profile.username,
                    &profile.global_name,
                    profile.avatar_hash,
                    profile.avatar_url,
                    now,
                    id
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("update user profile: {}", error)))?;

        if affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        drop(connection);
        self.load_by_id(id)
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_schema_on_file_database() {
        let file_path = std::env::temp_dir().join(format!("moe3_server_test_{}.sqlite3", uuid::Uuid::new_v4()));
        let path_string = file_path.to_str().expect("temp path should be valid unicode").to_string();

        let repository = SqliteUserRepository::new(&path_string).expect("repository should initialize");
        let found = repository.find_by_discord_user_id("not-found").expect("query should succeed");

        assert!(found.is_none());
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn load_by_id_returns_not_found_for_missing_row() {
        let repository = SqliteUserRepository::new_in_memory().expect("repository should initialize");

        let result = repository.load_by_id(-1);

        assert!(matches!(result, Err(RepositoryError::NotFound)));
    }

    #[test]
    fn insert_and_find_user() {
        let repository = SqliteUserRepository::new_in_memory().expect("repository should initialize");

        let created = repository
            .insert(NewUser {
                uuid: uuid::Uuid::now_v7(),
                discord_user_id: "1001".to_string(),
                username: "nemu".to_string(),
                global_name: Some("nemu_global".to_string()),
                avatar_hash: Some("hash".to_string()),
                avatar_url: Some("https://cdn.discordapp.com/avatar.png".to_string()),
                access_token: "token-1".to_string(),
            })
            .expect("insert should succeed");

        let fetched = repository
            .find_by_discord_user_id("1001")
            .expect("find should succeed")
            .expect("user should exist");

        assert_eq!(created.id, fetched.id);
        assert_eq!(created.uuid, fetched.uuid);
        assert_eq!(fetched.username, "nemu");
        assert_eq!(fetched.global_name.as_deref(), Some("nemu_global"));
        assert_eq!(fetched.access_token, "token-1");
        assert_eq!(created.last_access_at, fetched.last_access_at);
    }

    #[test]
    fn update_user_profile() {
        let repository = SqliteUserRepository::new_in_memory().expect("repository should initialize");

        let inserted = repository
            .insert(NewUser {
                uuid: uuid::Uuid::now_v7(),
                discord_user_id: "1001".to_string(),
                username: "old_user".to_string(),
                global_name: Some("old".to_string()),
                avatar_hash: None,
                avatar_url: Some("https://cdn.discordapp.com/old.png".to_string()),
                access_token: "token-1".to_string(),
            })
            .expect("insert should succeed");

        let inserted_id = repository
            .find_by_discord_user_id("1001")
            .expect("find should succeed")
            .expect("user should exist")
            .id;
        let updated = repository
            .update_profile(
                inserted_id,
                UserProfileUpdate {
                    username: "new_user".to_string(),
                    global_name: Some("new".to_string()),
                    avatar_hash: None,
                    avatar_url: Some("https://cdn.discordapp.com/new.png".to_string()),
                },
            )
            .expect("update should succeed");

        assert_eq!(updated.username, "new_user");
        assert_eq!(updated.global_name.as_deref(), Some("new"));
        assert_eq!(updated.avatar_url.as_deref(), Some("https://cdn.discordapp.com/new.png"));
        assert_eq!(updated.last_access_at, inserted.last_access_at);
    }

    #[test]
    fn update_last_access_at_by_access_token_updates_timestamp() {
        let repository = SqliteUserRepository::new_in_memory().expect("repository should initialize");

        let created = repository
            .insert(NewUser {
                uuid: uuid::Uuid::now_v7(),
                discord_user_id: "1001".to_string(),
                username: "nemu".to_string(),
                global_name: None,
                avatar_hash: None,
                avatar_url: None,
                access_token: "token-1".to_string(),
            })
            .expect("insert should succeed");

        let touched_at = chrono::Utc::now() + chrono::TimeDelta::minutes(5);
        let updated = repository
            .update_last_access_at_by_access_token("token-1", touched_at)
            .expect("update should succeed");

        assert!(updated);

        let fetched = repository
            .find_by_access_token("token-1")
            .expect("find should succeed")
            .expect("user should exist");
        assert_ne!(created.last_access_at, fetched.last_access_at);
        assert_eq!(fetched.last_access_at, touched_at);
    }

    #[test]
    fn update_last_access_at_by_access_token_returns_false_when_user_is_missing() {
        let repository = SqliteUserRepository::new_in_memory().expect("repository should initialize");

        let updated = repository
            .update_last_access_at_by_access_token("missing-token", chrono::Utc::now())
            .expect("update should succeed");

        assert!(!updated);
    }
}
