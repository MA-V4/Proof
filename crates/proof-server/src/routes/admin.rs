use axum::{extract::State, Json};
use crate::{error::AppResult, state::SharedState};

pub async fn reset(State(state): State<SharedState>) -> AppResult<Json<serde_json::Value>> {
    let s = state.read().await;
    let db = s.db.clone();
    drop(s);

    sqlx::query("DELETE FROM divergences").execute(&db.pool).await?;
    sqlx::query("DELETE FROM verification_events").execute(&db.pool).await?;
    sqlx::query("DELETE FROM audit_entries").execute(&db.pool).await?;

    let mut s = state.write().await;
    s.events_verified = 0;
    s.recent.clear();

    Ok(Json(serde_json::json!({ "ok": true })))
}