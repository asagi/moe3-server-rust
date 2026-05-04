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
use super::Unit;
use super::User;

// ============================================================================
// definitions
// ============================================================================

///
/// SQLite 用のメッセージリポジトリ構造体
///
#[derive(Debug, Clone)]
pub(crate) struct SqliteMessageRepository {
    message_database_path: String,
}

const READY_TURN: &str = "ready";
const DEBRIEF_TURN: &str = "debrief";

/// SQLite 用のメッセージリポジトリ構造体の実装
impl SqliteMessageRepository {
    ///
    /// new 関数
    ///
    pub(crate) fn new(messages_database_path: &str) -> Self {
        Self {
            message_database_path: messages_database_path.to_string(),
        }
    }

    ///
    /// メッセージ DB を生成し、初期の GameCreated メッセージを保存する
    ///
    pub(crate) fn create_game_db_with_game_created_message(&self, game_uuid: Uuid, user: &User) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::GameCreated { user: user.clone() };
        let message = Message {
            sender: None,
            turn: READY_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 参加表明のシステムメッセージを保存する
    ///
    pub(crate) fn append_player_joined_message(&self, game_uuid: Uuid, user: &User) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::PlayerJoined { user: user.clone() };
        let message = Message {
            sender: None,
            turn: READY_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 7 人が揃い担当国割り当てが完了したシステムメッセージを保存する
    ///
    pub(crate) fn append_ready_message(&self, game_uuid: Uuid) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::Ready;
        let message = Message {
            sender: None,
            turn: READY_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 募集不成立による中止のシステムメッセージを保存する
    ///
    pub(crate) fn append_aborted_message(&self, game_uuid: Uuid) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::Aborted;
        let message = Message {
            sender: None,
            turn: READY_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 季節ごとのメインフェイズ開始のシステムメッセージを保存する
    ///
    pub(crate) fn append_start_season_message(&self, game_uuid: Uuid, turn: &str, season: &str) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::StartSeason {
            season: season.to_string(),
        };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 卓主無政府化のシステムメッセージを保存する
    ///
    pub(crate) fn append_owner_absent_message(&self, game_uuid: Uuid, turn: &str) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::OwnerAbsent;
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 制覇終了のシステムメッセージを保存する
    ///
    pub(crate) fn append_solo_message(&self, game_uuid: Uuid, power: Power) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::Solo { power };
        let message = Message {
            sender: None,
            turn: DEBRIEF_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 卓閉鎖のシステムメッセージを保存する
    ///
    pub(crate) fn append_closed_message(&self, game_uuid: Uuid) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::Closed;
        let message = Message {
            sender: None,
            turn: DEBRIEF_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 講和成立のシステムメッセージを保存する
    ///
    pub(crate) fn append_draw_message(&self, game_uuid: Uuid) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::Draw;
        let message = Message {
            sender: None,
            turn: DEBRIEF_TURN.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 卓主による講和宣言のシステムメッセージを保存する
    ///
    pub(crate) fn append_draw_proposed_message(&self, game_uuid: Uuid, turn: &str) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::DrawProposed;
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// 卓主による講和撤回のシステムメッセージを保存する
    ///
    pub(crate) fn append_draw_rescinded_message(&self, game_uuid: Uuid, turn: &str) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;

        self.init_schema(&connection)?;

        let catalog = SystemNoticeCatalog::DrawRescinded;
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };

        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// ユニット配置のシステムメッセージを保存する
    ///
    pub(crate) fn append_unit_placed_message(&self, game_uuid: Uuid, turn: &str, unit: Unit) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::UnitPlaced { unit };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// ユニット置換のシステムメッセージを保存する
    ///
    pub(crate) fn append_unit_replaced_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        old_unit: Unit,
        new_unit: Unit,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::UnitReplaced { old_unit, new_unit };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    ///
    /// ユニット除去のシステムメッセージを保存する
    ///
    pub(crate) fn append_unit_removed_message(&self, game_uuid: Uuid, turn: &str, unit: Unit) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::UnitRemoved { unit };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message, &catalog)?;
        Ok(())
    }

    /// メッセージ DB 接続を開く
    fn open_connection(&self) -> Result<Connection, RepositoryError> {
        Connection::open(&self.message_database_path)
            .map_err(|error| RepositoryError::Unavailable(format!("open message sqlite: {}", error)))
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

    /// システムメッセージを挿入する
    fn insert_system_message(
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
            .map_err(|error| RepositoryError::Unavailable(format!("insert system message: {}", error)))?;

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
            SystemNoticeCatalog::DrawProposed => "draw_proposed",
            SystemNoticeCatalog::OwnerAbsent => "owner_absent",
            SystemNoticeCatalog::DrawRescinded => "draw_rescinded",
            SystemNoticeCatalog::Solo { .. } => "solo",
            SystemNoticeCatalog::Draw => "draw",
            SystemNoticeCatalog::Closed => "closed",
            SystemNoticeCatalog::UnitPlaced { .. } => "unit_placed",
            SystemNoticeCatalog::UnitReplaced { .. } => "unit_replaced",
            SystemNoticeCatalog::UnitRemoved { .. } => "unit_removed",
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
            SystemNoticeCatalog::Solo { power } => {
                json!({ "power": power.symbol() })
            }
            SystemNoticeCatalog::UnitPlaced { unit } => json!({
                "power": unit.power.symbol(),
                "unit_label": unit.label(),
            }),
            SystemNoticeCatalog::UnitReplaced { old_unit, new_unit } => json!({
                "old_power": old_unit.power.symbol(),
                "old_unit_label": old_unit.label(),
                "new_power": new_unit.power.symbol(),
                "new_unit_label": new_unit.label(),
            }),
            SystemNoticeCatalog::UnitRemoved { unit } => json!({
                "power": unit.power.symbol(),
                "unit_label": unit.label(),
            }),
            SystemNoticeCatalog::Ready
            | SystemNoticeCatalog::Aborted
            | SystemNoticeCatalog::DrawProposed
            | SystemNoticeCatalog::OwnerAbsent
            | SystemNoticeCatalog::DrawRescinded
            | SystemNoticeCatalog::Draw
            | SystemNoticeCatalog::Closed => json!({}),
        }
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_repository() -> (SqliteMessageRepository, String) {
        let path = std::env::temp_dir().join(format!("moe3-message-test-{}", Uuid::now_v7()));
        let base = path.to_string_lossy().to_string();
        let messages_path = format!("{}.messages.db", base);
        (SqliteMessageRepository::new(&messages_path), messages_path)
    }

    fn sample_user() -> User {
        User {
            uuid: Uuid::now_v7(),
            discord_user_id: "1002".to_string(),
            username: "joiner".to_string(),
            global_name: None,
            avatar_hash: None,
            avatar_url: None,
        }
    }

    #[test]
    fn append_player_joined_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();
        let user = sample_user();

        repository
            .append_player_joined_message(game_uuid, &user)
            .expect("append player joined message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (sender_power, turn, context, kind, catalog): (Option<String>, String, String, String, String) = connection
            .query_row(
                "SELECT sender_power, turn, context, kind, system_notice_catalog FROM messages LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "ready");
        assert_eq!(kind, "system");
        assert_eq!(catalog, "player_joined");
        assert_eq!(
            context,
            format!("{} ({}) が参加を表明しました。", user.username, user.discord_user_id)
        );
    }

    #[test]
    fn append_draw_proposed_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();

        repository
            .append_draw_proposed_message(game_uuid, "1901s")
            .expect("append draw proposed message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (sender_power, turn, context, kind, catalog): (Option<String>, String, String, String, String) = connection
            .query_row(
                "SELECT sender_power, turn, context, kind, system_notice_catalog FROM messages LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(catalog, "draw_proposed");
        assert_eq!(context, "卓主によって講和が宣言されました。");
    }

    #[test]
    fn append_draw_rescinded_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();

        repository
            .append_draw_rescinded_message(game_uuid, "1901s")
            .expect("append draw rescinded message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (sender_power, turn, context, kind, catalog): (Option<String>, String, String, String, String) = connection
            .query_row(
                "SELECT sender_power, turn, context, kind, system_notice_catalog FROM messages LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(catalog, "draw_rescinded");
        assert_eq!(context, "卓主によって講和が撤回されました。");
    }
}
