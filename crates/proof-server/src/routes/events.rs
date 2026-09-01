use axum::{extract::State, Json};
use crate::state::{RecentEvent, SharedState};

pub async fn recent_events(State(state): State<SharedState>) -> Json<Vec<RecentEvent>> {
    let s = state.read().await;
    Json(s.recent.iter().take(50).cloned().collect())
}