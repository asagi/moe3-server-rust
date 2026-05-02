// ============================================================================
// imports
// ============================================================================

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::params;
use serde_json::json;
use uuid::Uuid;

use super::Message;
use super::MessageKind;
use super::Power;
use super::RepositoryError;
use super::SystemNotice;
use super::SystemNoticeCatalog;
use super::User;

// ============================================================================
// definitions
// ============================================================================

///
/// SQLite 用のメッセージリポジトリ構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SqliteMessageRepository {
    base_database_path: String,
}

/// SQLite 用のメッセージリポジトリ構造体の実装
impl SqliteMessageRepository {
    ///
    /// new 関数
    ///
    pub(crate) fn new(database_path: &str) -> Self {
        Self {
            base_database_path: database_path.to_string(),
        }
    }

    ///
    /// 卓ごとのメッセージ DB を生成し、初期の GameCreated メッセージを保存する
    ///
    pub(crate) fn create_game_db_with_game_created_message(&self, game_uuid: Uuid, user: &User) -> Result<(), RepositoryError> {
        let db_path = self.message_database_path(game_uuid);
        let connection = Connection::open(&db_path)
            .map_err(|error| RepositoryError::Unavailable(format!("open message sqlite: {}", error)))?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::GameCreated { user: user.clone() };
        let message = Message {
            sender: None,
            turn: "ready".to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_initial_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    /// メッセージ DB ファイルパスを生成する
    fn message_database_path(&self, game_uuid: Uuid) -> String {
        format!("{}.messages.{}.sqlite3", self.base_database_path, game_uuid)
    }

    /// スキーマを初期化する
    fn init_schema(&self, connection: &Connection) -> Result<(), RepositoryError> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS messages (
                message_uuid TEXT PRIMARY KEY,
                game_uuid TEXT NOT NULL,
                sender_power TEXT,
                turn TEXT NOT NULL,
                context TEXT NOT NULL,
                kind TEXT NOT NULL,
                system_notice_catalog TEXT,
                kind_payload TEXT,
                created_at TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                deleted_at TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_messages_game_created_at
                ON messages(game_uuid, created_at);

            CREATE INDEX IF NOT EXISTS idx_messages_game_deleted_created_at
                ON messages(game_uuid, is_deleted, created_at);
        "#;

        connection
            .execute_batch(sql)
            .map_err(|error| RepositoryError::Unavailable(format!("create message tables: {}", error)))?;

        Ok(())
    }

    /// 初期メッセージを挿入する
    fn insert_initial_message(
        &self,
        connection: &Connection,
        game_uuid: Uuid,
        message: &Message,
        catalog: &SystemNoticeCatalog,
    ) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();
        let message_uuid = Uuid::now_v7();

        let sender_power = message.sender.map(Self::power_to_symbol);
        let kind = Self::kind_to_text(&message.kind);
        let system_notice_catalog = Self::catalog_to_text(catalog);
        let kind_payload = Self::catalog_payload(catalog).to_string();

        connection
            .execute(
                r#"
                INSERT INTO messages (
                    message_uuid,
                    game_uuid,
                    sender_power,
                    turn,
                    context,
                    kind,
                    system_notice_catalog,
                    kind_payload,
                    created_at,
                    is_deleted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0)
                "#,
                params![
                    message_uuid.to_string(),
                    game_uuid.to_string(),
                    sender_power,
                    message.turn,
                    message.context,
                    kind,
                    system_notice_catalog,
                    kind_payload,
                    now,
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("insert initial message: {}", error)))?;

        Ok(())
    }

    fn power_to_symbol(power: Power) -> String {
        power.symbol().to_string()
    }

    fn kind_to_text(kind: &MessageKind) -> &'static str {
        match kind {
            MessageKind::Public(_) => "public",
            MessageKind::Confidential(_) => "confidential",
            MessageKind::Personal(_) => "personal",
            MessageKind::Ghost(_) => "ghost",
            MessageKind::System(_) => "system",
        }
    }

    fn catalog_to_text(catalog: &SystemNoticeCatalog) -> &'static str {
        match catalog {
            SystemNoticeCatalog::GameCreated { .. } => "game_created",
            SystemNoticeCatalog::PlayerJoined { .. } => "player_joined",
            SystemNoticeCatalog::Ready => "ready",
            SystemNoticeCatalog::Aborted => "aborted",
            SystemNoticeCatalog::StartSeason { .. } => "start_season",
            SystemNoticeCatalog::PowerEliminated { .. } => "power_eliminated",
            SystemNoticeCatalog::SettlementProposed => "settlement_proposed",
            SystemNoticeCatalog::OwnerAbsent => "owner_absent",
            SystemNoticeCatalog::SettlementRescinded => "settlement_rescinded",
            SystemNoticeCatalog::Solo { .. } => "solo",
            SystemNoticeCatalog::Draw => "draw",
            SystemNoticeCatalog::Closed => "closed",
        }
    }

    fn catalog_payload(catalog: &SystemNoticeCatalog) -> serde_json::Value {
        match catalog {
            SystemNoticeCatalog::GameCreated { user } | SystemNoticeCatalog::PlayerJoined { user } => json!({
                "user": {
                    "uuid": user.uuid.to_string(),
                    "discord_user_id": user.discord_user_id,
                    "username": user.username,
                    "global_name": user.global_name,
                    "avatar_hash": user.avatar_hash,
                    "avatar_url": user.avatar_url,
                }
            }),
            SystemNoticeCatalog::StartSeason { season } => json!({ "season": season }),
            SystemNoticeCatalog::PowerEliminated { power } | SystemNoticeCatalog::Solo { power } => {
                json!({ "power": power.symbol() })
            }
            SystemNoticeCatalog::Ready
            | SystemNoticeCatalog::Aborted
            | SystemNoticeCatalog::SettlementProposed
            | SystemNoticeCatalog::OwnerAbsent
            | SystemNoticeCatalog::SettlementRescinded
            | SystemNoticeCatalog::Draw
            | SystemNoticeCatalog::Closed => json!({}),
        }
    }
}
