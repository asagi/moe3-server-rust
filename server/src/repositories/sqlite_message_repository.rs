// ============================================================================
// imports
// ============================================================================

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::params;
use tokio::sync::broadcast;
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
    pub sender_kind: MessageSenderKind,
    pub sender_power: Option<Power>,
    pub sender_user_uuid: Option<Uuid>,
    pub turn: String,
    pub context: String,
    pub kind: String,
    pub recipients: Vec<Power>,
    pub created_at: String,
}

///
/// messages テーブルの 1 行分の生データの構造体
///
struct MessageRow {
    message_uuid: String,
    sender_power: Option<String>,
    sender_kind: Option<String>,
    sender_user_uuid: Option<String>,
    turn: String,
    context: String,
    kind: String,
    recipients: Option<String>,
    created_at: String,
}

///
/// メッセージ送信者種別の列挙体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageSenderKind {
    /// 担当国としての発言
    Power,
    /// 担当国を持たないユーザーとしての発言（開戦前夜・感想戦）
    User,
    /// 管理人発言
    Admin,
    /// システムメッセージ
    System,
}

/// メッセージ送信者種別の列挙体の実装
impl MessageSenderKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Power => "power",
            Self::User => "user",
            Self::Admin => "admin",
            Self::System => "system",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "power" => Some(Self::Power),
            "user" => Some(Self::User),
            "admin" => Some(Self::Admin),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

///
/// ユーザー発言の登録内容の構造体
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewChatMessage {
    pub sender_kind: MessageSenderKind,
    pub sender_power: Option<Power>,
    pub sender_user_uuid: Uuid,
    pub turn: String,
    pub context: String,
    pub kind: MessageKind,
}

///
/// SQLite 用のメッセージリポジトリ構造体
///
/// メッセージを保存するたびに、対象卓の UUID を `notifier` で通知する（SSE 配信用）。
///
#[derive(Debug, Clone)]
pub(crate) struct SqliteMessageRepository {
    message_database_path: String,
    notifier: broadcast::Sender<Uuid>,
}

/// メッセージ保存通知チャネルの容量
const NOTIFIER_CAPACITY: usize = 1024;

const READY_TURN: &str = "ready";
const DEBRIEF_TURN: &str = "debrief";

/// SQLite 用のメッセージリポジトリ構造体の実装
impl SqliteMessageRepository {
    ///
    /// new 関数
    ///
    pub(crate) fn new(messages_database_path: &str) -> Self {
        let (notifier, _) = broadcast::channel(NOTIFIER_CAPACITY);
        Self {
            message_database_path: messages_database_path.to_string(),
            notifier,
        }
    }

    ///
    /// メッセージ保存通知を購読する
    ///
    /// 受信する値はメッセージが保存された（またはフェイズが進行した）卓の UUID。
    ///
    pub(crate) fn subscribe(&self) -> broadcast::Receiver<Uuid> {
        self.notifier.subscribe()
    }

    /// メッセージ保存を購読者に通知する（購読者がいない場合は何もしない）
    fn notify(&self, game_uuid: Uuid) {
        let _ = self.notifier.send(game_uuid);
    }

    ///
    /// メッセージの保存を伴わない卓の更新（フェイズ進行など）を購読者に通知する
    ///
    /// SSE 配信はこの通知を受けて卓の状態を読み直し、season / phase イベントを送る。
    ///
    pub(crate) fn notify_game_updated(&self, game_uuid: Uuid) {
        self.notify(game_uuid);
    }

    ///
    /// ユーザー発言（公式声明・機密書簡・独り言・管理人発言）を保存する
    ///
    pub(crate) fn append_chat_message(&self, game_uuid: Uuid, message: NewChatMessage) -> Result<MessageRecord, RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;

        let now = Utc::now().to_rfc3339();
        let message_uuid = Uuid::now_v7();
        let sender_power = message.sender_power.map(Self::power_to_symbol);
        let kind = Self::kind_to_text(&message.kind);
        let recipients = match &message.kind {
            MessageKind::Confidential(letter) => letter.recipients.clone(),
            _ => Vec::new(),
        };
        let recipients_text = if recipients.is_empty() {
            None
        } else {
            Some(recipients.iter().map(|power| power.symbol()).collect::<Vec<_>>().join(","))
        };

