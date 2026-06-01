// ============================================================================
// imports
// ============================================================================

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::params;
use uuid::Uuid;

use super::Message;
use super::MessageKind;
use super::Power;
use super::Province;
use super::RepositoryError;
use super::SystemNotice;
use super::SystemNoticeCatalog;
use super::Unit;
use super::User;

// ============================================================================
// definitions
// ============================================================================

///
/// メッセージクエリ結果の構造体
///
#[derive(Debug, Clone)]
pub(crate) struct MessageRecord {
    pub message_uuid: Uuid,
    pub sender_power: Option<Power>,
    pub turn: String,
    pub context: String,
    pub kind: String,
    pub recipients: Vec<Power>,
    pub created_at: String,
}

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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
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

        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    ///
    /// プレイヤーによる即時進行合意のシステムメッセージを保存する
    ///
    pub(crate) fn append_progress_consented_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        power: Power,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::ProgressConsented { power };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    ///
    /// プレイヤーによる即時進行合意撤回のシステムメッセージを保存する
    ///
    pub(crate) fn append_progress_consensus_rescinded_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        power: Power,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::ProgressConsensusRescinded { power };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    ///
    /// 全生存国の即時進行合意成立メッセージを保存する
    ///
    pub(crate) fn append_progress_consensus_reached_message(&self, game_uuid: Uuid, turn: &str) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::ProgressConsensusReached;
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
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
        self.insert_system_message(&connection, game_uuid, &message)?;
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
        self.insert_system_message(&connection, game_uuid, &message)?;
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
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    ///
    /// 占領登録のシステムメッセージを保存する
    ///
    pub(crate) fn append_territory_set_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        province: Province,
        power: Power,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::TerritorySet { province, power };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    ///
    /// 占領置換のシステムメッセージを保存する
    ///
    pub(crate) fn append_territory_replaced_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        province: Province,
        old_power: Power,
        new_power: Power,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::TerritoryReplaced {
            province,
            old_power,
            new_power,
        };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    ///
    /// 占領解放のシステムメッセージを保存する
    ///
    pub(crate) fn append_territory_released_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        province: Province,
        old_power: Power,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::TerritoryReleased { province, old_power };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
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
                recipients TEXT,
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
    fn insert_system_message(&self, connection: &Connection, game_uuid: Uuid, message: &Message) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();
        let message_uuid = Uuid::now_v7();

        let sender_power = message.sender.map(Self::power_to_symbol);
        let kind = Self::kind_to_text(&message.kind);

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
                    created_at,
                    is_deleted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)
                "#,
                params![
                    message_uuid.to_string(),
                    game_uuid.to_string(),
                    sender_power,
                    message.turn,
                    message.context,
                    kind,
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

    ///
    /// 指定した卓・シーズンのメッセージ一覧を返す
    ///
    pub(crate) fn find_by_game_and_season(
        &self,
        game_uuid: Uuid,
        season: &str,
        after_uuid: Option<Uuid>,
    ) -> Result<Vec<MessageRecord>, RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;

        let after_str = after_uuid.map(|u| u.to_string());

        let mut stmt = connection
            .prepare(
                r#"
                SELECT message_uuid, sender_power, turn, context, kind, recipients, created_at
                FROM messages
                WHERE game_uuid = ?1
                  AND turn = ?2
                  AND is_deleted = 0
                  AND (?3 IS NULL OR message_uuid > ?3)
                ORDER BY message_uuid ASC
                "#,
            )
            .map_err(|e| RepositoryError::Unavailable(format!("prepare find_by_game_and_season: {}", e)))?;

        let records = stmt
            .query_map(params![game_uuid.to_string(), season, after_str], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|e| RepositoryError::Unavailable(format!("query find_by_game_and_season: {}", e)))?;

        let mut result = Vec::new();
        for row in records {
            let (message_uuid_str, sender_power_str, turn, context, kind, recipients_str, created_at) =
                row.map_err(|e| RepositoryError::Unavailable(format!("row find_by_game_and_season: {}", e)))?;

            let message_uuid = Uuid::parse_str(&message_uuid_str)
                .map_err(|e| RepositoryError::Unavailable(format!("parse message_uuid: {}", e)))?;

            let sender_power = sender_power_str.as_deref().and_then(Power::from_symbol);

            let recipients = recipients_str
                .as_deref()
                .map(|s| s.split(',').filter_map(Power::from_symbol).collect())
                .unwrap_or_default();

            result.push(MessageRecord {
                message_uuid,
                sender_power,
                turn,
                context,
                kind,
                recipients,
                created_at,
            });
        }

        Ok(result)
    }

    pub(crate) fn append_progress_mode_changed_message(&self, game_uuid: Uuid, turn: &str) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::ProgressModeChanged;
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
    }

    pub(crate) fn append_next_update_at_changed_message(
        &self,
        game_uuid: Uuid,
        turn: &str,
        next_update_at: &str,
    ) -> Result<(), RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;
        let catalog = SystemNoticeCatalog::NextUpdateAtChanged {
            next_update_at: next_update_at.to_string(),
        };
        let message = Message {
            sender: None,
            turn: turn.to_string(),
            context: catalog.to_string(),
            kind: MessageKind::System(SystemNotice {}),
        };
        self.insert_system_message(&connection, game_uuid, &message)?;
        Ok(())
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
        let (sender_power, turn, context, kind): (Option<String>, String, String, String) = connection
            .query_row("SELECT sender_power, turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "ready");
        assert_eq!(kind, "system");
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
        let (sender_power, turn, context, kind): (Option<String>, String, String, String) = connection
            .query_row("SELECT sender_power, turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
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
        let (sender_power, turn, context, kind): (Option<String>, String, String, String) = connection
            .query_row("SELECT sender_power, turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "卓主によって講和が撤回されました。");
    }

    #[test]
    fn append_territory_set_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();
        let province = Province::from_code("par").expect("par should be valid");

        repository
            .append_territory_set_message(game_uuid, "1901s", province, Power::France)
            .expect("append territory set message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (turn, context, kind): (String, String, String) = connection
            .query_row("SELECT turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("read inserted message");

        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "パリ の保有国が France に変更されました。");
    }

    #[test]
    fn append_territory_replaced_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();
        let province = Province::from_code("par").expect("par should be valid");

        repository
            .append_territory_replaced_message(game_uuid, "1901s", province, Power::France, Power::England)
            .expect("append territory replaced message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (turn, context, kind): (String, String, String) = connection
            .query_row("SELECT turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("read inserted message");

        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "パリ の保有国が France から England に変更されました。");
    }

    #[test]
    fn append_territory_released_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();
        let province = Province::from_code("par").expect("par should be valid");

        repository
            .append_territory_released_message(game_uuid, "1901s", province, Power::France)
            .expect("append territory released message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (turn, context, kind): (String, String, String) = connection
            .query_row("SELECT turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("read inserted message");

        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "France が保有していた パリ が解放されました。");
    }

    #[test]
    fn append_progress_mode_changed_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();

        repository
            .append_progress_mode_changed_message(game_uuid, "1901s")
            .expect("append progress mode changed message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (sender_power, turn, context, kind): (Option<String>, String, String, String) = connection
            .query_row("SELECT sender_power, turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("read inserted message");

        assert_eq!(sender_power, None);
        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "進行モードが定時進行から合意進行に変更されました。");
    }

    #[test]
    fn append_progress_consented_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();

        repository
            .append_progress_consented_message(game_uuid, "1901s", Power::France)
            .expect("append progress consented message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (turn, context, kind): (String, String, String) = connection
            .query_row("SELECT turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("read inserted message");

        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "France が即時進行に合意しました。");
    }

    #[test]
    fn append_progress_consensus_rescinded_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();

        repository
            .append_progress_consensus_rescinded_message(game_uuid, "1901s", Power::England)
            .expect("append progress consensus rescinded message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (turn, context, kind): (String, String, String) = connection
            .query_row("SELECT turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("read inserted message");

        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(context, "England が即時進行への合意を撤回しました。");
    }

    #[test]
    fn append_progress_consensus_reached_message_persists_system_message() {
        let (repository, db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();

        repository
            .append_progress_consensus_reached_message(game_uuid, "1901s")
            .expect("append progress consensus reached message should succeed");

        let connection = Connection::open(&db_path).expect("open message db");
        let (turn, context, kind): (String, String, String) = connection
            .query_row("SELECT turn, context, kind FROM messages LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("read inserted message");

        assert_eq!(turn, "1901s");
        assert_eq!(kind, "system");
        assert_eq!(
            context,
            "全ての生存国の合意を確認しました。メインフェイズをただちに終了します。"
        );
    }
}
