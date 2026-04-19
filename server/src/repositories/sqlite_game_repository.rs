use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::Transaction;
use rusqlite::params;

use super::GameRepository;
use super::NewGame;
use super::RepositoryError;
use crate::domain::Game;
use crate::domain::Order;
use crate::domain::OrderKind;
use crate::domain::Phase;
use crate::domain::PhaseKind;
use crate::domain::Power;
use crate::domain::Province;
use crate::domain::Territory;
use crate::domain::Unit;

pub(crate) struct SqliteGameRepository {
    connection: Mutex<Connection>,
}

impl SqliteGameRepository {
    #[cfg_attr(test, allow(dead_code))]
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
                order_index INTEGER NOT NULL,
                power TEXT NOT NULL,
                unit_power TEXT NOT NULL,
                unit_location TEXT NOT NULL,
                unit_kind TEXT NOT NULL,
                unit_dislodged_from TEXT,
                unit_dislodged INTEGER NOT NULL,
                dislodged_from TEXT,
                status TEXT NOT NULL,
                order_kind TEXT NOT NULL,
                dest TEXT,
                via_convoy INTEGER,
                target_unit_power TEXT,
                target_unit_location TEXT,
                target_unit_kind TEXT,
                target_unit_dislodged_from TEXT,
                target_unit_dislodged INTEGER,
                target_dest TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(phase_id, order_index)
            );
        "#;

        self.connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?
            .execute_batch(sql)
            .map_err(|error| RepositoryError::Unavailable(format!("create game tables: {}", error)))?;

        Ok(())
    }

    fn phase_kind_name(kind: &PhaseKind) -> &'static str {
        match kind {
            PhaseKind::Ready(_) => "ready",
            PhaseKind::SpringMain(_) => "spring_main",
            PhaseKind::SpringRetreat(_) => "spring_retreat",
            PhaseKind::FallMain(_) => "fall_main",
            PhaseKind::FallRetreat(_) => "fall_retreat",
            PhaseKind::Adjustment(_) => "adjustment",
            PhaseKind::Debrief(_) => "debrief",
        }
    }

    fn phase_kind_from_name(name: &str) -> Result<PhaseKind, RepositoryError> {
        let kind = match name {
            "ready" => Phase::new_ready().kind,
            "spring_main" => Phase::new_spring_main(1900, 0).kind,
            "spring_retreat" => Phase::new_spring_retreat(1901, 0).kind,
            "fall_main" => Phase::new_fall_main(1901, 0).kind,
            "fall_retreat" => Phase::new_fall_retreat(1901, 0).kind,
            "adjustment" => Phase::new_adjustment(1901, 0).kind,
            "debrief" => Phase::new_debrief(1901, 0).kind,
            _ => return Err(RepositoryError::Unavailable(format!("unknown phase_kind: {}", name))),
        };
        Ok(kind)
    }
    fn power_to_code(power: Power) -> &'static str {
        match power {
            Power::Austria => "a",
            Power::England => "e",
            Power::France => "f",
            Power::Germany => "g",
            Power::Italy => "i",
            Power::Russia => "r",
            Power::Turkey => "t",
        }
    }

    fn power_from_code(code: &str) -> Result<Power, RepositoryError> {
        match code {
            "a" => Ok(Power::Austria),
            "e" => Ok(Power::England),
            "f" => Ok(Power::France),
            "g" => Ok(Power::Germany),
            "i" => Ok(Power::Italy),
            "r" => Ok(Power::Russia),
            "t" => Ok(Power::Turkey),
            _ => Err(RepositoryError::Unavailable(format!("invalid power code: {}", code))),
        }
    }

    fn order_status_name(order: &Order) -> &'static str {
        if order.is_unresolved() {
            "unresolved"
        } else if order.is_failure() {
            "failure"
        } else if order.is_success() {
            "success"
        } else if order.is_dislodged() {
            "dislodged"
        } else if order.is_cut() {
            "cut"
        } else if order.is_valid() {
            "valid"
        } else if order.is_invalid() {
            "invalid"
        } else if order.is_unreachable() {
            "unreachable"
        } else {
            "unresolved"
        }
    }

    fn apply_order_status(mut order: Order, status: &str) -> Result<Order, RepositoryError> {
        order = match status {
            "unresolved" => order.set_unresolved(),
            "failure" => order.set_failure(),
            "success" => order.set_success(),
            "dislodged" => order.set_dislodged(),
            "cut" => order.set_cut(),
            "valid" => order.set_valid(),
            "invalid" => order.set_invalid(),
            "unreachable" => order.set_unreachable(),
            _ => return Err(RepositoryError::Unavailable(format!("invalid order status: {}", status))),
        };

        Ok(order)
    }

    fn order_kind_name(kind: &OrderKind) -> &'static str {
        match kind {
            OrderKind::Hold(_) => "hold",
            OrderKind::Move(_) => "move",
            OrderKind::Support(_) => "support",
            OrderKind::Convoy(_) => "convoy",
            OrderKind::Retreat(_) => "retreat",
            OrderKind::Build(_) => "build",
            OrderKind::Disband(_) => "disband",
        }
    }

    fn parse_province(code: &str) -> Result<Province, RepositoryError> {
        Province::from_code(code).ok_or_else(|| RepositoryError::Unavailable(format!("invalid province code: {}", code)))
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

    fn serialize_unit(unit: &Unit) -> String {
        let dislodged_from = unit
            .dislodged_from
            .map(|province| province.code_with_coast().to_string())
            .unwrap_or_default();
        format!(
            "{}|{}|{}|{}|{}",
            Self::power_to_code(unit.power),
            Self::unit_kind_name(unit),
            unit.location.code_with_coast(),
            if unit.dislodged { 1 } else { 0 },
            dislodged_from,
        )
    }

    fn deserialize_unit(text: &str) -> Result<Unit, RepositoryError> {
        let parts = text.split('|').collect::<Vec<_>>();
        if parts.len() != 5 {
            return Err(RepositoryError::Unavailable(format!("invalid unit payload: {}", text)));
        }

        let power = Self::power_from_code(parts[0])?;
        let location = Self::parse_province(parts[2])?;
        let dislodged = parts[3] == "1";
        let dislodged_from = if parts[4].is_empty() {
            None
        } else {
            Some(Self::parse_province(parts[4])?)
        };

        Self::build_unit(power, location, parts[1], dislodged_from, dislodged)
    }

    fn serialize_units(units: &[Unit]) -> String {
        units.iter().map(Self::serialize_unit).collect::<Vec<_>>().join("\n")
    }

    fn deserialize_units(text: &str) -> Result<Vec<Unit>, RepositoryError> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        text.lines().map(Self::deserialize_unit).collect::<Result<Vec<_>, _>>()
    }

    fn serialize_territory(territory: &Territory) -> String {
        format!("{}|{}", Self::power_to_code(territory.power), territory.code_with_coast())
    }

    fn deserialize_territory(text: &str) -> Result<Territory, RepositoryError> {
        let parts = text.split('|').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(RepositoryError::Unavailable(format!("invalid territory payload: {}", text)));
        }

        let power = Self::power_from_code(parts[0])?;

        Ok(Territory::new(power, parts[1]))
    }

    fn serialize_territories(territories: &[Territory]) -> String {
        territories
            .iter()
            .map(Self::serialize_territory)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn deserialize_territories(text: &str) -> Result<Vec<Territory>, RepositoryError> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        text.lines().map(Self::deserialize_territory).collect::<Result<Vec<_>, _>>()
    }

    fn serialize_codes(codes: &[String]) -> String {
        codes.join(",")
    }

    fn deserialize_codes(text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        text.split(',').map(|value| value.to_string()).collect::<Vec<_>>()
    }

    fn insert_phase_orders(
        transaction: &Transaction<'_>,
        phase_id: i64,
        phase: &Phase,
        now: &str,
    ) -> Result<(), RepositoryError> {
        for (order_index, order) in phase.orders.iter().enumerate() {
            let unit_dislodged_from = order
                .unit
                .dislodged_from
                .map(|province| province.code_with_coast().to_string());
            let dislodged_from = order.dislodged_from.map(|province| province.code_with_coast().to_string());

            let (dest, via_convoy, target_unit, target_dest) = match order.kind {
                OrderKind::Hold(_) | OrderKind::Build(_) | OrderKind::Disband(_) => (None, None, None, None),
                OrderKind::Move(move_order) => (
                    Some(move_order.dest.code_with_coast().to_string()),
                    Some(if move_order.via_convoy { 1 } else { 0 }),
                    None,
                    None,
                ),
                OrderKind::Support(support_order) => (
                    None,
                    None,
                    Some(support_order.target_unit),
                    support_order
                        .target_dest
                        .map(|province| province.code_with_coast().to_string()),
                ),
                OrderKind::Convoy(convoy_order) => (
                    None,
                    None,
                    Some(convoy_order.target_unit),
                    Some(convoy_order.target_dest.code_with_coast().to_string()),
                ),
                OrderKind::Retreat(retreat_order) => (Some(retreat_order.dest.code_with_coast().to_string()), None, None, None),
            };

            let target_unit_power = target_unit.map(|unit| Self::power_to_code(unit.power).to_string());
            let target_unit_location = target_unit.map(|unit| unit.location.code_with_coast().to_string());
            let target_unit_kind = target_unit.map(|unit| Self::unit_kind_name(&unit).to_string());
            let target_unit_dislodged_from =
                target_unit.and_then(|unit| unit.dislodged_from.map(|province| province.code_with_coast().to_string()));
            let target_unit_dislodged = target_unit.map(|unit| if unit.dislodged { 1 } else { 0 });

            transaction
                .execute(
                    r#"
                    INSERT INTO game_phase_orders (
                        phase_id,
                        order_index,
                        power,
                        unit_power,
                        unit_location,
                        unit_kind,
                        unit_dislodged_from,
                        unit_dislodged,
                        dislodged_from,
                        status,
                        order_kind,
                        dest,
                        via_convoy,
                        target_unit_power,
                        target_unit_location,
                        target_unit_kind,
                        target_unit_dislodged_from,
                        target_unit_dislodged,
                        target_dest,
                        created_at,
                        updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
                    "#,
                    params![
                        phase_id,
                        order_index as i32,
                        Self::power_to_code(order.power),
                        Self::power_to_code(order.unit.power),
                        order.unit.location.code_with_coast(),
                        Self::unit_kind_name(&order.unit),
                        unit_dislodged_from,
                        if order.unit.dislodged { 1 } else { 0 },
                        dislodged_from,
                        Self::order_status_name(order),
                        Self::order_kind_name(&order.kind),
                        dest,
                        via_convoy,
                        target_unit_power,
                        target_unit_location,
                        target_unit_kind,
                        target_unit_dislodged_from,
                        target_unit_dislodged,
                        target_dest,
                        now,
                        now,
                    ],
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
                SELECT
                    power,
                    unit_power,
                    unit_location,
                    unit_kind,
                    unit_dislodged_from,
                    unit_dislodged,
                    dislodged_from,
                    status,
                    order_kind,
                    dest,
                    via_convoy,
                    target_unit_power,
                    target_unit_location,
                    target_unit_kind,
                    target_unit_dislodged_from,
                    target_unit_dislodged,
                    target_dest
                FROM game_phase_orders
                WHERE phase_id = ?1
                ORDER BY order_index
                "#,
            )
            .map_err(|error| RepositoryError::Unavailable(format!("prepare select phase orders: {}", error)))?;

        let rows = statement
            .query_map(params![phase_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<i32>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, Option<String>>(14)?,
                    row.get::<_, Option<i32>>(15)?,
                    row.get::<_, Option<String>>(16)?,
                ))
            })
            .map_err(|error| RepositoryError::Unavailable(format!("query select phase orders: {}", error)))?;

        let mut orders = Vec::new();
        for row in rows {
            let (
                power,
                unit_power,
                unit_location,
                unit_kind,
                unit_dislodged_from,
                unit_dislodged,
                dislodged_from,
                status,
                order_kind,
                dest,
                via_convoy,
                target_unit_power,
                target_unit_location,
                target_unit_kind,
                target_unit_dislodged_from,
                target_unit_dislodged,
                target_dest,
            ) = row.map_err(|error| RepositoryError::Unavailable(format!("read phase order row: {}", error)))?;

            let unit = Self::build_unit(
                Self::power_from_code(&unit_power)?,
                Self::parse_province(&unit_location)?,
                &unit_kind,
                match unit_dislodged_from {
                    Some(value) => Some(Self::parse_province(&value)?),
                    None => None,
                },
                unit_dislodged == 1,
            )?;

            let target_unit = match (
                target_unit_power,
                target_unit_location,
                target_unit_kind,
                target_unit_dislodged,
            ) {
                (Some(t_power), Some(t_location), Some(t_kind), Some(t_dislodged)) => Some(Self::build_unit(
                    Self::power_from_code(&t_power)?,
                    Self::parse_province(&t_location)?,
                    &t_kind,
                    match target_unit_dislodged_from {
                        Some(value) => Some(Self::parse_province(&value)?),
                        None => None,
                    },
                    t_dislodged == 1,
                )?),
                (None, None, None, None) => None,
                _ => return Err(RepositoryError::Unavailable("inconsistent target unit columns".to_string())),
            };

            let power = Self::power_from_code(&power)?;
            let mut order = match order_kind.as_str() {
                "hold" => Order::new_hold(power, unit),
                "move" => {
                    let dest = Self::parse_province(
                        dest.as_deref()
                            .ok_or_else(|| RepositoryError::Unavailable("missing move dest".to_string()))?,
                    )?;
                    let mut move_order = Order::new_move(power, unit, dest);
                    if via_convoy.unwrap_or(0) == 1 {
                        move_order = move_order.set_via_convoy();
                    }
                    move_order
                }
                "support" => Order::new_support(
                    power,
                    unit,
                    target_unit.ok_or_else(|| RepositoryError::Unavailable("missing support target unit".to_string()))?,
                    match target_dest {
                        Some(value) => Some(Self::parse_province(&value)?),
                        None => None,
                    },
                ),
                "convoy" => Order::new_convoy(
                    power,
                    unit,
                    target_unit.ok_or_else(|| RepositoryError::Unavailable("missing convoy target unit".to_string()))?,
                    Self::parse_province(
                        target_dest
                            .as_deref()
                            .ok_or_else(|| RepositoryError::Unavailable("missing convoy target dest".to_string()))?,
                    )?,
                ),
                "retreat" => Order::new_retreat(
                    power,
                    unit,
                    Self::parse_province(
                        dest.as_deref()
                            .ok_or_else(|| RepositoryError::Unavailable("missing retreat dest".to_string()))?,
                    )?,
                ),
                "build" => Order::new_build(power, unit),
                "disband" => Order::new_disband(power, unit),
                _ => return Err(RepositoryError::Unavailable(format!("invalid order kind: {}", order_kind))),
            };

            order.dislodged_from = match dislodged_from {
                Some(value) => Some(Self::parse_province(&value)?),
                None => None,
            };

            orders.push(Self::apply_order_status(order, &status)?);
        }

        Ok(orders)
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
                    created_at,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    game.uuid.to_string(),
                    game.game_number,
                    game.regulation.face_type as i32,
                    game.regulation.progress_mode as i32,
                    game.regulation.duration_type as i32,
                    game.regulation.start_date.to_string(),
                    game.regulation.first_period_hour as i32,
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
                        Self::phase_kind_name(&phase.kind),
                        Self::serialize_units(&phase.units),
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
}

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
            kind: SqliteGameRepository::phase_kind_from_name(&phase_kind).expect("phase kind parse"),
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
        assert_eq!(restored_phase.orders, phase.orders);
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
