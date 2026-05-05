// ============================================================================
// imports
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Utc;
use rusqlite::Connection;
use rusqlite::Transaction;
use rusqlite::params;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::DurationType;
use super::FaceType;
use super::Game;
use super::GameRepository;
use super::GameStatus;
use super::NewGame;
use super::Order;
use super::OrderKind;
use super::OrderStatus;
use super::Phase;
use super::PhaseKind;
use super::Player;
use super::Power;
use super::ProgressMode;
use super::Province;
use super::Regulation;
use super::RepositoryError;
use super::Territory;
use super::Unit;

// ============================================================================
// definitions
// ============================================================================

///
/// SQLite 用の卓リポジトリ構造体
///
#[derive(Clone)]
pub(crate) struct SqliteGameRepository {
    connection: Arc<Mutex<Connection>>,
}

/// SQLite 用の卓リポジトリ構造体の実装
impl SqliteGameRepository {
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
    /// テスト用のインメモリリポジトリを生成する
    ///
    #[cfg(test)]
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
            CREATE TABLE IF NOT EXISTS games (
                uuid TEXT PRIMARY KEY,
                game_number INTEGER,
                keyword TEXT,
                regulation_face_type INTEGER NOT NULL,
                regulation_progress_mode INTEGER NOT NULL,
                regulation_duration_type INTEGER NOT NULL,
                regulation_start_date TEXT NOT NULL,
                regulation_first_period_hour INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'preparing',
                is_draw INTEGER NOT NULL DEFAULT 0,
                is_solo INTEGER NOT NULL DEFAULT 0,
                next_update TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS game_players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_uuid TEXT NOT NULL,
                user_uuid TEXT NOT NULL,
                power INTEGER,
                is_accepting_draw INTEGER NOT NULL,
                is_owner INTEGER NOT NULL,
                requested_power INTEGER
            );

