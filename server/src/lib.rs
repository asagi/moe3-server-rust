#![cfg_attr(not(test), allow(unused_imports))]
// ============================================================================
// modules
// ============================================================================

mod api;
mod domain;
mod repositories;
mod services;

// ============================================================================
// exports
// ============================================================================

// structs
pub(crate) use domain::Game;
pub(crate) use domain::Order;
pub(crate) use domain::Phase;
pub(crate) use domain::PhaseContext;
pub(crate) use domain::Player;
pub(crate) use domain::Province;
pub(crate) use domain::Regulation;
pub(crate) use domain::Territory;
pub(crate) use domain::Unit;
pub(crate) use repositories::DiscordProfile;
pub(crate) use repositories::NewGame;
pub(crate) use repositories::NewUser;
pub(crate) use repositories::RepositoryError;
pub(crate) use repositories::SqliteGameRepository;
pub(crate) use repositories::SqliteUserRepository;
pub(crate) use repositories::UserProfileUpdate;
pub(crate) use repositories::UserRecord;
pub(crate) use repositories::UserRepository;
pub(crate) use services::AuthService;
pub(crate) use services::CreateGameCommand;
pub(crate) use services::GameProgressionService;
pub(crate) use services::GameService;
pub(crate) use services::LoginCommand;
pub(crate) use services::LoginUser;

// enums
pub(crate) use domain::DurationType;
pub(crate) use domain::FaceType;
pub(crate) use domain::GameStatus;
pub(crate) use domain::OrderKind;
pub(crate) use domain::OrderStatus;
pub(crate) use domain::PhaseKind;
pub(crate) use domain::Power;
pub(crate) use domain::ProgressMode;
pub(crate) use services::AuthError;
pub(crate) use services::CreateGameError;
pub(crate) use services::DiscordClientError;
pub(crate) use services::GameProgressionError;

// traits
pub(crate) use repositories::GameRepository;
pub(crate) use services::DiscordIdentityProvider;

// type aliases
pub(crate) use repositories::UserId;

// ============================================================================
// internal: single-instance lock
// ============================================================================

use fs4::FileExt;
use std::fs::File;
use std::sync::Arc;
use std::sync::Mutex;

/// Global lock file handle. Held for the lifetime of the server process.
static INSTANCE_LOCK: Mutex<Option<Arc<File>>> = Mutex::new(None);

fn acquire_instance_lock(db_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let lock_path = format!("{}.lock", db_path);
    let file = File::create(&lock_path)?;

    match file.try_lock_exclusive() {
        Ok(()) => {
            let mut lock = INSTANCE_LOCK.lock().unwrap();
            *lock = Some(Arc::new(file));
            println!("[LOCK] Acquired exclusive lock on {}", lock_path);
            Ok(())
        }
        Err(_) => Err(format!("Another instance is already running (lock file: {})", lock_path).into()),
    }
}

// ============================================================================
// public API
// ============================================================================

pub async fn serve(addr: std::net::SocketAddr, db_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Ensure single instance
    acquire_instance_lock(db_path)?;

    let user_repository = SqliteUserRepository::new(db_path)?;
    let game_repository = SqliteGameRepository::new(db_path)?;
    let game_service = GameService::new(user_repository, game_repository.clone());
    let pre_handler = api::GlobalPreHandler::new(game_repository);
    let state = api::AppState::new(game_service, pre_handler);
    let router = api::create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