        connection
            .execute(
                r#"
                INSERT INTO messages (
                    message_uuid,
                    game_uuid,
                    sender_power,
                    sender_kind,
                    sender_user_uuid,
                    turn,
                    context,
                    kind,
                    recipients,
                    created_at,
                    is_deleted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)
                "#,
                params![
                    message_uuid.to_string(),
                    game_uuid.to_string(),
                    sender_power,
                    message.sender_kind.as_str(),
                    message.sender_user_uuid.to_string(),
                    message.turn,
                    message.context,
                    kind,
                    recipients_text,
                    now,
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("insert chat message: {}", error)))?;

        self.notify(game_uuid);

        Ok(MessageRecord {
            message_uuid,
            sender_kind: message.sender_kind,
            sender_power: message.sender_power,
            sender_user_uuid: Some(message.sender_user_uuid),
            turn: message.turn,
            context: message.context,
            kind: kind.to_string(),
            recipients,
            created_at: now,
        })
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
                sender_kind TEXT,
                sender_user_uuid TEXT,
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

        // 既存 DB に送信者種別・送信ユーザーの列が無い場合は追加する
        for column in ["sender_kind", "sender_user_uuid"] {
            let exists: bool = connection
                .query_row(
                    "SELECT COUNT(*) > 0 FROM pragma_table_info('messages') WHERE name = ?1",
                    params![column],
                    |row| row.get(0),
                )
                .map_err(|error| RepositoryError::Unavailable(format!("inspect messages table: {}", error)))?;
            if !exists {
                connection
                    .execute(&format!("ALTER TABLE messages ADD COLUMN {} TEXT", column), [])
                    .map_err(|error| RepositoryError::Unavailable(format!("add messages.{} column: {}", column, error)))?;
            }
        }

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
                    sender_kind,
                    turn,
                    context,
                    kind,
                    created_at,
                    is_deleted
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)
                "#,
                params![
                    message_uuid.to_string(),
                    game_uuid.to_string(),
                    sender_power,
                    MessageSenderKind::System.as_str(),
                    message.turn,
                    message.context,
                    kind,
                    now,
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("insert system message: {}", error)))?;

        self.notify(game_uuid);

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
                SELECT message_uuid, sender_power, sender_kind, sender_user_uuid, turn, context, kind, recipients, created_at
                FROM messages
                WHERE game_uuid = ?1
                  AND turn = ?2
                  AND is_deleted = 0
                  AND (?3 IS NULL OR message_uuid > ?3)
                ORDER BY message_uuid ASC
                "#,
            )
            .map_err(|e| RepositoryError::Unavailable(format!("prepare find_by_game_and_season: {}", e)))?;

        let rows = stmt
            .query_map(params![game_uuid.to_string(), season, after_str], Self::read_message_row)
            .map_err(|e| RepositoryError::Unavailable(format!("query find_by_game_and_season: {}", e)))?;

        Self::collect_message_records(rows, "find_by_game_and_season")
    }

    ///
    /// 指定した卓の、指定メッセージより後のメッセージ一覧をシーズンを問わず返す
    ///
    /// `after_uuid` が `None` の場合は全件を返す。
    ///
    pub(crate) fn find_by_game_after(
        &self,
        game_uuid: Uuid,
        after_uuid: Option<Uuid>,
    ) -> Result<Vec<MessageRecord>, RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;

        let after_str = after_uuid.map(|u| u.to_string());

        let mut stmt = connection
            .prepare(
                r#"
                SELECT message_uuid, sender_power, sender_kind, sender_user_uuid, turn, context, kind, recipients, created_at
                FROM messages
                WHERE game_uuid = ?1
                  AND is_deleted = 0
                  AND (?2 IS NULL OR message_uuid > ?2)
                ORDER BY message_uuid ASC
                "#,
            )
            .map_err(|e| RepositoryError::Unavailable(format!("prepare find_by_game_after: {}", e)))?;

        let rows = stmt
            .query_map(params![game_uuid.to_string(), after_str], Self::read_message_row)
            .map_err(|e| RepositoryError::Unavailable(format!("query find_by_game_after: {}", e)))?;

        Self::collect_message_records(rows, "find_by_game_after")
    }

    ///
    /// 指定した卓の最新メッセージの UUID を返す（削除済みを含む）
    ///
    pub(crate) fn find_latest_message_uuid(&self, game_uuid: Uuid) -> Result<Option<Uuid>, RepositoryError> {
        let connection = self.open_connection()?;
        self.init_schema(&connection)?;

        let latest: Option<String> = connection
            .query_row(
                "SELECT MAX(message_uuid) FROM messages WHERE game_uuid = ?1",
                params![game_uuid.to_string()],
                |row| row.get(0),
            )
            .map_err(|e| RepositoryError::Unavailable(format!("query find_latest_message_uuid: {}", e)))?;

        latest
            .map(|value| Uuid::parse_str(&value).map_err(|e| RepositoryError::Unavailable(format!("parse message_uuid: {}", e))))
            .transpose()
    }

    /// メッセージ行を読み取る
    fn read_message_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRow> {
        Ok(MessageRow {
            message_uuid: row.get(0)?,
            sender_power: row.get(1)?,
            sender_kind: row.get(2)?,
            sender_user_uuid: row.get(3)?,
            turn: row.get(4)?,
            context: row.get(5)?,
            kind: row.get(6)?,
            recipients: row.get(7)?,
            created_at: row.get(8)?,
        })
    }

    /// メッセージ行をメッセージクエリ結果に変換して収集する
    fn collect_message_records<I>(rows: I, label: &str) -> Result<Vec<MessageRecord>, RepositoryError>
    where
        I: Iterator<Item = rusqlite::Result<MessageRow>>,
    {
        let mut result = Vec::new();
        for row in rows {
            let row = row.map_err(|e| RepositoryError::Unavailable(format!("row {}: {}", label, e)))?;
            result.push(Self::message_row_to_record(row)?);
        }
        Ok(result)
    }

    /// メッセージ行をメッセージクエリ結果に変換する
    fn message_row_to_record(row: MessageRow) -> Result<MessageRecord, RepositoryError> {
        let message_uuid =
            Uuid::parse_str(&row.message_uuid).map_err(|e| RepositoryError::Unavailable(format!("parse message_uuid: {}", e)))?;

        let sender_power = row.sender_power.as_deref().and_then(Power::from_symbol);

        let sender_user_uuid = row
            .sender_user_uuid
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|e| RepositoryError::Unavailable(format!("parse sender_user_uuid: {}", e)))?;

        // 送信者種別の列が追加される前のメッセージは、種別と送信国から推定する
        let sender_kind = if row.kind == "system" {
            MessageSenderKind::System
        } else {
            row.sender_kind
                .as_deref()
                .and_then(MessageSenderKind::from_str)
                .unwrap_or(if sender_power.is_some() {
                    MessageSenderKind::Power
                } else {
                    MessageSenderKind::User
                })
        };

        let recipients = row
            .recipients
            .as_deref()
            .map(|s| s.split(',').filter_map(Power::from_symbol).collect())
            .unwrap_or_default();

        Ok(MessageRecord {
            message_uuid,
            sender_kind,
            sender_power,
            sender_user_uuid,
            turn: row.turn,
            context: row.context,
            kind: row.kind,
            recipients,
            created_at: row.created_at,
        })
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
        assert_eq!(context, format!("@{} が参加を表明しました。", user.username));
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

    // ============================================================================
    // find_by_game_and_season tests
    // ============================================================================

    fn insert_message_row(
        connection: &Connection,
        game_uuid: Uuid,
        turn: &str,
        kind: &str,
        sender_power: Option<&str>,
        recipients: Option<&str>,
        is_deleted: bool,
    ) -> Uuid {
        let message_uuid = Uuid::now_v7();
        let now = chrono::Utc::now().to_rfc3339();
        connection
            .execute(
                r#"INSERT INTO messages
                   (message_uuid, game_uuid, sender_power, turn, context, kind, recipients, created_at, is_deleted)
                   VALUES (?1, ?2, ?3, ?4, 'test context', ?5, ?6, ?7, ?8)"#,
                rusqlite::params![
                    message_uuid.to_string(),
                    game_uuid.to_string(),
                    sender_power,
                    turn,
                    kind,
                    recipients,
                    now,
                    if is_deleted { 1 } else { 0 },
                ],
            )
            .expect("insert test message");
        message_uuid
    }

    fn setup_db(db_path: &str) -> Connection {
        let conn = Connection::open(db_path).expect("open db");
        conn.execute_batch(
            r#"
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
            "#,
        )
        .expect("create table");
        conn
    }

    #[test]
    fn find_by_game_and_season_returns_matching_season() {
        let (repository, db_path) = new_test_repository();
        let conn = setup_db(&db_path);
        let game_uuid = Uuid::now_v7();

        insert_message_row(&conn, game_uuid, "1901s", "public", Some("f"), None, false);
        insert_message_row(&conn, game_uuid, "1901f", "public", Some("e"), None, false);

        let result = repository
            .find_by_game_and_season(game_uuid, "1901s", None)
            .expect("should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].turn, "1901s");
    }

    #[test]
    fn find_by_game_and_season_excludes_deleted_rows() {
        let (repository, db_path) = new_test_repository();
        let conn = setup_db(&db_path);
        let game_uuid = Uuid::now_v7();

        insert_message_row(&conn, game_uuid, "1901s", "public", Some("f"), None, false);
        insert_message_row(&conn, game_uuid, "1901s", "public", Some("e"), None, true);

        let result = repository
            .find_by_game_and_season(game_uuid, "1901s", None)
            .expect("should succeed");

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn find_by_game_and_season_after_uuid_filters_correctly() {
        let (repository, db_path) = new_test_repository();
        let conn = setup_db(&db_path);
        let game_uuid = Uuid::now_v7();

        let uuid1 = insert_message_row(&conn, game_uuid, "1901s", "public", Some("f"), None, false);
        // uuid2 は uuid1 より後のタイムスタンプになるよう少し待つ代わりに、
        // UUID v7 は時刻単調増加なので now_v7() の連続呼び出しで十分大きい
        let uuid2 = insert_message_row(&conn, game_uuid, "1901s", "system", None, None, false);

        // uuid1 以降 (uuid1 より大きい) → uuid2 のみ返る
        let result = repository
            .find_by_game_and_season(game_uuid, "1901s", Some(uuid1))
            .expect("should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message_uuid, uuid2);
    }

    #[test]
    fn find_by_game_and_season_parses_confidential_recipients() {
        let (repository, db_path) = new_test_repository();
        let conn = setup_db(&db_path);
        let game_uuid = Uuid::now_v7();

        insert_message_row(&conn, game_uuid, "1901s", "confidential", Some("f"), Some("e,g"), false);

        let result = repository
            .find_by_game_and_season(game_uuid, "1901s", None)
            .expect("should succeed");

        assert_eq!(result.len(), 1);
        let msg = &result[0];
        assert_eq!(msg.sender_power, Some(Power::France));
        assert_eq!(msg.recipients, vec![Power::England, Power::Germany]);
    }

    #[test]
    fn find_by_game_and_season_migrates_legacy_schema_and_infers_sender_kind() {
        let (repository, db_path) = new_test_repository();
        // 送信者種別の列が無い旧スキーマに、旧形式の行を入れておく
        let conn = setup_db(&db_path);
        let game_uuid = Uuid::now_v7();
        insert_message_row(&conn, game_uuid, "1901s", "public", Some("f"), None, false);
        insert_message_row(&conn, game_uuid, "1901s", "system", None, None, false);

        let result = repository
            .find_by_game_and_season(game_uuid, "1901s", None)
            .expect("should succeed");

        assert_eq!(result[0].sender_kind, MessageSenderKind::Power);
        assert_eq!(result[0].sender_user_uuid, None);
        assert_eq!(result[1].sender_kind, MessageSenderKind::System);
    }

    #[test]
    fn append_chat_message_persists_sender_and_recipients() {
        let (repository, _db_path) = new_test_repository();
        let game_uuid = Uuid::now_v7();
        let user_uuid = Uuid::now_v7();

        let record = repository
            .append_chat_message(
                game_uuid,
                NewChatMessage {
                    sender_kind: MessageSenderKind::Power,
                    sender_power: Some(Power::France),
                    sender_user_uuid: user_uuid,
                    turn: "1901s".to_string(),
                    context: "密談".to_string(),
                    kind: MessageKind::confidential(vec![Power::England, Power::Germany]),
                },
            )
            .expect("append should succeed");

        let result = repository.find_by_game_after(game_uuid, None).expect("should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message_uuid, record.message_uuid);
        assert_eq!(result[0].sender_kind, MessageSenderKind::Power);
        assert_eq!(result[0].sender_user_uuid, Some(user_uuid));
        assert_eq!(result[0].kind, "confidential");
        assert_eq!(result[0].recipients, vec![Power::England, Power::Germany]);
        assert_eq!(
            repository.find_latest_message_uuid(game_uuid).expect("latest"),
            Some(record.message_uuid)
        );
        assert_eq!(repository.find_latest_message_uuid(Uuid::now_v7()).expect("latest"), None);
    }

    #[test]
    fn find_by_game_and_season_returns_empty_for_different_game() {
        let (repository, db_path) = new_test_repository();
        let conn = setup_db(&db_path);
        let game_uuid = Uuid::now_v7();
        let other_game = Uuid::now_v7();

        insert_message_row(&conn, other_game, "1901s", "public", Some("f"), None, false);

        let result = repository
            .find_by_game_and_season(game_uuid, "1901s", None)
            .expect("should succeed");

        assert!(result.is_empty());
    }
}