            CREATE TABLE IF NOT EXISTS game_phases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_uuid TEXT NOT NULL,
                phase_index INTEGER NOT NULL,
                phase_year INTEGER NOT NULL,
                phase_kind TEXT NOT NULL,
                units TEXT NOT NULL,
                territories TEXT NOT NULL,
                standoff_codes TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(game_uuid, phase_index)
            );

            CREATE TABLE IF NOT EXISTS game_phase_orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                phase_id INTEGER NOT NULL,
                order_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .execute_batch(sql)
            .map_err(|error| RepositoryError::Unavailable(format!("create game tables: {}", error)))?;

        Ok(())
    }

    /// ゲームステータスを文字列に変換する
    fn status_to_str(status: GameStatus) -> &'static str {
        match status {
            GameStatus::Preparing => "preparing",
            GameStatus::Ready => "ready",
            GameStatus::InProgress => "in_progress",
            GameStatus::Finished => "finished",
            GameStatus::Aborted => "aborted",
            GameStatus::Closed => "closed",
        }
    }

    /// 文字列をゲームステータスに変換する
    fn status_from_str(text: &str) -> Result<GameStatus, RepositoryError> {
        match text {
            "preparing" => Ok(GameStatus::Preparing),
            "ready" => Ok(GameStatus::Ready),
            "in_progress" => Ok(GameStatus::InProgress),
            "finished" => Ok(GameStatus::Finished),
            "aborted" => Ok(GameStatus::Aborted),
            "closed" => Ok(GameStatus::Closed),
            _ => Err(RepositoryError::Unavailable(format!("unknown game status: {}", text))),
        }
    }

    /// Power を整数から変換する
    fn power_from_i32(value: i32) -> Result<Power, RepositoryError> {
        use strum::IntoEnumIterator;
        Power::iter()
            .find(|p| *p as i32 == value)
            .ok_or_else(|| RepositoryError::Unavailable(format!("invalid power value: {}", value)))
    }

    /// ゲームステータスを文字列に変換する
    fn serialize_status(status: GameStatus) -> String {
        Self::status_to_str(status).to_string()
    }

    /// フェイズの種類を文字列に変換する
    fn serialize_phase_kind(kind: &PhaseKind) -> Result<String, RepositoryError> {
        serde_json::to_string(kind).map_err(|error| RepositoryError::Unavailable(format!("serialize phase kind json: {}", error)))
    }

    /// 文字列をフェイズの種類に変換する
    fn deserialize_phase_kind(text: &str) -> Result<PhaseKind, RepositoryError> {
        serde_json::from_str(text)
            .map_err(|error| RepositoryError::Unavailable(format!("deserialize phase kind json: {}", error)))
    }

    /// ユニットの種類を文字列に変換する
    fn unit_kind_name(unit: &Unit) -> &'static str {
        if unit.is_fleet() { "fleet" } else { "army" }
    }

    /// ユニットを構築する
    fn build_unit(
        power: Power,
        location: Province,
        kind_name: &str,
        dislodged_from: Option<Province>,
        dislodged: bool,
    ) -> Result<Unit, RepositoryError> {
        let mut unit = match kind_name {
            "army" => Unit::new_army(power, location),
            "fleet" => Unit::new_fleet(power, location),
            _ => return Err(RepositoryError::Unavailable(format!("invalid unit kind: {}", kind_name))),
        };

        if dislodged {
            unit = unit.set_dislodged_from(dislodged_from);
        } else if dislodged_from.is_some() {
            return Err(RepositoryError::Unavailable(
                "unit has dislodged_from while dislodged=false".to_string(),
            ));
        }

        Ok(unit)
    }

    /// ユニットをペイロードに変換する
    fn unit_to_payload(unit: &Unit) -> UnitPayload {
        UnitPayload {
            power: unit.power,
            unit_kind: Self::unit_kind_name(unit).to_string(),
            location: unit.location.code_with_coast().to_string(),
            dislodged: unit.dislodged,
            dislodged_from: unit.dislodged_from.map(|province| province.code_with_coast().to_string()),
        }
    }

    /// ペイロードをユニットに変換する
    fn unit_from_payload(payload: &UnitPayload) -> Result<Unit, RepositoryError> {
        Self::build_unit(
            payload.power,
            Province::from_code(&payload.location)
                .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", payload.location)))?,
            &payload.unit_kind,
            match &payload.dislodged_from {
                Some(value) => Some(
                    Province::from_code(value)
                        .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", value)))?,
                ),
                None => None,
            },
            payload.dislodged,
        )
    }

    /// ユニットのリストを JSON 文字列に変換する
    fn serialize_units(units: &[Unit]) -> Result<String, RepositoryError> {
        let payloads = units.iter().map(Self::unit_to_payload).collect::<Vec<_>>();
        serde_json::to_string(&payloads).map_err(|error| RepositoryError::Unavailable(format!("serialize units json: {}", error)))
    }

    /// JSON 文字列をユニットのリストに変換する
    fn deserialize_units(text: &str) -> Result<Vec<Unit>, RepositoryError> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let payloads: Vec<UnitPayload> = serde_json::from_str(text)
            .map_err(|error| RepositoryError::Unavailable(format!("deserialize units json: {}", error)))?;

        payloads.iter().map(Self::unit_from_payload).collect::<Result<Vec<_>, _>>()
    }

    /// 命令を JSON 文字列に変換する
    fn serialize_order(order: &Order) -> Result<String, RepositoryError> {
        let kind = match order.kind {
            OrderKind::Hold(_) => OrderKindPayload::Hold,
            OrderKind::Move(move_order) => OrderKindPayload::Move {
                dest: move_order.dest.code_with_coast().to_string(),
                via_convoy: move_order.via_convoy,
            },
            OrderKind::Support(support_order) => OrderKindPayload::Support {
                target_unit: Self::unit_to_payload(&support_order.target_unit),
                target_dest: support_order
                    .target_dest
                    .map(|province| province.code_with_coast().to_string()),
            },
            OrderKind::Convoy(convoy_order) => OrderKindPayload::Convoy {
                target_unit: Self::unit_to_payload(&convoy_order.target_unit),
                target_dest: convoy_order.target_dest.code_with_coast().to_string(),
            },
            OrderKind::Retreat(retreat_order) => OrderKindPayload::Retreat {
                dest: retreat_order.dest.code_with_coast().to_string(),
            },
            OrderKind::Build(_) => OrderKindPayload::Build,
            OrderKind::Disband(_) => OrderKindPayload::Disband,
        };

        let payload = OrderPayload {
            power: order.power,
            unit: Self::unit_to_payload(&order.unit),
            dislodged_from: order.dislodged_from.map(|province| province.code_with_coast().to_string()),
            status: order.status,
            kind,
        };

        serde_json::to_string(&payload).map_err(|error| RepositoryError::Unavailable(format!("serialize order json: {}", error)))
    }

    /// JSON 文字列を命令に変換する
    fn deserialize_order(text: &str) -> Result<Order, RepositoryError> {
        let payload: OrderPayload = serde_json::from_str(text)
            .map_err(|error| RepositoryError::Unavailable(format!("deserialize order json: {}", error)))?;

        let OrderPayload {
            power,
            unit,
            dislodged_from,
            status,
            kind,
        } = payload;

        let unit = Self::unit_from_payload(&unit)?;

        let mut order = match kind {
            OrderKindPayload::Hold => Order::new_hold(power, unit),
            OrderKindPayload::Move { dest, via_convoy } => {
                let mut move_order = Order::new_move(
                    power,
                    unit,
                    Province::from_code(&dest)
                        .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", dest)))?,
                );
                if via_convoy {
                    move_order = move_order.set_via_convoy();
                }
                move_order
            }
            OrderKindPayload::Support {
                target_unit,
                target_dest,
            } => Order::new_support(
                power,
                unit,
                Self::unit_from_payload(&target_unit)?,
                match target_dest {
                    Some(value) => Some(
                        Province::from_code(&value)
                            .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", value)))?,
                    ),
                    None => None,
                },
            ),
            OrderKindPayload::Convoy {
                target_unit,
                target_dest,
            } => Order::new_convoy(
                power,
                unit,
                Self::unit_from_payload(&target_unit)?,
                Province::from_code(&target_dest)
                    .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", target_dest)))?,
            ),
            OrderKindPayload::Retreat { dest } => Order::new_retreat(
                power,
                unit,
                Province::from_code(&dest)
                    .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", dest)))?,
            ),
            OrderKindPayload::Build => Order::new_build(power, unit),
            OrderKindPayload::Disband => Order::new_disband(power, unit),
        };

        order.dislodged_from = match dislodged_from {
            Some(value) => Some(
                Province::from_code(&value)
                    .ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", value)))?,
            ),
            None => None,
        };

        order.status = status;
        Ok(order)
    }

    /// 占領情報のリストを JSON 文字列に変換する
    fn serialize_territories(territories: &[Territory]) -> String {
        let payloads = territories
            .iter()
            .map(|t| TerritoryPayload {
                power: t.power,
                code: t.code_with_coast().to_string(),
            })
            .collect::<Vec<_>>();

        serde_json::to_string(&payloads)
            .map_err(|error| RepositoryError::Unavailable(format!("serialize territories json: {}", error)))
            .unwrap_or_default()
    }

    /// JSON 文字列を占領情報のリストに変換する
    fn deserialize_territories(text: &str) -> Result<Vec<Territory>, RepositoryError> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let payloads: Vec<TerritoryPayload> = serde_json::from_str(text)
            .map_err(|error| RepositoryError::Unavailable(format!("deserialize territories json: {}", error)))?;

        payloads
            .into_iter()
            .map(|p| {
                let power = p.power;
                Ok(Territory::new(power, &p.code))
            })
            .collect::<Result<Vec<_>, RepositoryError>>()
    }

    /// スタンドオフ発生地域のリストを JSON 文字列に変換する
    fn serialize_codes(codes: &[String]) -> String {
        serde_json::to_string(codes).unwrap_or_else(|_| codes.join(","))
    }

    /// JSON 文字列をスタンドオフ発生地域のリストに変換する
    fn deserialize_codes(text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        serde_json::from_str::<Vec<String>>(text)
            .unwrap_or_else(|_| text.split(',').map(|value| value.to_string()).collect::<Vec<_>>())
    }

    /// 次の更新予定日時を RFC3339 形式の文字列に変換する
    fn serialize_next_update(next_update: NaiveDateTime) -> String {
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(next_update, Utc).to_rfc3339()
    }

    /// RFC3339 形式の文字列を次の更新予定日時に変換する
    fn parse_next_update(text: &str) -> Result<NaiveDateTime, RepositoryError> {
        chrono::DateTime::parse_from_rfc3339(text)
            .map(|dt| dt.with_timezone(&Utc).naive_utc())
            .map_err(|error| RepositoryError::Unavailable(format!("parse next_update: {}", error)))
    }

    /// クエリを実行してゲームのリストをロードする
    fn load_games_by_query<P>(connection: &Connection, sql: &str, params: P) -> Result<Vec<Game>, RepositoryError>
    where
        P: rusqlite::Params,
    {
        struct GameRow {
            uuid: String,
            game_number: Option<i32>,
            keyword: Option<String>,
            face_type: i32,
            progress_mode: i32,
            duration_type: i32,
            start_date: String,
            first_period_hour: i32,
            status: String,
            is_draw: i32,
            is_solo: i32,
            next_update: Option<String>,
        }

        let game_rows: Vec<GameRow> = {
            let mut stmt = connection
                .prepare(sql)
                .map_err(|error| RepositoryError::Unavailable(format!("prepare load games: {}", error)))?;

            stmt.query_map(params, |row| {
                Ok(GameRow {
                    uuid: row.get(0)?,
                    game_number: row.get(1)?,
                    keyword: row.get(2)?,
                    face_type: row.get(3)?,
                    progress_mode: row.get(4)?,
                    duration_type: row.get(5)?,
                    start_date: row.get(6)?,
                    first_period_hour: row.get(7)?,
                    status: row.get(8)?,
                    is_draw: row.get(9)?,
                    is_solo: row.get(10)?,
                    next_update: row.get(11)?,
                })
            })
            .map_err(|error| RepositoryError::Unavailable(format!("query load games: {}", error)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| RepositoryError::Unavailable(format!("collect load game rows: {}", error)))?
        };

        let mut games = Vec::new();

        for row in game_rows {
            let regulation = Regulation::new(
                FaceType::try_from(row.face_type).map_err(|e| RepositoryError::Unavailable(e.to_string()))?,
                ProgressMode::try_from(row.progress_mode).map_err(|e| RepositoryError::Unavailable(e.to_string()))?,
                DurationType::try_from(row.duration_type).map_err(|e| RepositoryError::Unavailable(e.to_string()))?,
                NaiveDate::parse_from_str(&row.start_date, "%Y-%m-%d")
                    .map_err(|error| RepositoryError::Unavailable(format!("parse start_date: {}", error)))?,
                row.first_period_hour as u8,
            )
            .map_err(|e| RepositoryError::Unavailable(e.to_string()))?;

            let uuid = Uuid::parse_str(&row.uuid)
                .map_err(|error| RepositoryError::Unavailable(format!("parse game uuid: {}", error)))?;

            let players: Vec<Player> = {
                let mut stmt = connection
                    .prepare(
                        r#"
                        SELECT user_uuid, power, is_accepting_draw, is_owner, requested_power
                        FROM game_players
                        WHERE game_uuid = ?1
                        "#,
                    )
                    .map_err(|error| RepositoryError::Unavailable(format!("prepare load players: {}", error)))?;

                let raw_rows = stmt
                    .query_map(params![row.uuid], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, Option<i32>>(1)?,
                            r.get::<_, i32>(2)?,
                            r.get::<_, i32>(3)?,
                            r.get::<_, Option<i32>>(4)?,
                        ))
                    })
                    .map_err(|error| RepositoryError::Unavailable(format!("query players: {}", error)))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| RepositoryError::Unavailable(format!("collect player rows: {}", error)))?;

                raw_rows
                    .into_iter()
                    .map(
                        |(user_uuid_str, power_val, is_accepting_draw, is_owner, requested_power_val)| {
                            let user_uuid = Uuid::parse_str(&user_uuid_str)
                                .map_err(|error| RepositoryError::Unavailable(format!("parse player uuid: {}", error)))?;
                            let power = power_val.map(Self::power_from_i32).transpose()?;
                            let requested_power = requested_power_val.map(Self::power_from_i32).transpose()?;
                            Ok(Player {
                                user_uuid,
                                power,
                                is_accepting_draw: is_accepting_draw != 0,
                                is_owner: is_owner != 0,
                                requested_power,
                            })
                        },
                    )
                    .collect::<Result<Vec<_>, RepositoryError>>()?
            };

            struct PhaseRow {
                id: i64,
                phase_index: i32,
                phase_year: i32,
                phase_kind: String,
                units: String,
                territories: String,
                standoff_codes: String,
            }

            let phase_rows: Vec<PhaseRow> = {
                let mut stmt = connection
                    .prepare(
                        r#"
                        SELECT id, phase_index, phase_year, phase_kind, units, territories, standoff_codes
                        FROM game_phases
                        WHERE game_uuid = ?1
                        ORDER BY phase_index ASC
                        "#,
                    )
                    .map_err(|error| RepositoryError::Unavailable(format!("prepare load phases: {}", error)))?;

                stmt.query_map(params![row.uuid], |r| {
                    Ok(PhaseRow {
                        id: r.get(0)?,
                        phase_index: r.get(1)?,
                        phase_year: r.get(2)?,
                        phase_kind: r.get(3)?,
                        units: r.get(4)?,
                        territories: r.get(5)?,
                        standoff_codes: r.get(6)?,
                    })
                })
                .map_err(|error| RepositoryError::Unavailable(format!("query phases: {}", error)))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| RepositoryError::Unavailable(format!("collect phase rows: {}", error)))?
            };

            let mut phases = Vec::new();
            for phase_row in phase_rows {
                let orders: Vec<Order> = {
                    let mut stmt = connection
                        .prepare(
                            r#"
                            SELECT order_json
                            FROM game_phase_orders
                            WHERE phase_id = ?1
                            "#,
                        )
                        .map_err(|error| RepositoryError::Unavailable(format!("prepare load orders: {}", error)))?;

                    let order_jsons: Vec<String> = stmt
                        .query_map(params![phase_row.id], |r| r.get(0))
                        .map_err(|error| RepositoryError::Unavailable(format!("query orders: {}", error)))?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|error| RepositoryError::Unavailable(format!("collect order jsons: {}", error)))?;

                    order_jsons
                        .iter()
                        .map(|json| Self::deserialize_order(json))
                        .collect::<Result<Vec<_>, _>>()?
                };

                let phase = Phase {
                    game_number: None,
                    index: phase_row.phase_index,
                    year: phase_row.phase_year,
                    kind: Self::deserialize_phase_kind(&phase_row.phase_kind)?,
                    orders,
                    units: Self::deserialize_units(&phase_row.units)?,
                    territories: Self::deserialize_territories(&phase_row.territories)?,
                    standoff_codes: Self::deserialize_codes(&phase_row.standoff_codes),
                };
                phases.push(phase);
            }

            let next_update = row.next_update.as_deref().map(Self::parse_next_update).transpose()?;

            games.push(Game {
                uuid,
                game_number: row.game_number,
                keyword: row.keyword,
                regulation,
                players,
                phases,
                status: Self::status_from_str(&row.status)?,
                is_draw: row.is_draw != 0,
                is_solo: row.is_solo != 0,
                next_update_at: next_update,
            });
        }

        Ok(games)
    }

    /// ゲームを挿入するトランザクション内でフェイズの命令を挿入する
    fn insert_phase_orders(
        transaction: &Transaction<'_>,
        phase_id: i64,
        phase: &Phase,
        now: &str,
    ) -> Result<(), RepositoryError> {
        for order in phase.orders.iter() {
            let order_json = Self::serialize_order(order)?;

            transaction
                .execute(
                    r#"
                    INSERT INTO game_phase_orders (
                        phase_id,
                        order_json,
                        created_at,
                        updated_at
                    ) VALUES (?1, ?2, ?3, ?4)
                    "#,
                    params![phase_id, order_json, now, now],
                )
                .map_err(|error| RepositoryError::Unavailable(format!("insert game phase order: {}", error)))?;
        }

        Ok(())
    }

    /// フェイズの命令をロードする（テスト用）
    #[cfg(test)]
    fn load_phase_orders_for_test(connection: &Connection, phase_id: i64) -> Result<Vec<Order>, RepositoryError> {
        let mut statement = connection
            .prepare(
                r#"
                SELECT order_json
                FROM game_phase_orders
                WHERE phase_id = ?1
                "#,
            )
            .map_err(|error| RepositoryError::Unavailable(format!("prepare select phase orders: {}", error)))?;

        let rows = statement
            .query_map(params![phase_id], |row| row.get::<_, String>(0))
            .map_err(|error| RepositoryError::Unavailable(format!("query select phase orders: {}", error)))?;

        rows.map(|row| {
            let order_json = row.map_err(|error| RepositoryError::Unavailable(format!("read phase order row: {}", error)))?;
            Self::deserialize_order(&order_json)
        })
        .collect::<Result<Vec<_>, _>>()
    }
}

