use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "moe3.sqlite3".to_string());

    println!("Listening on {addr}");

    if let Err(e) = server::serve(addr, &db_path).await {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}
