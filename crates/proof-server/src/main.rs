mod db;
mod error;
mod routes;
mod state;

use axum::{
    routing::{delete, get, post},
    Router,
};
use db::Db;
use state::{AppState, SharedState};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "proof_server=info".into()))
        .init();

    let db_path = std::env::var("PROOF_DB").unwrap_or_else(|_| "proof.db".into());
    let specs_dir = std::env::var("PROOF_SPECS_DIR").unwrap_or_else(|_| "examples".into());

    tracing::info!("opening database: {}", db_path);
    let db = Db::open(&db_path).await?;
    db.migrate().await?;
    tracing::info!("migrations applied");

    let app_state = AppState::load(&specs_dir, db).await?;
    let state: SharedState = Arc::new(RwLock::new(app_state));

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/specs", get(routes::specs::list_specs))
        .route(
            "/specs/:name/divergences",
            get(routes::specs::get_divergences),
        )
        .route(
            "/specs/:name/divergences/:id",
            delete(routes::specs::resolve_divergence),
        )
        .route("/specs/:name/audit", get(routes::audit::get_spec_audit))
        .route(
            "/specs/:name/audit/export",
            get(routes::audit::export_fca_pack),
        )
        .route("/specs/:name/signoff", post(routes::audit::sign_off))
        .route("/audit", get(routes::audit::get_all_audit))
        .route("/events/recent", get(routes::events::recent_events))
        .route("/verify/:spec_name", post(routes::verify::verify_event))
        .route(
            "/verify/:spec_name/batch",
            post(routes::verify::verify_batch),
        )
        .route("/simulate", post(routes::simulate::simulate))
        .route("/diff", post(routes::diff::diff))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3001";
    tracing::info!("PROOF server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
