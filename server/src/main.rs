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

    println!("Listening on {addr}");

    if let Err(e) = server::serve(addr, &main_db_path, &messages_db_path).await {
        eprintln!("Server error: {e}");
        exit(1);
    }
}
