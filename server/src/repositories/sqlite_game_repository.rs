#![cfg_attr(not(test), allow(dead_code))]
// ============================================================================
// imports
// ============================================================================

// standard library
use std::sync::Mutex;

// external crates
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Utc;
use rusqlite::Connection;
use rusqlite::Transaction;
use rusqlite::params;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

// structs
use super::DurationType;
use super::FaceType;
use super::Game;
use super::NewGame;
use super::Order;
use super::Phase;
use super::PhaseKind;
use super::Player;
use super::ProgressMode;
use super::Province;
use super::Regulation;
use super::Territory;
use super::Unit;

// enums
use super::GameStatus;
use super::OrderKind;
use super::OrderStatus;
use super::Power;
use super::RepositoryError;

// traits
use super::GameRepository;

// ============================================================================
// definitions
// ============================================================================

pub(crate) struct SqliteGameRepository {
    connection: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UnitPayload {
    power: Power,
    unit_kind: String,
    location: String,
    dislodged: bool,
    dislodged_from: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderPayload {
    power: Power,
    unit: UnitPayload,
    dislodged_from: Option<String>,
    status: OrderStatus,
    kind: OrderKindPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TerritoryPayload {
    power: Power,
    code: String,
}

impl SqliteGameRepository {
    #[allow(dead_code)]
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
            CREATE TABLE IF NOT EXISTS games (
                uuid TEXT PRIMARY KEY,
                game_number INTEGER,
                regulation_face_type INTEGER NOT NULL,
                regulation_progress_mode INTEGER NOT NULL,
                regulation_duration_type INTEGER NOT NULL,
                regulation_start_date TEXT NOT NULL,
                regulation_first_period_hour INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'preparing',
                is_canceled INTEGER NOT NULL DEFAULT 0,
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

    fn status_to_str(status: GameStatus) -> &'static str {
        match status {
            GameStatus::Preparing => "preparing",
            GameStatus::Ready => "ready",
            GameStatus::InProgress => "in_progress",
            GameStatus::Finished => "finished",
            GameStatus::Closed => "closed",
        }
    }

    fn status_from_str(text: &str) -> Result<GameStatus, RepositoryError> {
        match text {
            "preparing" => Ok(GameStatus::Preparing),
            "ready" => Ok(GameStatus::Ready),
            "in_progress" => Ok(GameStatus::InProgress),
            "finished" => Ok(GameStatus::Finished),
            "closed" => Ok(GameStatus::Closed),
            _ => Err(RepositoryError::Unavailable(format!("unknown game status: {}", text))),
        }
    }

    fn power_from_i32(value: i32) -> Result<Power, RepositoryError> {
        use strum::IntoEnumIterator;
        Power::iter()
            .find(|p| *p as i32 == value)
            .ok_or_else(|| RepositoryError::Unavailable(format!("invalid power value: {}", value)))
    }

    fn serialize_status(status: GameStatus) -> String {
        Self::status_to_str(status).to_string()
    }

    fn serialize_phase_kind(kind: &PhaseKind) -> Result<String, RepositoryError> {
        serde_json::to_string(kind).map_err(|error| RepositoryError::Unavailable(format!("serialize phase kind json: {}", error)))
    }

    fn deserialize_phase_kind(text: &str) -> Result<PhaseKind, RepositoryError> {
        serde_json::from_str(text)
            .map_err(|error| RepositoryError::Unavailable(format!("deserialize phase kind json: {}", error)))
    }

    fn unit_kind_name(unit: &Unit) -> &'static str {
        if unit.is_fleet() { "fleet" } else { "army" }
    }

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

    fn unit_to_payload(unit: &Unit) -> UnitPayload {
        UnitPayload {
            power: unit.power,
            unit_kind: Self::unit_kind_name(unit).to_string(),
            location: unit.location.code_with_coast().to_string(),
            dislodged: unit.dislodged,
            dislodged_from: unit.dislodged_from.map(|province| province.code_with_coast().to_string()),
        }
    }

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

    fn serialize_units(units: &[Unit]) -> Result<String, RepositoryError> {
        let payloads = units.iter().map(Self::unit_to_payload).collect::<Vec<_>>();
        serde_json::to_string(&payloads).map_err(|error| RepositoryError::Unavailable(format!("serialize units json: {}", error)))
    }

    fn deserialize_units(text: &str) -> Result<Vec<Unit>, RepositoryError> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let payloads: Vec<UnitPayload> = serde_json::from_str(text)
            .map_err(|error| RepositoryError::Unavailable(format!("deserialize units json: {}", error)))?;

        payloads.iter().map(Self::unit_from_payload).collect::<Result<Vec<_>, _>>()
    }

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

    fn serialize_codes(codes: &[String]) -> String {
        serde_json::to_string(codes).unwrap_or_else(|_| codes.join(","))
    }

    fn deserialize_codes(text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        // Try JSON first, fallback to comma-separated for backward compatibility
        serde_json::from_str::<Vec<String>>(text)
            .unwrap_or_else(|_| text.split(',').map(|value| value.to_string()).collect::<Vec<_>>())
    }

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

impl GameRepository for SqliteGameRepository {
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
                    regulation_face_type,
                    regulation_progress_mode,
                    regulation_duration_type,
                    regulation_start_date,
                    regulation_first_period_hour,
                    status,
                    is_canceled,
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
                    game.regulation.face_type as i32,
                    game.regulation.progress_mode as i32,
                    game.regulation.duration_type as i32,
                    game.regulation.start_date.to_string(),
                    game.regulation.first_period_hour as i32,
                    Self::serialize_status(game.status),
                    game.is_canceld as i32,
                    game.is_draw as i32,
                    game.is_solo as i32,
                    game.next_update.map(|dt| dt.to_string()),
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

    fn find_all_active(&self) -> Result<Vec<Game>, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        struct GameRow {
            uuid: String,
            game_number: Option<i32>,
            face_type: i32,
            progress_mode: i32,
            duration_type: i32,
            start_date: String,
            first_period_hour: i32,
            status: String,
            is_canceled: i32,
            is_draw: i32,
            is_solo: i32,
            next_update: Option<String>,
        }

        let game_rows: Vec<GameRow> = {
            let mut stmt = connection
                .prepare(
                    r#"
                    SELECT uuid, game_number, regulation_face_type, regulation_progress_mode,
                           regulation_duration_type, regulation_start_date, regulation_first_period_hour,
                           status, is_canceled, is_draw, is_solo, next_update
                    FROM games
                    WHERE status != 'closed'
                    "#,
                )
                .map_err(|error| RepositoryError::Unavailable(format!("prepare find_all_active: {}", error)))?;

            stmt.query_map([], |row| {
                Ok(GameRow {
                    uuid: row.get(0)?,
                    game_number: row.get(1)?,
                    face_type: row.get(2)?,
                    progress_mode: row.get(3)?,
                    duration_type: row.get(4)?,
                    start_date: row.get(5)?,
                    first_period_hour: row.get(6)?,
                    status: row.get(7)?,
                    is_canceled: row.get(8)?,
                    is_draw: row.get(9)?,
                    is_solo: row.get(10)?,
                    next_update: row.get(11)?,
                })
            })
            .map_err(|error| RepositoryError::Unavailable(format!("query find_all_active: {}", error)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| RepositoryError::Unavailable(format!("collect game rows: {}", error)))?
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

            let next_update = row
                .next_update
                .as_deref()
                .map(|s| {
                    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                        .map_err(|error| RepositoryError::Unavailable(format!("parse next_update: {}", error)))
                })
                .transpose()?;

            games.push(Game {
                uuid,
                game_number: row.game_number,
                regulation,
                players,
                phases,
                status: Self::status_from_str(&row.status)?,
                is_canceld: row.is_canceled != 0,
                is_draw: row.is_draw != 0,
                is_solo: row.is_solo != 0,
                next_update,
            });
        }

        Ok(games)
    }

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
                SET status = ?2,
                    is_canceled = ?3,
                    is_draw = ?4,
                    is_solo = ?5,
                    next_update = ?6,
                    updated_at = ?7
                WHERE uuid = ?1
                "#,
                params![
                    game.uuid.to_string(),
                    Self::serialize_status(game.status),
                    game.is_canceld as i32,
                    game.is_draw as i32,
                    game.is_solo as i32,
                    game.next_update.map(|dt| dt.to_string()),
                    now,
                ],
            )
            .map_err(|error| RepositoryError::Unavailable(format!("update game: {}", error)))?;

        let existing_indices: Vec<i32> = {
            let mut stmt = transaction
                .prepare("SELECT phase_index FROM game_phases WHERE game_uuid = ?1")
                .map_err(|error| RepositoryError::Unavailable(format!("prepare existing phases: {}", error)))?;

            stmt.query_map(params![game.uuid.to_string()], |row| row.get(0))
                .map_err(|error| RepositoryError::Unavailable(format!("query existing phases: {}", error)))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| RepositoryError::Unavailable(format!("collect existing phase indices: {}", error)))?
        };

        for phase in &game.phases {
            if existing_indices.contains(&phase.index) {
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

        transaction
            .commit()
            .map_err(|error| RepositoryError::Unavailable(format!("commit game update transaction: {}", error)))?;

        Ok(())
    }
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
            is_canceld: false,
            is_draw: false,
            is_solo: false,
            next_update: None,
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
            is_canceld: false,
            is_draw: false,
            is_solo: false,
            next_update: None,
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
            is_canceld: false,
            is_draw: false,
            is_solo: false,
            next_update: None,
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
}
