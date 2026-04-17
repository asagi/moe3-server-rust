use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite::params;

use super::NewUser;
use super::RepositoryError;
use super::UserProfileUpdate;
use super::UserRecord;
use super::UserRepository;

pub(crate) struct SqliteUserRepository {
    connection: Mutex<Connection>,
}

impl SqliteUserRepository {
    pub(crate) fn new(database_path: &str) -> Result<Self, RepositoryError> {
        let connection =
            Connection::open(database_path).map_err(|error| RepositoryError::Unavailable(format!("open sqlite: {}", error)))?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.init_schema()?;
        Ok(repository)
    }

    #[cfg(test)]
    pub(crate) fn new_in_memory() -> Result<Self, RepositoryError> {
        let connection = Connection::open_in_memory()
            .map_err(|error| RepositoryError::Unavailable(format!("open sqlite in memory: {}", error)))?;

        let repository = Self {
            connection: Mutex::new(connection),
        };
        repository.init_schema()?;
        Ok(repository)
    }

    fn init_schema(&self) -> Result<(), RepositoryError> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                discord_user_id TEXT NOT NULL UNIQUE,
                username TEXT NOT NULL,
                global_name TEXT,
                avatar_hash TEXT,
                avatar_url TEXT,
                access_token TEXT NOT NULL UNIQUE,
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

    fn row_to_user_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserRecord> {
        Ok(UserRecord {
            id: row.get("id")?,
            discord_user_id: row.get("discord_user_id")?,
            username: row.get("username")?,
            global_name: row.get("global_name")?,
            avatar_hash: row.get("avatar_hash")?,
            avatar_url: row.get("avatar_url")?,
            access_token: row.get("access_token")?,
        })
    }

    fn load_by_id(&self, user_id: i64) -> Result<UserRecord, RepositoryError> {
        let sql = r#"
            SELECT id, discord_user_id, username, global_name, avatar_hash, avatar_url, access_token
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

impl UserRepository for SqliteUserRepository {
    fn find_by_discord_user_id(&self, discord_user_id: &str) -> Result<Option<UserRecord>, RepositoryError> {
        let sql = r#"
            SELECT id, discord_user_id, username, global_name, avatar_hash, avatar_url, access_token
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

    fn insert(&self, new_user: NewUser) -> Result<UserRecord, RepositoryError> {
        let now = Utc::now().to_rfc3339();

        let sql = r#"
            INSERT INTO users (
                discord_user_id,
                username,
                global_name,
                avatar_hash,
                avatar_url,
                access_token,
                created_at,
                updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#;

        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;
        let result = connection.execute(
            sql,
            params![
                &new_user.discord_user_id,
                &new_user.username,
                &new_user.global_name,
                &new_user.avatar_hash,
                &new_user.avatar_url,
                &new_user.access_token,
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

    fn update_profile(&self, discord_user_id: &str, profile: UserProfileUpdate) -> Result<UserRecord, RepositoryError> {
        let now = Utc::now().to_rfc3339();

        let sql = r#"
            UPDATE users
            SET username = ?1,
                global_name = ?2,
                avatar_hash = ?3,
                avatar_url = ?4,
                updated_at = ?5
            WHERE discord_user_id = ?6
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
                    discord_user_id
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("update user profile: {}", error)))?;

        if affected == 0 {
            return Err(RepositoryError::NotFound);
        }

        let id: i64 = connection
            .query_row(
                "SELECT id FROM users WHERE discord_user_id = ?1",
                params![discord_user_id],
                |row| row.get(0),
            )
            .map_err(|error| RepositoryError::Unavailable(format!("load updated user id: {}", error)))?;

        drop(connection);
        self.load_by_id(id)
    }
}

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
        assert_eq!(fetched.username, "nemu");
        assert_eq!(fetched.global_name.as_deref(), Some("nemu_global"));
        assert_eq!(fetched.access_token, "token-1");
    }

    #[test]
    fn update_user_profile() {
        let repository = SqliteUserRepository::new_in_memory().expect("repository should initialize");

        repository
            .insert(NewUser {
                discord_user_id: "1001".to_string(),
                username: "old_user".to_string(),
                global_name: Some("old".to_string()),
                avatar_hash: None,
                avatar_url: Some("https://cdn.discordapp.com/old.png".to_string()),
                access_token: "token-1".to_string(),
            })
            .expect("insert should succeed");

        let updated = repository
            .update_profile(
                "1001",
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
    }
}
