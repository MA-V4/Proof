use axum::{extract::State, Json};
use crate::state::SharedState;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status:          &'static str,
    specs:           usize,
    events_verified: u64,
    divergences:     i64,
}

pub async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    let s    = state.read().await;
    let db   = s.db.clone();
    let evs  = s.events_verified;
    let nsp  = s.specs.len();
    drop(s);

    let divergences = db.count_divergences(None).await.unwrap_or(0);

    Json(HealthResponse {
        status: "ok",
        specs:  nsp,
        events_verified: evs,
        divergences,
    })
}