// ============================================================================
// imports
// ============================================================================

use std::env::var;
use std::net::SocketAddr;
use std::process::exit;

// ============================================================================
// functions
// ============================================================================

///
/// Tokio メイン関数
///
#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let main_db_path = var("MAIN_DATABASE_PATH").unwrap_or_else(|_| "moe3.main.db".to_string());
    let messages_db_path = var("MESSAGES_DATABASE_PATH").unwrap_or_else(|_| "moe3.messages.db".to_string());
    // 未指定時は debug ビルドなら全オリジン許可、release ビルドなら許可なし（安全側）
    let default_cors_allowed_origins = if cfg!(debug_assertions) { "*" } else { "" };
    let cors_allowed_origins: Vec<String> = var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| default_cors_allowed_origins.to_string())
        .split(',')
        .map(|origin| origin.trim().to_string())
        .filter(|origin| !origin.is_empty())
        .collect();

    println!("Listening on {addr}");
    println!("CORS allowed origins: {cors_allowed_origins:?}");

    if let Err(e) = server::serve(addr, &main_db_path, &messages_db_path, &cors_allowed_origins).await {
        eprintln!("Server error: {e}");
        exit(1);
    }
}