/// SQLite 用の卓リポジトリ構造体の実装（GameRepository トレイト）
impl GameRepository for SqliteGameRepository {
    /// ゲームにプレイヤーを追加する
    fn add_player(&self, game_uuid: Uuid, user_uuid: Uuid, requested_power: Option<Power>) -> Result<(), RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;
        let affected = connection
            .execute(
                r#"
                INSERT INTO game_players (game_uuid, user_uuid, power, is_accepting_draw, is_owner, requested_power)
                SELECT ?1, ?2, NULL, 0, 0, ?3
                WHERE NOT EXISTS (
                    SELECT 1 FROM game_players WHERE game_uuid = ?1 AND user_uuid = ?2
                )
                AND (SELECT COUNT(*) FROM game_players WHERE game_uuid = ?1) < 7
                "#,
                params![
                    game_uuid.to_string(),
                    user_uuid.to_string(),
                    requested_power.map(|p| p as i32),
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("insert game player: {}", error)))?;
        if affected == 0 {
            return Err(RepositoryError::Conflict);
        }
        Ok(())
    }

    /// ゲームを挿入する
    fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError> {
        let now = Utc::now().to_rfc3339();
        let game = new_game.game;

        let mut connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        let transaction = connection
            .transaction()
            .map_err(|error| RepositoryError::Unavailable(format!("begin game insert transaction: {}", error)))?;

        transaction
            .execute(
                r#"
                INSERT INTO games (
                    uuid,
                    game_number,
                    keyword,
                    regulation_face_type,
                    regulation_progress_mode,
                    regulation_duration_type,
                    regulation_start_date,
                    regulation_first_period_hour,
                    status,
                    is_draw,
                    is_solo,
                    next_update,
                    created_at,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
                params![
                    game.uuid.to_string(),
                    game.game_number,
                    game.keyword,
                    game.regulation.face_type as i32,
                    game.regulation.progress_mode as i32,
                    game.regulation.duration_type as i32,
                    game.regulation.start_date.to_string(),
                    game.regulation.first_period_hour as i32,
                    Self::serialize_status(game.status),
                    game.is_draw as i32,
                    game.is_solo as i32,
                    game.next_update_at.map(Self::serialize_next_update),
                    now,
                    now,
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("insert game: {}", error)))?;

        for player in &game.players {
            transaction
                .execute(
                    r#"
                    INSERT INTO game_players (
                        game_uuid,
                        user_uuid,
                        power,
                        is_accepting_draw,
                        is_owner,
                        requested_power
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                    "#,
                    params![
                        game.uuid.to_string(),
                        player.user_uuid.to_string(),
                        player.power.map(|v| v as i32),
                        player.is_accepting_draw as i32,
                        player.is_owner as i32,
                        player.requested_power.map(|v| v as i32),
                    ],
                )
                .map_err(|error| RepositoryError::Unavailable(format!("insert game player: {}", error)))?;
        }

        for phase in &game.phases {
            transaction
                .execute(
                    r#"
                    INSERT INTO game_phases (
                        game_uuid,
                        phase_index,
                        phase_year,
                        phase_kind,
                        units,
                        territories,
                        standoff_codes,
                        created_at,
                        updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    "#,
                    params![
                        game.uuid.to_string(),
                        phase.index,
                        phase.year,
                        Self::serialize_phase_kind(&phase.kind)?,
                        Self::serialize_units(&phase.units)?,
                        Self::serialize_territories(&phase.territories),
                        Self::serialize_codes(&phase.standoff_codes),
                        now,
                        now,
                    ],
                )
                .map_err(|error| RepositoryError::Unavailable(format!("insert game phase: {}", error)))?;

            let phase_id = transaction.last_insert_rowid();
            Self::insert_phase_orders(&transaction, phase_id, phase, &now)?;
        }

        transaction
            .commit()
            .map_err(|error| RepositoryError::Unavailable(format!("commit game insert transaction: {}", error)))?;

        Ok(game)
    }

    /// アクティブなゲームを全て取得する
    fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        Self::load_games_by_query(
            &connection,
            r#"
            SELECT uuid, game_number, keyword, regulation_face_type, regulation_progress_mode,
                   regulation_duration_type, regulation_start_date, regulation_first_period_hour,
                   status, is_draw, is_solo, next_update
            FROM games
            WHERE status NOT IN ('closed', 'aborted')
            "#,
            [],
        )
    }

    /// 指定した UUID のゲームを取得する
    fn find_by_uuid(&self, game_uuid: Uuid) -> Result<Option<Game>, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        let mut games = Self::load_games_by_query(
            &connection,
            r#"
            SELECT uuid, game_number, keyword, regulation_face_type, regulation_progress_mode,
                   regulation_duration_type, regulation_start_date, regulation_first_period_hour,
                   status, is_draw, is_solo, next_update
            FROM games
            WHERE uuid = ?1
            "#,
            params![game_uuid.to_string()],
        )?;

        Ok(games.pop())
    }

    /// 更新予定日時が過ぎているゲームの UUID を取得する
    fn find_progress_candidates(&self, now: NaiveDateTime) -> Result<Vec<Uuid>, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        let mut stmt = connection
            .prepare(
                r#"
                SELECT uuid
                FROM games
                WHERE status NOT IN ('closed', 'aborted')
                  AND next_update IS NOT NULL
                                    AND julianday(next_update) <= julianday(?1)
                                ORDER BY julianday(next_update) ASC, uuid ASC
                "#,
            )
            .map_err(|error| RepositoryError::Unavailable(format!("prepare find_progress_candidates: {}", error)))?;

        let uuid_rows = stmt
            .query_map(params![Self::serialize_next_update(now)], |row| row.get::<_, String>(0))
            .map_err(|error| RepositoryError::Unavailable(format!("query find_progress_candidates: {}", error)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| RepositoryError::Unavailable(format!("collect find_progress_candidates: {}", error)))?;

        uuid_rows
            .into_iter()
            .map(|value| {
                Uuid::parse_str(&value)
                    .map_err(|error| RepositoryError::Unavailable(format!("parse progress candidate uuid: {}", error)))
            })
            .collect()
    }

    /// そのユーザーが既に参加しているアクティブなゲームが存在するかを返却する
    fn exists_active_game_for_user(&self, user_uuid: Uuid) -> Result<bool, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        let count: i64 = connection
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM games
                INNER JOIN game_players ON game_players.game_uuid = games.uuid
                WHERE game_players.user_uuid = ?1
                  AND games.status NOT IN ('finished', 'closed', 'aborted')
                "#,
                params![user_uuid.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| RepositoryError::Unavailable(format!("exists_active_game_for_user: {}", error)))?;

        Ok(count > 0)
    }

    /// ゲームを更新する
    fn update(&self, game: &Game) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();

        let mut connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        let transaction = connection
            .transaction()
            .map_err(|error| RepositoryError::Unavailable(format!("begin game update transaction: {}", error)))?;

        transaction
            .execute(
                r#"
                UPDATE games
                SET game_number = ?2,
                    regulation_progress_mode = ?3,
                    status = ?4,
                    is_draw = ?5,
                    is_solo = ?6,
                    next_update = ?7,
                    updated_at = ?8
                WHERE uuid = ?1
                "#,
                params![
                    game.uuid.to_string(),
                    game.game_number,
                    game.regulation.progress_mode as i32,
                    Self::serialize_status(game.status),
                    game.is_draw as i32,
                    game.is_solo as i32,
                    game.next_update_at.map(Self::serialize_next_update),
                    now,
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("update game: {}", error)))?;

        let existing_phase_ids: HashMap<i32, i64> = {
            let mut stmt = transaction
                .prepare("SELECT phase_index, id FROM game_phases WHERE game_uuid = ?1")
                .map_err(|error| RepositoryError::Unavailable(format!("prepare existing phases: {}", error)))?;

            let rows = stmt
                .query_map(params![game.uuid.to_string()], |row| {
                    Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
                })
                .map_err(|error| RepositoryError::Unavailable(format!("query existing phases: {}", error)))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| RepositoryError::Unavailable(format!("collect existing phase ids: {}", error)))?;

            rows.into_iter().collect()
        };

        for phase in &game.phases {
            if let Some(phase_id) = existing_phase_ids.get(&phase.index).copied() {
                transaction
                    .execute(
                        r#"
                        UPDATE game_phases
                        SET phase_year = ?1,
                            phase_kind = ?2,
                            units = ?3,
                            territories = ?4,
                            standoff_codes = ?5,
                            updated_at = ?6
                        WHERE id = ?7
                        "#,
                        params![
                            phase.year,
                            Self::serialize_phase_kind(&phase.kind)?,
                            Self::serialize_units(&phase.units)?,
                            Self::serialize_territories(&phase.territories),
                            Self::serialize_codes(&phase.standoff_codes),
                            now,
                            phase_id,
                        ],
                    )
                    .map_err(|error| RepositoryError::Unavailable(format!("update existing game phase: {}", error)))?;

                transaction
                    .execute("DELETE FROM game_phase_orders WHERE phase_id = ?1", params![phase_id])
                    .map_err(|error| RepositoryError::Unavailable(format!("delete existing phase orders: {}", error)))?;

                Self::insert_phase_orders(&transaction, phase_id, phase, &now)?;
                continue;
            }

            transaction
                .execute(
                    r#"
                    INSERT INTO game_phases (
                        game_uuid,
                        phase_index,
                        phase_year,
                        phase_kind,
                        units,
                        territories,
                        standoff_codes,
                        created_at,
                        updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    "#,
                    params![
                        game.uuid.to_string(),
                        phase.index,
                        phase.year,
                        Self::serialize_phase_kind(&phase.kind)?,
                        Self::serialize_units(&phase.units)?,
                        Self::serialize_territories(&phase.territories),
                        Self::serialize_codes(&phase.standoff_codes),
                        now,
                        now,
                    ],
                )
                .map_err(|error| RepositoryError::Unavailable(format!("insert new game phase: {}", error)))?;

            let phase_id = transaction.last_insert_rowid();
            Self::insert_phase_orders(&transaction, phase_id, phase, &now)?;
        }

        for player in &game.players {
            transaction
                .execute(
                    r#"
                    UPDATE game_players
                    SET is_accepting_draw = ?3,
                        power = ?4
                    WHERE game_uuid = ?1 AND user_uuid = ?2
                    "#,
                    params![
                        game.uuid.to_string(),
                        player.user_uuid.to_string(),
                        player.is_accepting_draw as i32,
                        player.power.map(|p| p as i32),
                    ],
                )
                .map_err(|error| RepositoryError::Unavailable(format!("update game player: {}", error)))?;
        }

        transaction
            .commit()
            .map_err(|error| RepositoryError::Unavailable(format!("commit game update transaction: {}", error)))?;

        Ok(())
    }

    /// 卓番号を採番する
    fn assign_game_number(&self, game_uuid: Uuid) -> Result<i32, RepositoryError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        // TransactionBehavior::Exclusive でトランザクションを開始することで
        // 採番と書き込みをアトミックに行い、重複採番を防ぐ。
        // Transaction は Drop 時に自動ロールバックされるためエラー時も安全。
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)
            .map_err(|error| RepositoryError::Unavailable(format!("begin exclusive transaction: {}", error)))?;

        // 既に採番済みならそのまま返す
        let existing: Option<i32> = transaction
            .query_row(
                "SELECT game_number FROM games WHERE uuid = ?1",
                params![game_uuid.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| RepositoryError::Unavailable(format!("query existing game number: {}", error)))?;

        if let Some(n) = existing {
            return Ok(n);
        }

        // 採番と同時に games テーブルを更新する
        let assigned: i32 = transaction
            .query_row(
                r#"
                UPDATE games
                SET game_number = (SELECT COALESCE(MAX(game_number), 0) + 1 FROM games)
                WHERE uuid = ?1 AND game_number IS NULL
                RETURNING game_number
                "#,
                params![game_uuid.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| RepositoryError::Unavailable(format!("assign game number: {}", error)))?;

        transaction
            .commit()
            .map_err(|error| RepositoryError::Unavailable(format!("commit assign game number: {}", error)))?;

        Ok(assigned)
    }
}

