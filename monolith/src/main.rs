use infocar_monolith::cli::check_services::run_check_services;
use infocar_monolith::config::AppConfig;
use infocar_monolith::http::routes::build_router;
use infocar_monolith::http::state::AppState;
use std::env;
use std::net::SocketAddr;
use std::process;

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "check-services" {
        let ok = run_check_services(&config).await;
        if ok {
            process::exit(0);
        } else {
            process::exit(1);
        }
    }

    let port = config.port;
    let state = AppState::from_config(config);
    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Infocar Rust Monolith listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Failed to start Axum server");
}
