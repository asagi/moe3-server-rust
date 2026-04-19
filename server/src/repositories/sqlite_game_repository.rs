use std::sync::Mutex;

use chrono::Utc;
use rusqlite::Connection;
use rusqlite::params;

use super::GameRepository;
use super::NewGame;
use super::RepositoryError;
use crate::domain::Game;
use crate::domain::PhaseKind;

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
                phase_kind TEXT NOT NULL
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
}

impl GameRepository for SqliteGameRepository {
    fn insert(&self, new_game: NewGame) -> Result<Game, RepositoryError> {
        let now = Utc::now().to_rfc3339();
        let game = new_game.game;

        let connection = self
            .connection
            .lock()
            .map_err(|error| RepositoryError::Unavailable(format!("lock sqlite connection: {}", error)))?;

        connection
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
            connection
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
            connection
                .execute(
                    r#"
                    INSERT INTO game_phases (game_uuid, phase_index, phase_year, phase_kind)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    params![
                        game.uuid.to_string(),
                        phase.index,
                        phase.year,
                        Self::phase_kind_name(&phase.kind)
                    ],
                )
                .map_err(|error| RepositoryError::Unavailable(format!("insert game phase: {}", error)))?;
        }

        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DurationType;
    use crate::domain::FaceType;
    use crate::domain::Phase;
    use crate::domain::Player;
    use crate::domain::Power;
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
}
