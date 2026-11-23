use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::env;
use std::net::SocketAddr;

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "monolith"
    }))
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "check-services" {
        println!("Infocar Core Status: SAUDÁVEL (Bootstrap Rust operacional)");
        return;
    }

    let app = Router::new().route("/api/health", get(health));

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Infocar Rust Monolith listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to start Axum server");
}