///
/// ユニット情報のペイロード構造体
///
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnitPayload {
    power: Power,
    unit_kind: String,
    location: String,
    dislodged: bool,
    dislodged_from: Option<String>,
}

///
/// 命令情報のペイロード構造体
///
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderPayload {
    power: Power,
    unit: UnitPayload,
    dislodged_from: Option<String>,
    status: OrderStatus,
    kind: OrderKindPayload,
}

///
/// 命令種別情報のペイロード構造体
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum OrderKindPayload {
    Hold,
    Move {
        dest: String,
        via_convoy: bool,
    },
    Support {
        target_unit: UnitPayload,
        target_dest: Option<String>,
    },
    Convoy {
        target_unit: UnitPayload,
        target_dest: String,
    },
    Retreat {
        dest: String,
    },
    Build,
    Disband,
}

///
/// 領土情報のペイロード構造体
///
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TerritoryPayload {
    power: Power,
    code: String,
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::Player;
    use crate::domain::ProgressMode;
    use crate::domain::Regulation;

    #[test]
    fn insert_persists_game_with_owner_and_ready_phase() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let owner_uuid = uuid::Uuid::now_v7();
        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: owner_uuid,
                power: None,
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let created = repository
            .insert(NewGame { game: game.clone() })
            .expect("insert should succeed");

        assert_eq!(created.uuid, game.uuid);
        assert_eq!(created.players.len(), 1);
        assert_eq!(created.players[0].user_uuid, owner_uuid);
        assert!(created.players[0].is_owner);
        assert_eq!(created.players[0].requested_power, Some(Power::France));
        assert_eq!(created.phases.len(), 1);
    }

    #[test]
    fn phase_persistence_can_reconstruct_domain_fields() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let mut phase = Phase::new_ready();
        phase.standoff_codes = vec!["bur".to_string(), "mun".to_string()];

        let mut move_order = Order::new_move(
            Power::France,
            Unit::new_army(Power::France, Province::from_code("par").expect("valid province")),
            Province::from_code("bur").expect("valid province"),
        );
        let move_order = move_order.set_success();

        phase.orders = vec![move_order];

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: true,
                requested_power: None,
            }],
            phases: vec![phase.clone()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        repository.insert(NewGame { game }).expect("insert should succeed");

        let connection = repository.connection.lock().expect("sqlite connection lock should succeed");

        let (phase_id, phase_index, phase_year, phase_kind, units_text, territories_text, standoff_text, created_at, updated_at): (
            i64,
            i32,
            i32,
            String,
            String,
            String,
            String,
            String,
            String,
        ) = connection
            .query_row(
                r#"
                SELECT
                    id,
                    phase_index,
                    phase_year,
                    phase_kind,
                    units,
                    territories,
                    standoff_codes,
                    created_at,
                    updated_at
                FROM game_phases
                ORDER BY id
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                    ))
                },
            )
            .expect("phase row should exist");

        let restored_phase = Phase {
            game_number: None,
            index: phase_index,
            year: phase_year,
            kind: SqliteGameRepository::deserialize_phase_kind(&phase_kind).expect("phase kind parse"),
            units: SqliteGameRepository::deserialize_units(&units_text).expect("units parse"),
            territories: SqliteGameRepository::deserialize_territories(&territories_text).expect("territories parse"),
            standoff_codes: SqliteGameRepository::deserialize_codes(&standoff_text),
            orders: SqliteGameRepository::load_phase_orders_for_test(&connection, phase_id).expect("orders parse"),
        };

        assert_eq!(restored_phase.index, phase.index);
        assert_eq!(restored_phase.year, phase.year);
        assert_eq!(restored_phase.kind, phase.kind);
        assert_eq!(restored_phase.units, phase.units);
        assert_eq!(restored_phase.territories, phase.territories);
        assert_eq!(restored_phase.standoff_codes, phase.standoff_codes);
        {
            let mut restored_jsons = restored_phase
                .orders
                .iter()
                .map(|o| SqliteGameRepository::serialize_order(o).expect("serialize"))
                .collect::<Vec<_>>();
            let mut orig_jsons = phase
                .orders
                .iter()
                .map(|o| SqliteGameRepository::serialize_order(o).expect("serialize"))
                .collect::<Vec<_>>();
            restored_jsons.sort();
            orig_jsons.sort();
            assert_eq!(restored_jsons, orig_jsons);
        }
        assert!(!created_at.is_empty());
        assert!(!updated_at.is_empty());
    }

    #[test]
    fn find_all_active_reconstructs_status_flags_and_next_update() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let next_update = chrono::NaiveDate::from_ymd_opt(2026, 4, 20)
            .expect("valid date")
            .and_hms_opt(9, 0, 0)
            .expect("valid datetime");

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::InProgress,
            is_draw: true,
            is_solo: false,
            next_update_at: Some(next_update),
        };

        repository.insert(NewGame { game }).expect("insert should succeed");

        let active_games = repository.find_all_active().expect("find_all_active should succeed");
        assert_eq!(active_games.len(), 1);
        let restored = &active_games[0];
        assert_eq!(restored.status, GameStatus::InProgress);
        assert!(restored.is_draw);
        assert!(!restored.is_solo);
        assert_eq!(restored.next_update_at, Some(next_update));
    }

    #[test]
    fn update_replaces_existing_phase_snapshot_and_orders() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let mut initial_phase = Phase::new_ready();
        initial_phase.standoff_codes = vec!["old".to_string()];

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![initial_phase],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let mut created = repository.insert(NewGame { game }).expect("insert should succeed");
        let mut updated_phase = created.phases[0].clone();
        updated_phase.year = 1901;
        updated_phase.standoff_codes = vec!["new".to_string()];

        let move_order = Order::new_move(
            Power::France,
            Unit::new_army(Power::France, Province::from_code("par").expect("valid province")),
            Province::from_code("bur").expect("valid province"),
        )
        .set_success();
        updated_phase.orders = vec![move_order];

        created.status = GameStatus::InProgress;
        created.phases = vec![updated_phase];

        repository.update(&created).expect("update should succeed");

        let active_games = repository.find_all_active().expect("find_all_active should succeed");
        assert_eq!(active_games.len(), 1);

        let restored_phase = active_games[0]
            .phases
            .iter()
            .find(|phase| phase.index == 0)
            .expect("phase index 0 should exist");
        assert_eq!(restored_phase.year, 1901);
        assert_eq!(restored_phase.standoff_codes, vec!["new".to_string()]);
        assert_eq!(restored_phase.orders.len(), 1);
        assert!(matches!(restored_phase.orders[0].kind, OrderKind::Move(_)));
        assert!(restored_phase.orders[0].is_success());
    }

    #[test]
    fn update_persists_player_is_accepting_draw() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::InProgress,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let mut created = repository.insert(NewGame { game }).expect("insert should succeed");
        assert!(!created.players[0].is_accepting_draw);

        created.players[0].is_accepting_draw = true;
        repository.update(&created).expect("update should succeed");

        let restored = repository
            .find_by_uuid(created.uuid)
            .expect("find should succeed")
            .expect("game should exist");
        assert!(restored.players[0].is_accepting_draw);
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::Player;
    use crate::domain::ProgressMode;
    use crate::domain::Regulation;

    #[test]
    fn insert_rolls_back_when_player_insert_fails() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        {
            let connection = repository.connection.lock().expect("sqlite connection lock should succeed");
            connection
                .execute_batch(
                    r#"
                    CREATE TRIGGER fail_game_players_insert
                    BEFORE INSERT ON game_players
                    BEGIN
                        SELECT RAISE(ABORT, 'forced player insert failure');
                    END;
                    "#,
                )
                .expect("trigger creation should succeed");
        }

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let game_uuid = uuid::Uuid::now_v7();
        let game = Game {
            uuid: game_uuid,
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: None,
                is_accepting_draw: false,
                is_owner: true,
                requested_power: None,
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Preparing,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        let error = repository
            .insert(NewGame { game })
            .expect_err("insert should fail because trigger aborts player insert");
        assert!(error.to_string().contains("insert game player"));

        let game_count: i64 = repository
            .connection
            .lock()
            .expect("sqlite connection lock should succeed")
            .query_row("SELECT COUNT(*) FROM games WHERE uuid = ?1", [game_uuid.to_string()], |row| {
                row.get(0)
            })
            .expect("count query should succeed");

        assert_eq!(game_count, 0, "game row should be rolled back on failure");
    }

    #[test]
    fn find_all_active_excludes_aborted_games() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Aborted,
            is_draw: false,
            is_solo: false,
            next_update_at: None,
        };

        repository.insert(NewGame { game }).expect("insert should succeed");

        let active_games = repository.find_all_active().expect("find_all_active should succeed");
        assert!(active_games.is_empty(), "aborted game should not appear in find_all_active");
    }

    #[test]
    fn find_progress_candidates_excludes_aborted_games() {
        let repository = SqliteGameRepository::new_in_memory().expect("repository should initialize");

        let regulation = Regulation::new(
            FaceType::Girls,
            ProgressMode::Scheduled,
            DurationType::Short,
            chrono::NaiveDate::from_ymd_opt(2026, 4, 19).expect("valid date"),
            12,
        )
        .expect("valid regulation");

        let past = chrono::NaiveDate::from_ymd_opt(2026, 4, 18)
            .expect("valid date")
            .and_hms_opt(9, 0, 0)
            .expect("valid datetime");
        let now = chrono::NaiveDate::from_ymd_opt(2026, 4, 19)
            .expect("valid date")
            .and_hms_opt(9, 0, 0)
            .expect("valid datetime");

        let game = Game {
            uuid: uuid::Uuid::now_v7(),
            game_number: None,
            keyword: None,
            regulation,
            players: vec![Player {
                user_uuid: uuid::Uuid::now_v7(),
                power: Some(Power::France),
                is_accepting_draw: false,
                is_owner: true,
                requested_power: Some(Power::France),
            }],
            phases: vec![Phase::new_ready()],
            status: GameStatus::Aborted,
            is_draw: false,
            is_solo: false,
            next_update_at: Some(past),
        };

        repository.insert(NewGame { game }).expect("insert should succeed");

        let candidates = repository
            .find_progress_candidates(now)
            .expect("find_progress_candidates should succeed");
        assert!(
            candidates.is_empty(),
            "aborted game should not appear in find_progress_candidates"
        );
    }
}
