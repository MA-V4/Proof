use axum::{extract::State, Json};
use crate::state::SharedState;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status:          &'static str,
    specs:           usize,
    events_verified: u64,
    divergences:     usize,
}

pub async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    let s = state.read().await;
    Json(HealthResponse {
        status:          "ok",
        specs:           s.specs.len(),
        events_verified: s.events_verified,
        divergences:     s.divergences.len(),
    })
}